//! Bridge between `fish-game-core` (the deterministic simulation) and Bevy.
//!
//! The core owns all simulation state. The adapter:
//!
//!   1. Builds a [`FishGameInput`] from current keyboard state each fixed tick.
//!   2. Snapshots `CoreState.prev` before calling `tick`.
//!   3. Diffs the post-tick state against the snapshot and emits the legacy
//!      Bevy events (`PlayerHooked`, `PlayerAte`, `GameOver`, …). Existing
//!      SFX/UI consumers stay as-is.
//!   4. Mirrors core entities (boats, hooks, worms) into Bevy entities via a
//!      `CoreEntityMap`, spawning/despawning as slotmap membership changes.
//!
//! Lifecycle (pause / restart) lives here, NOT in core:
//!   - Pause ⇒ `CoreControl.paused` flips; `tick_core_from_input` returns
//!     early so core sees no ticks while paused.
//!   - Restart ⇒ rebuild `CoreState` with a freshly-derived seed. The old
//!     simulation ends cleanly; the new one starts at tick 0.

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use fish_game_core::boat::{BoatId, HookId, LineId, WormId};
use fish_game_core::{FishGameConfig, FishGameInput, FishGameState, GameOverCause, GamePhase};
use rand::{thread_rng, Rng};
use std::collections::BTreeMap;

use crate::player::events::{
    PlayerAte, PlayerBonked, PlayerBoosted, PlayerHooked, PlayerStarved,
};
use crate::shared::collision::Collider;
use crate::shared::game::{GameOver, GamePaused, GameRestarted, GameUnpaused};
use crate::shared::render::RenderLayer;

/// Resource wrapping the deterministic simulation. `state` is the source of
/// truth; `prev` is the snapshot taken just before the current tick and used
/// to derive events on the same frame.
#[derive(Resource)]
pub struct CoreState {
    pub state: FishGameState,
    pub prev: FishGameState,
}

impl CoreState {
    pub fn new(config: FishGameConfig) -> Self {
        let state = FishGameState::new(config);
        let prev = state.clone();
        Self { state, prev }
    }

    /// Replace the current sim with a fresh one (same tuning, new seed). The
    /// previous-state snapshot is reset to match so diff-based event
    /// emission doesn't fire spurious "started" transitions from leftover
    /// state. Used by restart.
    pub fn reset_with(&mut self, config: FishGameConfig) {
        self.state = FishGameState::new(config);
        self.prev = self.state.clone();
    }
}

/// Adapter-level lifecycle controls. Pause and restart are presentation
/// concerns — core doesn't know about them.
#[derive(Resource, Default)]
pub struct CoreControl {
    pub paused: bool,
    /// Set to true when the UI wants a pause toggle on the next tick boundary.
    /// Bounced through a resource so UI doesn't need to know about `FishGameInput`.
    pub pause_toggle_pending: bool,
    /// Set to true on the tick the sim was just restarted. Consumed by the
    /// diff-based event emitter to fire a single `GameRestarted` event.
    pub restart_fired: bool,
}

/// Maps core slotmap keys to Bevy `Entity` so presentation can sync sprites.
#[derive(Resource, Default)]
pub struct CoreEntityMap {
    pub boats: BTreeMap<BoatId, Entity>,
    pub hooks: BTreeMap<HookId, Entity>,
    pub lines: BTreeMap<LineId, Entity>,
    pub worms: BTreeMap<WormId, Entity>,
}

/// Resource holding the lyon stroke color / texture handles for hook / line /
/// worm rendering. Built from `AssetServer` at startup.
#[derive(Resource)]
pub struct BoatAssets {
    pub boat: Handle<Image>,
    pub hook: Handle<Image>,
    pub worm_frame1: Handle<Image>,
    pub worm_frame2: Handle<Image>,
    pub line_color: Color,
}

impl FromWorld for BoatAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        Self {
            boat: asset_server.load("sprites/boat/boat.png"),
            hook: asset_server.load("sprites/hook/hook.png"),
            worm_frame1: asset_server.load("sprites/worm/worm1.png"),
            worm_frame2: asset_server.load("sprites/worm/worm2.png"),
            line_color: Color::BLACK,
        }
    }
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        let config = FishGameConfig::default().with_seed(random_seed());
        let core_state = CoreState::new(config);

        app.insert_resource(core_state)
            .init_resource::<CoreEntityMap>()
            .init_resource::<BoatAssets>()
            .init_resource::<CoreControl>();

        app.insert_resource(Time::<Fixed>::from_hz(60.0));

        app.add_systems(
            FixedUpdate,
            (tick_core_from_input, diff_and_emit_events, sync_ecs_from_core).chain(),
        );
    }
}

fn random_seed() -> [u8; 32] {
    // Only called at startup and on restart — presentation may use any entropy
    // source it wants. Core never calls `thread_rng`.
    let mut seed = [0u8; 32];
    thread_rng().fill(&mut seed);
    seed
}

fn build_input(keyboard: &ButtonInput<KeyCode>) -> FishGameInput {
    FishGameInput {
        move_left: keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA),
        move_right: keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD),
        move_up: keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW),
        move_down: keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS),
        boost_pressed: keyboard.pressed(KeyCode::Space),
        boost_just_pressed: keyboard.just_pressed(KeyCode::Space),
    }
}

fn tick_core_from_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut core: ResMut<CoreState>,
    mut control: ResMut<CoreControl>,
) {
    // Handle pause toggle first so it takes effect this tick.
    let keyboard_pause = keyboard.just_pressed(KeyCode::Escape);
    if keyboard_pause || control.pause_toggle_pending {
        control.pause_toggle_pending = false;
        // Paused games can only be unpaused if they're still in progress.
        if core.state.phase != GamePhase::GameOver {
            control.paused = !control.paused;
        }
    }

    // Handle restart.
    control.restart_fired = false;
    if keyboard.just_pressed(KeyCode::KeyR) {
        let config = core.state.config.clone().with_seed(random_seed());
        core.reset_with(config);
        control.paused = false;
        control.restart_fired = true;
        return;
    }

    if control.paused {
        // While paused we still need `prev == state` so diffs don't fire
        // stale events on unpause.
        core.prev = core.state.clone();
        return;
    }

    let input = build_input(&keyboard);
    core.prev = core.state.clone();
    core.state.tick(input);
}

fn diff_and_emit_events(
    core: Res<CoreState>,
    control: Res<CoreControl>,
    mut ev_boosted: EventWriter<PlayerBoosted>,
    mut ev_hooked: EventWriter<PlayerHooked>,
    mut ev_bonked: EventWriter<PlayerBonked>,
    mut ev_starved: EventWriter<PlayerStarved>,
    mut ev_ate: EventWriter<PlayerAte>,
    mut ev_game_over: EventWriter<GameOver>,
    mut ev_restart: EventWriter<GameRestarted>,
    mut ev_paused: EventWriter<GamePaused>,
    mut ev_unpaused: EventWriter<GameUnpaused>,
    mut prev_paused: Local<bool>,
) {
    use fish_game_core::player::PlayerState;

    let prev = &core.prev;
    let curr = &core.state;

    // Boost edge.
    if prev.player.state != PlayerState::Boost && curr.player.state == PlayerState::Boost {
        ev_boosted.send(PlayerBoosted {
            player: Entity::PLACEHOLDER,
        });
    }

    // Ate a worm this tick. `hunger_ticks_remaining` only ever decreases inside
    // a tick — if it went up, the player ate.
    if curr.player.hunger_ticks_remaining > prev.player.hunger_ticks_remaining {
        ev_ate.send(PlayerAte {
            player_entity: Entity::PLACEHOLDER,
            worm_entity: Entity::PLACEHOLDER,
        });
    }

    // GameOver edge.
    if prev.phase != GamePhase::GameOver && curr.phase == GamePhase::GameOver {
        match curr.game_over_cause {
            Some(GameOverCause::Hooked) => {
                ev_hooked.send(PlayerHooked {
                    player_entity: Entity::PLACEHOLDER,
                    hook_entity: Entity::PLACEHOLDER,
                });
            }
            Some(GameOverCause::Bonked) => {
                ev_bonked.send(PlayerBonked {
                    player_entity: Entity::PLACEHOLDER,
                    boat_entity: Entity::PLACEHOLDER,
                });
            }
            Some(GameOverCause::Starved) => {
                ev_starved.send(PlayerStarved {
                    player_entity: Entity::PLACEHOLDER,
                });
            }
            None => {}
        }
        ev_game_over.send(GameOver { winning_boat: None });
    }

    // Pause edge — fires based on the adapter's `paused` flag, not core phase.
    if control.paused && !*prev_paused {
        ev_paused.send(GamePaused);
    } else if !control.paused && *prev_paused {
        ev_unpaused.send(GameUnpaused);
    }
    *prev_paused = control.paused;

    if control.restart_fired {
        ev_restart.send(GameRestarted);
    }
}

/// Spawn/despawn Bevy entities so they mirror the core slotmaps, and update
/// their `Transform`s from core positions each tick. Also owns the rod/line
/// stroke re-draw.
fn sync_ecs_from_core(
    mut commands: Commands,
    core: Res<CoreState>,
    assets: Res<BoatAssets>,
    mut map: ResMut<CoreEntityMap>,
    mut transforms: Query<&mut Transform>,
) {
    let state = &core.state;

    // --- Boats ---
    for (bid, boat) in state.boats.iter() {
        if let Some(&entity) = map.boats.get(&bid) {
            if let Ok(mut tf) = transforms.get_mut(entity) {
                tf.translation = boat.pos;
                tf.rotation = if boat.facing_right {
                    Quat::IDENTITY
                } else {
                    Quat::from_rotation_y(std::f32::consts::PI)
                };
            }
        } else {
            let entity = commands
                .spawn((
                    SpriteBundle {
                        texture: assets.boat.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(boat.width, boat.height)),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: boat.pos,
                            rotation: if boat.facing_right {
                                Quat::IDENTITY
                            } else {
                                Quat::from_rotation_y(std::f32::consts::PI)
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    Collider {
                        width: boat.width,
                        height: boat.height,
                    },
                    RenderLayer::Objects,
                    BoatMarker(bid),
                ))
                .id();
            map.boats.insert(bid, entity);
        }
    }
    map.boats.retain(|bid, entity| {
        if state.boats.contains_key(*bid) {
            true
        } else {
            commands.entity(*entity).despawn_recursive();
            false
        }
    });

    // --- Hooks ---
    for (hid, hook) in state.hooks.iter() {
        if let Some(&entity) = map.hooks.get(&hid) {
            if let Ok(mut tf) = transforms.get_mut(entity) {
                tf.translation = hook.pos;
            }
        } else {
            let entity = commands
                .spawn((
                    SpriteBundle {
                        texture: assets.hook.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(hook.width, hook.height)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(hook.pos),
                        ..Default::default()
                    },
                    Collider {
                        width: hook.width,
                        height: hook.height,
                    },
                    RenderLayer::Objects,
                    HookMarker(hid),
                ))
                .id();
            map.hooks.insert(hid, entity);
        }
    }
    map.hooks.retain(|hid, entity| {
        if state.hooks.contains_key(*hid) {
            true
        } else {
            commands.entity(*entity).despawn_recursive();
            false
        }
    });

    // --- Worms ---
    for (wid, worm) in state.worms.iter() {
        if let Some(&entity) = map.worms.get(&wid) {
            if let Ok(mut tf) = transforms.get_mut(entity) {
                tf.translation = worm.pos;
            }
        } else {
            let entity = commands
                .spawn((
                    SpriteBundle {
                        texture: assets.worm_frame1.clone(),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(worm.width, worm.height)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(worm.pos),
                        ..Default::default()
                    },
                    Collider {
                        width: worm.width,
                        height: worm.height,
                    },
                    RenderLayer::Objects,
                    WormMarker(wid),
                ))
                .id();
            map.worms.insert(wid, entity);
        }
    }
    map.worms.retain(|wid, entity| {
        if state.worms.contains_key(*wid) {
            true
        } else {
            commands.entity(*entity).despawn_recursive();
            false
        }
    });

    // --- Lines ---
    // Re-draw every frame since the endpoints move; cheaper than diffing.
    for (lid, line) in state.lines.iter() {
        let mut builder = PathBuilder::new();
        builder.move_to(Vec2::new(line.start_pos.x, line.start_pos.y));
        builder.line_to(Vec2::new(line.end_pos.x, line.end_pos.y));
        let path = builder.build();

        let stroke = Stroke {
            color: assets.line_color,
            options: StrokeOptions::default()
                .with_line_width(1.0)
                .with_line_cap(LineCap::Round)
                .with_line_join(LineJoin::Round),
        };

        if let Some(&entity) = map.lines.get(&lid) {
            commands
                .entity(entity)
                .insert((ShapeBundle { path, ..default() }, stroke));
        } else {
            let entity = commands
                .spawn((
                    ShapeBundle { path, ..default() },
                    stroke,
                    LineMarker(lid),
                ))
                .id();
            map.lines.insert(lid, entity);
        }
    }
    map.lines.retain(|lid, entity| {
        if state.lines.contains_key(*lid) {
            true
        } else {
            commands.entity(*entity).despawn_recursive();
            false
        }
    });
}

#[derive(Component)]
pub struct BoatMarker(pub BoatId);
#[derive(Component)]
pub struct HookMarker(pub HookId);
#[derive(Component)]
pub struct LineMarker(pub LineId);
#[derive(Component)]
pub struct WormMarker(pub WormId);

/// Component tagging the player entity so presentation systems can find it
/// without knowing about core. The player is a singleton, but the adapter
/// owns spawning/respawning it just like boats/hooks/worms.
#[derive(Component)]
pub struct PlayerMarker;

/// Sync the player's `Transform` from the core state. Called in `Update`
/// after `FixedUpdate` so the render frame sees the latest tick's position.
pub fn sync_player_transform(
    core: Res<CoreState>,
    mut q: Query<&mut Transform, With<PlayerMarker>>,
) {
    let Ok(mut tf) = q.get_single_mut() else { return; };
    tf.translation = core.state.player.pos;
}
