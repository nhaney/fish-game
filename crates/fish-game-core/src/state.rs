use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

use crate::boat::{
    self, Boat, BoatId, Hook, HookId, Line, LineId, Worm, WormId, HOOK_SIZE,
};
use crate::collision::aabb_overlap;
use crate::config::FishGameConfig;
use crate::input::FishGameInput;
use crate::player::{
    self, ActiveBoost, BoostCooldown, Player, PlayerState,
};
use crate::rng::GameRng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Running,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOverCause {
    Hooked,
    Starved,
    Bonked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub count: u32,
    pub interval_ticks_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difficulty {
    pub multiplier: u8,
    pub interval_ticks_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnTicks {
    pub boats_interval_remaining: u32,
}

/// The complete simulation state. Owned by core. Presentation reads it and
/// diffs against the previous tick's snapshot to drive SFX/VFX/UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishGameState {
    pub tick: u64,
    pub phase: GamePhase,
    pub game_over_cause: Option<GameOverCause>,
    pub game_over_boat: Option<BoatId>,
    pub rng: GameRng,
    pub player: Player,
    pub boats: SlotMap<BoatId, Boat>,
    pub hooks: SlotMap<HookId, Hook>,
    pub lines: SlotMap<LineId, Line>,
    pub worms: SlotMap<WormId, Worm>,
    pub score: Score,
    pub difficulty: Difficulty,
    pub spawn_ticks: SpawnTicks,
    pub config: FishGameConfig,
}

impl FishGameState {
    pub fn new(config: FishGameConfig) -> Self {
        let rng = GameRng::from_seed(config.seed);
        let player = Player::new(&config.player);
        Self {
            tick: 0,
            phase: GamePhase::Running,
            game_over_cause: None,
            game_over_boat: None,
            rng,
            player,
            boats: SlotMap::with_key(),
            hooks: SlotMap::with_key(),
            lines: SlotMap::with_key(),
            worms: SlotMap::with_key(),
            score: Score {
                count: 0,
                interval_ticks_remaining: config.score_interval_ticks,
            },
            difficulty: Difficulty {
                multiplier: 1,
                interval_ticks_remaining: config.difficulty_interval_ticks,
            },
            spawn_ticks: SpawnTicks {
                boats_interval_remaining: config.boat_spawn_interval_ticks,
            },
            config,
        }
    }

    /// Advance the simulation by one fixed tick. State is owned by core;
    /// callers observe it through the returned reference (no event stream).
    pub fn tick(&mut self, input: FishGameInput) -> &FishGameState {
        self.tick = self.tick.wrapping_add(1);

        match self.phase {
            GamePhase::Running => tick_running(self, &input),
            GamePhase::GameOver => tick_game_over(self),
        }

        self
    }

    /// Deterministic hash of simulation-critical state. Excludes presentation-
    /// only fields (there are none here — the whole state is canonical).
    pub fn hash(&self) -> u64 {
        let mut h = FxHasher::default();
        hash_state(self, &mut h);
        h.finish()
    }

    /// Position of a hook in world space. Presentation/collision callers use
    /// this — it's just a field accessor now that hooks store world pos.
    pub fn hook_world_pos(&self, hook_id: HookId) -> Option<Vec3> {
        self.hooks.get(hook_id).map(|h| h.pos)
    }
}

fn tick_running(state: &mut FishGameState, input: &FishGameInput) {
    let dt = state.config.dt();

    // 1. EmitEvents / timers.
    tick_score_and_difficulty(state);
    tick_boost_cooldown(state, input);
    tick_hunger(state);
    tick_boat_spawner(state);

    // 2. Movement — input-driven velocity + boost + sink.
    tick_player_input_movement(state, input);
    tick_player_boost(state);
    apply_sink(state);

    // 3. FinalizeMovement — integrate positions.
    integrate_positions(state, dt);

    // 4. CalculateCollisions.
    clamp_player_to_arena(state);
    check_collisions(state);
}

/// After GameOver, boats continue sailing off and reel-in completes, but no
/// more collisions / scoring / spawns / player movement.
fn tick_game_over(state: &mut FishGameState) {
    let dt = state.config.dt();
    boat::step_boats(
        dt,
        &mut state.boats,
        &mut state.hooks,
        &mut state.lines,
        &mut state.worms,
    );
    boat::step_reeling_hooks(dt, &mut state.hooks, &mut state.lines);
    boat::despawn_offscreen_boats(
        &state.config.arena,
        &mut state.boats,
        &mut state.hooks,
        &mut state.lines,
        &mut state.worms,
    );
}

fn tick_score_and_difficulty(state: &mut FishGameState) {
    let s = &mut state.score;
    if s.interval_ticks_remaining > 0 {
        s.interval_ticks_remaining -= 1;
    }
    if s.interval_ticks_remaining == 0 {
        s.count += 1;
        s.interval_ticks_remaining = state.config.score_interval_ticks;
    }

    let d = &mut state.difficulty;
    if d.interval_ticks_remaining > 0 {
        d.interval_ticks_remaining -= 1;
    }
    if d.interval_ticks_remaining == 0 {
        if d.multiplier < state.config.max_difficulty {
            d.multiplier += 1;
        }
        d.interval_ticks_remaining = state.config.difficulty_interval_ticks;
    }
}

fn tick_boost_cooldown(state: &mut FishGameState, input: &FishGameInput) {
    let Some(mut cd) = state.player.boost_cooldown else {
        state.player.boost_blocked = false;
        return;
    };

    if cd.ticks_remaining > 0 {
        cd.ticks_remaining -= 1;
    }
    cd.did_release = cd.did_release || !input.boost_pressed;

    if cd.ticks_remaining == 0 && cd.did_release {
        state.player.boost_cooldown = None;
        state.player.boost_blocked = false;
    } else {
        state.player.boost_cooldown = Some(cd);
        state.player.boost_blocked = true;
    }
}

fn tick_hunger(state: &mut FishGameState) {
    if state.player.hunger_ticks_remaining > 0 {
        state.player.hunger_ticks_remaining -= 1;
    }
    if state.player.hunger_ticks_remaining == 0 {
        enter_game_over(state, GameOverCause::Starved, None);
    }
}

fn tick_boat_spawner(state: &mut FishGameState) {
    let s = &mut state.spawn_ticks;
    if s.boats_interval_remaining > 0 {
        s.boats_interval_remaining -= 1;
    }
    if s.boats_interval_remaining == 0 {
        let count = boat::roll_boats_per_spawn(state.difficulty.multiplier, &mut state.rng.rng);
        for _ in 0..count {
            boat::spawn_random_boat(
                &state.config,
                state.difficulty.multiplier,
                &mut state.rng.rng,
                &mut state.boats,
                &mut state.hooks,
                &mut state.lines,
                &mut state.worms,
            );
        }
        s.boats_interval_remaining = state.config.boat_spawn_interval_ticks;
    }
}

fn tick_player_input_movement(state: &mut FishGameState, input: &FishGameInput) {
    if state.player.state == PlayerState::Boost {
        return;
    }

    let mut velocity = state.player.velocity;
    let mut facing_right = state.player.facing_right;
    let target_speed =
        player::apply_input(&state.config.player, input, &mut velocity, &mut facing_right);
    state.player.velocity = velocity;
    state.player.facing_right = facing_right;

    if input.boost_just_pressed {
        player::try_start_boost(&mut state.player, &state.config.player, target_speed);
    } else if target_speed != Vec3::ZERO {
        if player::can_transition_to(state.player.state, PlayerState::Swim, state.player.boost_blocked) {
            state.player.state = PlayerState::Swim;
        }
    } else if player::can_transition_to(state.player.state, PlayerState::Idle, state.player.boost_blocked) {
        state.player.state = PlayerState::Idle;
    }
}

fn tick_player_boost(state: &mut FishGameState) {
    let Some(mut boost) = state.player.boost_data else {
        return;
    };
    if state.player.state != PlayerState::Boost {
        return;
    }

    state.player.velocity = boost.velocity;

    if boost.ticks_remaining > 0 {
        boost.ticks_remaining -= 1;
    }

    if boost.ticks_remaining == 0 {
        state.player.state = boost.prev_state;
        state.player.boost_data = None;
    } else {
        state.player.boost_data = Some(boost);
    }
}

fn apply_sink(state: &mut FishGameState) {
    state.player.velocity.y -= state.config.player.sink_weight;
}

fn integrate_positions(state: &mut FishGameState, dt: f32) {
    state.player.pos.x += state.player.velocity.x * dt;
    state.player.pos.y += state.player.velocity.y * dt;

    boat::step_boats(
        dt,
        &mut state.boats,
        &mut state.hooks,
        &mut state.lines,
        &mut state.worms,
    );
    boat::step_reeling_hooks(dt, &mut state.hooks, &mut state.lines);
}

fn clamp_player_to_arena(state: &mut FishGameState) {
    let arena = &state.config.arena;
    let half_w = arena.width / 2.0;
    let half_h = arena.height / 2.0;
    let p_half_w = state.config.player.width / 2.0;
    let p_half_h = state.config.player.height / 2.0;

    let pos = &mut state.player.pos;
    if pos.x - p_half_w < -half_w {
        pos.x = -half_w + p_half_w;
    }
    if pos.x + p_half_w > half_w {
        pos.x = half_w - p_half_w;
    }
    if pos.y - p_half_h < -half_h {
        pos.y = -half_h + p_half_h;
    }
    if pos.y > half_h + arena.offset {
        pos.y = half_h + arena.offset;
    }
}

fn check_collisions(state: &mut FishGameState) {
    let p_center = Vec2::new(state.player.pos.x, state.player.pos.y);
    let p_half = Vec2::new(
        state.config.player.width / 2.0,
        state.config.player.height / 2.0,
    );

    // Hook collision — ends game (hooked). Take the first hit for
    // determinism; slotmap iteration is deterministic so this is stable.
    let mut hit_hook: Option<HookId> = None;
    for (hid, hook) in state.hooks.iter() {
        let h_center = Vec2::new(hook.pos.x, hook.pos.y);
        let h_half = Vec2::new(hook.width / 2.0, hook.height / 2.0);
        if aabb_overlap(p_center, p_half, h_center, h_half) {
            hit_hook = Some(hid);
            break;
        }
    }
    if let Some(hid) = hit_hook {
        let boat_id = state.hooks.get(hid).map(|h| h.boat_id);
        enter_game_over(state, GameOverCause::Hooked, boat_id);
        boat::start_reel_in(hid, &mut state.hooks, &state.lines);
        return;
    }

    // Worm collision — eat one worm per tick (first hit).
    let mut eaten_worm: Option<WormId> = None;
    for (wid, worm) in state.worms.iter() {
        let w_center = Vec2::new(worm.pos.x, worm.pos.y);
        let w_half = Vec2::new(worm.width / 2.0, worm.height / 2.0);
        if aabb_overlap(p_center, p_half, w_center, w_half) {
            eaten_worm = Some(wid);
            break;
        }
    }
    if let Some(wid) = eaten_worm {
        boat::despawn_worm(wid, &mut state.worms, &mut state.boats);
        state.score.count += state.config.score_per_worm;
        let extra = state.config.player.extra_hunger_ticks_per_worm;
        let cap = state.config.player.hunger_ticks;
        state.player.hunger_ticks_remaining =
            (state.player.hunger_ticks_remaining + extra).min(cap.saturating_mul(4));
        if state.player.boosts_remaining < state.config.player.max_boosts {
            state.player.boosts_remaining += 1;
        }
    }

    // Boat collision — ends game (bonked).
    let mut hit_boat: Option<BoatId> = None;
    for (bid, boat) in state.boats.iter() {
        let b_center = Vec2::new(boat.pos.x, boat.pos.y);
        let b_half = Vec2::new(boat.width / 2.0, boat.height / 2.0);
        if aabb_overlap(p_center, p_half, b_center, b_half) {
            hit_boat = Some(bid);
            break;
        }
    }
    if let Some(bid) = hit_boat {
        enter_game_over(state, GameOverCause::Bonked, Some(bid));
    }
}

fn enter_game_over(state: &mut FishGameState, cause: GameOverCause, boat: Option<BoatId>) {
    if state.phase == GamePhase::GameOver {
        return;
    }
    state.phase = GamePhase::GameOver;
    state.game_over_cause = Some(cause);
    state.game_over_boat = boat;
    boat::despawn_all_worms(&mut state.worms, &mut state.boats);
    boat::trigger_boat_exit(boat, &mut state.boats);
    // Freeze player velocity — post-death "animation" is presentation.
    state.player.velocity = Vec3::ZERO;
    state.player.boost_data = None;
}

// --- Deterministic hashing --------------------------------------------------
//
// FxHasher-style: fast, deterministic, not secure (we never use it for
// security). Hashing the full state makes the hash sensitive to any mistake,
// which is exactly what we want for replay verification.

#[derive(Default)]
struct FxHasher {
    h: u64,
}

impl FxHasher {
    fn write_u64(&mut self, x: u64) {
        // rustc's FxHasher constants, inlined so we don't depend on a crate.
        const ROTATE: u32 = 5;
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        self.h = (self.h.rotate_left(ROTATE) ^ x).wrapping_mul(SEED);
    }
    fn write_u32(&mut self, x: u32) {
        self.write_u64(x as u64);
    }
    fn write_f32(&mut self, x: f32) {
        self.write_u32(x.to_bits());
    }
    fn write_bool(&mut self, x: bool) {
        self.write_u64(x as u64);
    }
    fn finish(&self) -> u64 {
        self.h
    }
}

fn hash_vec3(v: Vec3, h: &mut FxHasher) {
    h.write_f32(v.x);
    h.write_f32(v.y);
    h.write_f32(v.z);
}

fn hash_state(state: &FishGameState, h: &mut FxHasher) {
    h.write_u64(state.tick);
    h.write_u32(state.phase as u32);
    h.write_u32(state.game_over_cause.map(|c| c as u32).unwrap_or(u32::MAX));

    h.write_u32(state.rng.seed[0] as u32);
    // We don't hash the live rng internals (ChaCha8Rng doesn't expose them) —
    // instead we hash the seed stream. Divergence shows up through entities.

    hash_vec3(state.player.pos, h);
    hash_vec3(state.player.velocity, h);
    h.write_bool(state.player.facing_right);
    h.write_u32(state.player.state as u32);
    h.write_u32(state.player.boosts_remaining as u32);
    h.write_u32(state.player.hunger_ticks_remaining);

    h.write_u32(state.score.count);
    h.write_u32(state.score.interval_ticks_remaining);
    h.write_u32(state.difficulty.multiplier as u32);
    h.write_u32(state.difficulty.interval_ticks_remaining);
    h.write_u32(state.spawn_ticks.boats_interval_remaining);

    h.write_u32(state.boats.len() as u32);
    for (_, boat) in state.boats.iter() {
        hash_vec3(boat.pos, h);
        hash_vec3(boat.velocity, h);
        h.write_bool(boat.facing_right);
        h.write_f32(boat.width);
        h.write_f32(boat.height);
        h.write_u32(boat.boat_type as u32);
        h.write_u32(boat.hook_ids.len() as u32);
        h.write_u32(boat.line_ids.len() as u32);
        h.write_u32(boat.worm_ids.len() as u32);
        h.write_bool(boat.exiting);
        h.write_bool(boat.winner);
    }

    h.write_u32(state.hooks.len() as u32);
    for (_, hook) in state.hooks.iter() {
        hash_vec3(hook.pos, h);
        h.write_bool(hook.reel_velocity.is_some());
    }

    h.write_u32(state.worms.len() as u32);
    for (_, worm) in state.worms.iter() {
        hash_vec3(worm.pos, h);
    }

    h.write_u32(state.lines.len() as u32);
    for (_, line) in state.lines.iter() {
        hash_vec3(line.start_pos, h);
        hash_vec3(line.end_pos, h);
    }

    // Mark unused imports as used in release builds.
    let _ = HOOK_SIZE;
    let _: Option<ActiveBoost> = None;
    let _: Option<BoostCooldown> = None;
}
