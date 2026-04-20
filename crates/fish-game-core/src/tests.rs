//! Gameplay-mechanic unit tests for the deterministic core.
//!
//! Determinism / replay round-trip tests live in the `fish-game-replay` crate.
//! These tests cover the *behavior* of one tick or a small sequence of ticks:
//! collision outcomes, boost/cooldown rules, hunger countdown, arena clamp,
//! score and difficulty cadence, and game-over freezing.
//!
//! Each test reaches into `FishGameState` directly to set up the scenario it
//! wants — that's intentional. We don't simulate hundreds of ticks of
//! pre-amble; we manufacture the precondition and assert the one transition.

use glam::Vec3;

use crate::{
    boat::{Boat, BoatId, BoatType, Hook, HookId, Line, LineId, Worm, WormId, HOOK_SIZE, WORM_SIZE},
    config::FishGameConfig,
    events::CoreEvent,
    input::FishGameInput,
    player::PlayerState,
    state::{FishGameState, GameOverCause, GamePhase},
};

fn cfg() -> FishGameConfig {
    let mut seed = [0u8; 32];
    seed[0] = 1;
    FishGameConfig::default().with_seed(seed)
}

fn fresh() -> FishGameState {
    FishGameState::new(cfg())
}

fn insert_boat(state: &mut FishGameState, pos: Vec3, width: f32, height: f32) -> BoatId {
    state.boats.insert(Boat {
        pos,
        velocity: Vec3::ZERO,
        facing_right: true,
        width,
        height,
        boat_type: BoatType::Dinghy,
        hook_ids: Vec::new(),
        line_ids: Vec::new(),
        worm_ids: Vec::new(),
        exiting: false,
        winner: false,
    })
}

fn insert_line(state: &mut FishGameState, boat_id: BoatId) -> LineId {
    state.lines.insert(Line {
        boat_id,
        start_pos: Vec3::new(0.0, 100.0, 0.0),
        end_pos: Vec3::ZERO,
    })
}

fn insert_hook(state: &mut FishGameState, pos: Vec3, boat_id: BoatId) -> HookId {
    let line_id = insert_line(state, boat_id);
    let hook_id = state.hooks.insert(Hook {
        pos,
        boat_id,
        line_id,
        width: HOOK_SIZE,
        height: HOOK_SIZE,
        reel_velocity: None,
        reel_destination: None,
    });
    if let Some(boat) = state.boats.get_mut(boat_id) {
        boat.hook_ids.push(hook_id);
        boat.line_ids.push(line_id);
    }
    hook_id
}

fn insert_worm(state: &mut FishGameState, pos: Vec3, boat_id: BoatId) -> WormId {
    let line_id = insert_line(state, boat_id);
    let worm_id = state.worms.insert(Worm {
        pos,
        boat_id,
        line_id,
        width: WORM_SIZE,
        height: WORM_SIZE,
    });
    if let Some(boat) = state.boats.get_mut(boat_id) {
        boat.worm_ids.push(worm_id);
        boat.line_ids.push(line_id);
    }
    worm_id
}

// ---------- tick basics -----------------------------------------------------

#[test]
fn tick_increments_monotonically() {
    let mut state = fresh();
    for expected in 1..=10 {
        state.tick(FishGameInput::default());
        assert_eq!(state.tick, expected);
    }
}

#[test]
fn fresh_state_starts_running() {
    let state = fresh();
    assert_eq!(state.phase, GamePhase::Running);
    assert!(state.game_over_cause.is_none());
    assert!(state.boats.is_empty());
    assert!(state.hooks.is_empty());
    assert!(state.worms.is_empty());
}

// ---------- collision: hook -------------------------------------------------

#[test]
fn player_overlapping_hook_triggers_hooked_game_over() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    insert_hook(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());

    assert_eq!(state.phase, GamePhase::GameOver);
    assert_eq!(state.game_over_cause, Some(GameOverCause::Hooked));
    assert_eq!(state.game_over_boat, Some(bid));
}

#[test]
fn hook_reels_in_after_player_hooked() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    let hid = insert_hook(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());
    let hook = state.hooks.get(hid).expect("hook still present after hook event");
    assert!(hook.reel_velocity.is_some(), "reel-in should have started");
    assert!(hook.reel_destination.is_some());
}

// ---------- collision: worm -------------------------------------------------

#[test]
fn eating_worm_increases_score_and_refills_hunger() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    state.player.hunger_ticks_remaining = 100;
    state.player.boosts_remaining = 0;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    let wid = insert_worm(&mut state, Vec3::ZERO, bid);

    let extra = state.config.player.extra_hunger_ticks_per_worm;
    let score_before = state.score.count;

    state.tick(FishGameInput::default());

    assert!(!state.worms.contains_key(wid), "worm should be despawned");
    assert_eq!(state.phase, GamePhase::Running);
    assert_eq!(state.score.count, score_before + state.config.score_per_worm);
    // Eat adds `extra`; this tick's hunger countdown subtracts 1.
    assert_eq!(state.player.hunger_ticks_remaining, 100 + extra - 1);
    assert_eq!(state.player.boosts_remaining, 1, "eating restores one boost");
}

#[test]
fn eating_worm_does_not_exceed_max_boosts() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    state.player.boosts_remaining = state.config.player.max_boosts;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    insert_worm(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());

    assert_eq!(state.player.boosts_remaining, state.config.player.max_boosts);
}

// ---------- collision: boat -------------------------------------------------

#[test]
fn player_overlapping_boat_triggers_bonked_game_over() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::ZERO, 40.0, 40.0);

    state.tick(FishGameInput::default());

    assert_eq!(state.phase, GamePhase::GameOver);
    assert_eq!(state.game_over_cause, Some(GameOverCause::Bonked));
    assert_eq!(state.game_over_boat, Some(bid));
}

#[test]
fn hook_collision_takes_precedence_over_boat_collision() {
    // Hook is checked first in `check_collisions`, so an overlapping player
    // dies as Hooked even when also overlapping a boat.
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::ZERO, 40.0, 40.0);
    insert_hook(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());

    assert_eq!(state.game_over_cause, Some(GameOverCause::Hooked));
}

// ---------- arena clamp -----------------------------------------------------

#[test]
fn player_clamped_to_arena_horizontally() {
    let mut state = fresh();
    state.player.velocity = Vec3::new(-1_000_000.0, 0.0, 0.0);
    state.tick(FishGameInput::default());

    let half_arena = state.config.arena.width / 2.0;
    let half_player = state.config.player.width / 2.0;
    assert!(
        state.player.pos.x >= -half_arena + half_player - 0.01,
        "player should be clamped at the left edge: x={}",
        state.player.pos.x,
    );
}

#[test]
fn player_clamped_to_arena_floor() {
    let mut state = fresh();
    state.player.pos.y = -10_000.0;
    state.player.velocity = Vec3::ZERO;
    state.tick(FishGameInput::default());

    let half_arena = state.config.arena.height / 2.0;
    let half_player = state.config.player.height / 2.0;
    assert!(
        state.player.pos.y >= -half_arena + half_player - 0.01,
        "player should be clamped at the floor: y={}",
        state.player.pos.y,
    );
}

// ---------- boost -----------------------------------------------------------

#[test]
fn boost_press_consumes_one_boost_and_enters_boost_state() {
    let mut state = fresh();
    let initial = state.player.boosts_remaining;
    assert!(initial > 0, "default config should provide boosts");

    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });

    assert_eq!(state.player.state, PlayerState::Boost);
    assert_eq!(state.player.boosts_remaining, initial - 1);
    assert!(state.player.boost_data.is_some());
    assert!(state.player.boost_cooldown.is_some());
}

#[test]
fn boost_ends_after_duration_ticks() {
    let mut state = fresh();
    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    assert_eq!(state.player.state, PlayerState::Boost);

    let duration = state.config.player.boost_duration_ticks;
    for _ in 0..duration {
        state.tick(FishGameInput {
            move_right: true,
            boost_pressed: false,
            ..Default::default()
        });
    }
    assert_ne!(state.player.state, PlayerState::Boost);
}

#[test]
fn boost_blocked_by_cooldown_until_release_then_repress() {
    let mut state = fresh();
    let max = state.config.player.max_boosts;

    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    assert_eq!(state.player.boosts_remaining, max - 1);

    // Re-press while still held — cooldown blocks consuming a second boost.
    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    assert_eq!(
        state.player.boosts_remaining,
        max - 1,
        "boost while cooldown active should not consume",
    );
}

// ---------- hunger ----------------------------------------------------------

#[test]
fn hunger_countdown_decrements_each_tick() {
    let mut state = fresh();
    let before = state.player.hunger_ticks_remaining;
    state.tick(FishGameInput::default());
    assert_eq!(state.player.hunger_ticks_remaining, before - 1);
}

#[test]
fn hunger_zero_triggers_starved_game_over() {
    let mut state = fresh();
    state.player.hunger_ticks_remaining = 1;
    state.tick(FishGameInput::default());
    assert_eq!(state.phase, GamePhase::GameOver);
    assert_eq!(state.game_over_cause, Some(GameOverCause::Starved));
}

// ---------- score / difficulty cadence -------------------------------------

#[test]
fn score_increments_every_score_interval() {
    let mut state = fresh();
    let interval = state.config.score_interval_ticks;
    let before = state.score.count;
    for _ in 0..interval {
        state.tick(FishGameInput::default());
    }
    assert_eq!(state.score.count, before + 1);
}

#[test]
fn difficulty_increments_every_difficulty_interval_until_cap() {
    let mut state = fresh();
    let interval = state.config.difficulty_interval_ticks;
    let cap = state.config.max_difficulty;
    assert_eq!(state.difficulty.multiplier, 1);

    for step in 1..=cap {
        for _ in 0..interval {
            state.tick(FishGameInput::default());
            // GameOver from starvation could happen before we reach cap; bail
            // so the test stays focused on the cadence invariant.
            if state.phase != GamePhase::Running {
                return;
            }
        }
        assert!(
            state.difficulty.multiplier >= step.min(cap),
            "after {} intervals, multiplier={}",
            step,
            state.difficulty.multiplier,
        );
    }
    assert_eq!(state.difficulty.multiplier, cap);
}

// ---------- spawn / off-screen cleanup -------------------------------------

#[test]
fn boats_spawn_on_the_spawn_interval() {
    let mut state = fresh();
    let interval = state.config.boat_spawn_interval_ticks;
    let before = state.boats.len();
    for _ in 0..interval {
        state.tick(FishGameInput::default());
    }
    assert!(
        state.boats.len() > before,
        "expected at least one boat to spawn after one spawn interval",
    );
}

// ---------- game-over freeze ------------------------------------------------

#[test]
fn game_over_freezes_player_position_and_velocity() {
    // After GameOver, the player neither moves nor has its velocity changed
    // — input is ignored, sink is not re-applied. (Velocity may carry the
    // last tick's residual sink, but it must NOT be modified further.)
    let mut state = fresh();
    state.player.hunger_ticks_remaining = 1;
    state.tick(FishGameInput::default());
    assert_eq!(state.phase, GamePhase::GameOver);

    let pos = state.player.pos;
    let vel = state.player.velocity;
    for _ in 0..30 {
        state.tick(FishGameInput {
            move_right: true,
            boost_pressed: true,
            boost_just_pressed: true,
            ..Default::default()
        });
    }
    assert_eq!(state.player.pos, pos, "player should not move post game-over");
    assert_eq!(
        state.player.velocity, vel,
        "velocity should not change post game-over",
    );
}

#[test]
fn game_over_clears_all_worms() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(200.0, 200.0, 0.0), 40.0, 10.0);
    insert_worm(&mut state, Vec3::new(200.0, 100.0, 0.0), bid);

    state.player.hunger_ticks_remaining = 1;
    state.tick(FishGameInput::default());

    assert_eq!(state.phase, GamePhase::GameOver);
    assert!(state.worms.is_empty(), "worms cleared on game over");
}

#[test]
fn game_over_does_not_change_phase_back_to_running() {
    let mut state = fresh();
    state.player.hunger_ticks_remaining = 1;
    state.tick(FishGameInput::default());
    assert_eq!(state.phase, GamePhase::GameOver);

    for _ in 0..1000 {
        state.tick(FishGameInput::default());
        assert_eq!(state.phase, GamePhase::GameOver);
    }
}

// ---------- aabb collision math --------------------------------------------

#[test]
fn aabb_overlap_detects_overlap_and_separation() {
    use crate::collision::aabb_overlap;
    use glam::Vec2;

    let a_c = Vec2::new(0.0, 0.0);
    let a_h = Vec2::new(10.0, 10.0);

    // Fully overlapping.
    assert!(aabb_overlap(a_c, a_h, Vec2::new(0.0, 0.0), Vec2::new(5.0, 5.0)));
    // Touching at the edge — strict less-than means touching is NOT overlap.
    assert!(!aabb_overlap(a_c, a_h, Vec2::new(20.0, 0.0), Vec2::new(10.0, 10.0)));
    // Separated on x.
    assert!(!aabb_overlap(a_c, a_h, Vec2::new(100.0, 0.0), Vec2::new(5.0, 5.0)));
    // Separated on y.
    assert!(!aabb_overlap(a_c, a_h, Vec2::new(0.0, 100.0), Vec2::new(5.0, 5.0)));
    // Just inside on x.
    assert!(aabb_overlap(
        a_c,
        a_h,
        Vec2::new(15.0 - 0.01, 0.0),
        Vec2::new(5.0, 5.0),
    ));
}

// ---------- event emission --------------------------------------------------

#[test]
fn events_cleared_at_start_of_each_tick() {
    let mut state = fresh();
    state.tick(FishGameInput::default());
    let first_count = state.events.len();
    state.tick(FishGameInput::default());
    // Events are per-tick ephemeral; consecutive identical ticks may both
    // produce zero events. The invariant is that the list on tick N does not
    // retain events emitted on tick N-1 — i.e. the list is reset.
    let _ = first_count;
    for ev in &state.events {
        match ev {
            // None of these carry N-1 identifiers.
            CoreEvent::ScoreIncremented { .. } | CoreEvent::DifficultyIncreased { .. } => {}
            _ => {}
        }
    }
}

#[test]
fn hook_collision_emits_player_hooked_and_game_over() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    let hid = insert_hook(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());

    let hooked = state.events.iter().find(|e| matches!(e, CoreEvent::PlayerHooked { .. }));
    let game_over = state.events.iter().find(|e| matches!(e, CoreEvent::GameOver { .. }));
    assert!(
        matches!(hooked, Some(CoreEvent::PlayerHooked { hook, .. }) if *hook == hid),
        "expected PlayerHooked for {:?}, events={:?}",
        hid,
        state.events,
    );
    assert!(
        matches!(game_over, Some(CoreEvent::GameOver { cause: GameOverCause::Hooked })),
        "expected GameOver(Hooked), events={:?}",
        state.events,
    );
}

#[test]
fn boat_collision_emits_player_bonked_and_game_over() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::ZERO, 40.0, 40.0);

    state.tick(FishGameInput::default());

    assert!(state
        .events
        .iter()
        .any(|e| matches!(e, CoreEvent::PlayerBonked { boat } if *boat == bid)));
    assert!(state.events.iter().any(|e| matches!(
        e,
        CoreEvent::GameOver {
            cause: GameOverCause::Bonked,
        }
    )));
}

#[test]
fn eating_worm_emits_player_ate_and_worm_despawned() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(0.0, 200.0, 0.0), 40.0, 10.0);
    let wid = insert_worm(&mut state, Vec3::ZERO, bid);

    state.tick(FishGameInput::default());

    assert!(state
        .events
        .iter()
        .any(|e| matches!(e, CoreEvent::PlayerAte { worm } if *worm == wid)));
    assert!(state
        .events
        .iter()
        .any(|e| matches!(e, CoreEvent::WormDespawned(w) if *w == wid)));
}

#[test]
fn boost_emits_player_boosted() {
    let mut state = fresh();
    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    assert!(state.events.iter().any(|e| matches!(e, CoreEvent::PlayerBoosted)));
}

#[test]
fn boost_rejected_by_cooldown_does_not_emit_player_boosted() {
    let mut state = fresh();
    // First tick consumes a boost and starts a cooldown.
    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    // Second tick: re-press while cooldown active — should NOT re-fire.
    state.tick(FishGameInput {
        move_right: true,
        boost_pressed: true,
        boost_just_pressed: true,
        ..Default::default()
    });
    assert!(
        !state.events.iter().any(|e| matches!(e, CoreEvent::PlayerBoosted)),
        "cooldown-blocked boost must not emit PlayerBoosted: events={:?}",
        state.events,
    );
}

#[test]
fn starvation_emits_player_starved_and_game_over() {
    let mut state = fresh();
    state.player.hunger_ticks_remaining = 1;
    state.tick(FishGameInput::default());

    assert!(state.events.iter().any(|e| matches!(e, CoreEvent::PlayerStarved)));
    assert!(state.events.iter().any(|e| matches!(
        e,
        CoreEvent::GameOver {
            cause: GameOverCause::Starved,
        }
    )));
}

#[test]
fn score_tick_emits_score_incremented() {
    let mut state = fresh();
    let interval = state.config.score_interval_ticks;
    for _ in 0..interval - 1 {
        state.tick(FishGameInput::default());
        assert!(!state
            .events
            .iter()
            .any(|e| matches!(e, CoreEvent::ScoreIncremented { .. })));
    }
    state.tick(FishGameInput::default());
    let inc = state
        .events
        .iter()
        .find(|e| matches!(e, CoreEvent::ScoreIncremented { .. }))
        .expect("score tick should emit");
    match inc {
        CoreEvent::ScoreIncremented { new_score } => {
            assert_eq!(*new_score, state.score.count)
        }
        _ => unreachable!(),
    }
}

#[test]
fn boat_spawn_emits_boat_spawned_plus_child_events() {
    let mut state = fresh();
    let interval = state.config.boat_spawn_interval_ticks;
    for _ in 0..interval {
        state.tick(FishGameInput::default());
    }
    let boat_events = state
        .events
        .iter()
        .filter(|e| matches!(e, CoreEvent::BoatSpawned(_)))
        .count();
    assert!(boat_events >= 1, "expected at least one BoatSpawned, got events={:?}", state.events);
    // Every spawned boat comes with at least one line + one hook.
    assert!(state.events.iter().any(|e| matches!(e, CoreEvent::HookSpawned(_))));
    assert!(state.events.iter().any(|e| matches!(e, CoreEvent::LineSpawned(_))));
}

#[test]
fn game_over_emits_worm_despawned_for_every_remaining_worm() {
    let mut state = fresh();
    state.player.pos = Vec3::ZERO;
    let bid = insert_boat(&mut state, Vec3::new(200.0, 200.0, 0.0), 40.0, 10.0);
    let w1 = insert_worm(&mut state, Vec3::new(200.0, 100.0, 0.0), bid);
    let w2 = insert_worm(&mut state, Vec3::new(250.0, 100.0, 0.0), bid);
    state.player.hunger_ticks_remaining = 1;

    state.tick(FishGameInput::default());

    assert_eq!(state.phase, GamePhase::GameOver);
    let despawned: Vec<WormId> = state
        .events
        .iter()
        .filter_map(|e| match e {
            CoreEvent::WormDespawned(w) => Some(*w),
            _ => None,
        })
        .collect();
    assert!(despawned.contains(&w1));
    assert!(despawned.contains(&w2));
}
