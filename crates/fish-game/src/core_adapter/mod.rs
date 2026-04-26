//! Bridge between `fish-game-core` (the deterministic simulation) and Bevy.
//!
//! The core owns all simulation state. The adapter:
//!
//!   1. Builds a [`FishGameInput`] from current keyboard state each fixed tick.
//!   2. Calls `state.tick(input)`.
//!   3. Forwards every [`CoreEvent`] in `state.events` to its Bevy equivalent
//!      (`PlayerHooked`, `PlayerAte`, `GameOver`, …). No state diffing — the
//!      core tells us exactly what happened.
//!   4. Mirrors core entities (boats, hooks, worms) into Bevy entities via a
//!      `CoreEntityMap`, driven by `*Spawned`/`*Despawned` events.
//!
//! Lifecycle (pause / restart) lives here, NOT in core:
//!   - Pause ⇒ `CoreControl.paused` flips; `tick_core_from_input` returns
//!     early so core sees no ticks while paused.
//!   - Restart ⇒ rebuild `CoreState` with a freshly-derived seed. The old
//!     simulation ends cleanly; the new one starts at tick 0.

pub mod game_over;

use bevy::prelude::*;
use fish_game_core::boat::{BoatId, HookId, LineId, WormId, HOOK_SIZE};
use fish_game_core::{
    CoreEvent, FishGameConfig, FishGameInput, FishGameState, GameOverCause, GamePhase,
};
use rand::{thread_rng, Rng};
use std::collections::BTreeMap;

use crate::player::events::{PlayerAte, PlayerBonked, PlayerBoosted, PlayerHooked, PlayerStarved};
use crate::shared::collision::Collider;
use crate::shared::game::{GameOver, GamePaused, GameRestarted, GameUnpaused};
use crate::shared::render::RenderLayer;
use crate::shared::stages;

use game_over::GameOverWindDownPlugin;

/// Resource wrapping the deterministic simulation — the single source of
/// truth. Presentation reads `state` and forwards `state.events` after each
/// tick; no prev-snapshot is needed because the core tells us exactly what
/// transitioned.
#[derive(Resource)]
pub struct CoreState {
    pub state: FishGameState,
}

impl CoreState {
    pub fn new(config: FishGameConfig) -> Self {
        Self {
            state: FishGameState::new(config),
        }
    }

    /// Replace the current sim with a fresh one (same tuning, new seed).
    /// Used by restart.
    pub fn reset_with(&mut self, config: FishGameConfig) {
        self.state = FishGameState::new(config);
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

/// Resource holding the texture handles for boat / hook / worm rendering
/// plus the cached mesh + material used to draw the fishing line. Built from
/// `AssetServer` at startup.
#[derive(Resource)]
pub struct BoatAssets {
    pub boat: Handle<Image>,
    pub hook: Handle<Image>,
    pub worm_frame1: Handle<Image>,
    pub worm_frame2: Handle<Image>,
    pub line_mesh: Handle<Mesh>,
    pub line_material: Handle<ColorMaterial>,
}

const LINE_THICKNESS: f32 = 1.0;

impl FromWorld for BoatAssets {
    fn from_world(world: &mut World) -> Self {
        let boat: Handle<Image>;
        let hook: Handle<Image>;
        let worm_frame1: Handle<Image>;
        let worm_frame2: Handle<Image>;
        {
            let asset_server = world.get_resource::<AssetServer>().unwrap();
            boat = asset_server.load("sprites/boat/boat.png");
            hook = asset_server.load("sprites/hook/hook.png");
            worm_frame1 = asset_server.load("sprites/worm/worm1.png");
            worm_frame2 = asset_server.load("sprites/worm/worm2.png");
        }
        let line_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Rectangle::new(1.0, 1.0));
        let line_material = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::from(Color::BLACK));
        Self {
            boat,
            hook,
            worm_frame1,
            worm_frame2,
            line_mesh,
            line_material,
        }
    }
}

/// Endpoints that anchor a fishing-line entity's transform. Populated once at
/// spawn; read every frame by `sync_line_transforms_from_endpoints` to compute
/// the line's midpoint, angle, and length from the current Bevy `Transform`s
/// of the boat and hook. This works both during Running (transforms come from
/// `sync_ecs_from_core`) and during GameOver (transforms come from
/// `drift_boats_off_arena` / `advance_reeling_hooks`).
#[derive(Component)]
pub struct LineEndpoints {
    /// The boat entity whose translation + `rod_tip_offset` gives the line start.
    pub boat_entity: Entity,
    /// The hook entity whose translation (+ HOOK_SIZE/2 in Y) gives the line end.
    pub hook_entity: Entity,
    /// Fixed world-space offset from the boat's translation to the rod tip,
    /// computed once at spawn when `line.start_pos - boat.pos` is stable.
    pub rod_tip_offset: Vec3,
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
            (
                tick_core_from_input,
                forward_core_events,
                sync_ecs_from_core,
            )
                .chain(),
        );

        // Line transforms are derived from their boat + hook Transforms, which
        // are updated either by sync_ecs_from_core (Running) or by the
        // game-over wind-down systems (GameOver). Must run late in Update so
        // both sources have already written their transforms this frame.
        app.add_systems(
            Update,
            sync_line_transforms_from_endpoints.in_set(stages::PrepareRenderSet),
        );

        app.add_plugins(GameOverWindDownPlugin);
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
        // Drop any lingering events so presentation doesn't replay them on
        // unpause.
        core.state.events.clear();
        return;
    }

    let input = build_input(&keyboard);
    core.state.tick(input);
}

/// Forward each [`CoreEvent`] produced during the last `tick` to its Bevy
/// `Event` equivalent. The core is authoritative about what happened; this
/// system is a straight translation table with no diffing.
fn forward_core_events(
    core: Res<CoreState>,
    control: Res<CoreControl>,
    map: Res<CoreEntityMap>,
    mut ev_boosted: MessageWriter<PlayerBoosted>,
    mut ev_hooked: MessageWriter<PlayerHooked>,
    mut ev_bonked: MessageWriter<PlayerBonked>,
    mut ev_starved: MessageWriter<PlayerStarved>,
    mut ev_ate: MessageWriter<PlayerAte>,
    mut ev_game_over: MessageWriter<GameOver>,
    mut ev_restart: MessageWriter<GameRestarted>,
    mut ev_paused: MessageWriter<GamePaused>,
    mut ev_unpaused: MessageWriter<GameUnpaused>,
    mut prev_paused: Local<bool>,
) {
    for event in &core.state.events {
        match event {
            CoreEvent::PlayerBoosted => {
                ev_boosted.write(PlayerBoosted {
                    player: Entity::PLACEHOLDER,
                });
            }
            CoreEvent::PlayerAte { worm } => {
                let worm_entity = map.worms.get(worm).copied().unwrap_or(Entity::PLACEHOLDER);
                ev_ate.write(PlayerAte {
                    player_entity: Entity::PLACEHOLDER,
                    worm_entity,
                });
            }
            CoreEvent::PlayerHooked { hook, .. } => {
                let hook_entity = map.hooks.get(hook).copied().unwrap_or(Entity::PLACEHOLDER);
                ev_hooked.write(PlayerHooked {
                    player_entity: Entity::PLACEHOLDER,
                    hook_entity,
                });
            }
            CoreEvent::PlayerBonked { boat } => {
                let boat_entity = map.boats.get(boat).copied().unwrap_or(Entity::PLACEHOLDER);
                ev_bonked.write(PlayerBonked {
                    player_entity: Entity::PLACEHOLDER,
                    boat_entity,
                });
            }
            CoreEvent::PlayerStarved => {
                ev_starved.write(PlayerStarved {
                    player_entity: Entity::PLACEHOLDER,
                });
            }
            CoreEvent::GameOver { .. } => {
                // Resolve the winning boat entity from the core state.
                let winning_boat = core
                    .state
                    .game_over_boat
                    .and_then(|bid| map.boats.get(&bid).copied());
                ev_game_over.write(GameOver { winning_boat });
            }
            // Entity-lifecycle + score/difficulty events are consumed by
            // `sync_ecs_from_core` and UI code that reads `CoreState`
            // directly. No Bevy Event mapping needed — suppress explicitly
            // so a future added variant triggers a compile error.
            CoreEvent::ScoreIncremented { .. }
            | CoreEvent::DifficultyIncreased { .. }
            | CoreEvent::BoatSpawned(_)
            | CoreEvent::HookSpawned(_)
            | CoreEvent::LineSpawned(_)
            | CoreEvent::WormSpawned(_)
            | CoreEvent::WormDespawned(_) => {}
        }
    }

    // Pause / restart stay adapter-local — core has no concept of them.
    if control.paused && !*prev_paused {
        ev_paused.write(GamePaused);
    } else if !control.paused && *prev_paused {
        ev_unpaused.write(GameUnpaused);
    }
    *prev_paused = control.paused;

    if control.restart_fired {
        ev_restart.write(GameRestarted);
    }
}

/// Spawn/despawn Bevy entities so they mirror the core slotmaps, and update
/// their `Transform`s from core positions each tick.
///
/// Returns early once the core enters `GamePhase::GameOver` — at that point
/// the game-over wind-down module (`game_over.rs`) owns all `Transform`
/// updates and entity despawning for boats, hooks, lines, and worms.
fn sync_ecs_from_core(
    mut commands: Commands,
    core: Res<CoreState>,
    assets: Res<BoatAssets>,
    mut map: ResMut<CoreEntityMap>,
    mut transforms: Query<&mut Transform>,
) {
    let state = &core.state;

    if state.phase == GamePhase::GameOver {
        return;
    }

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
                    Sprite {
                        image: assets.boat.clone(),
                        custom_size: Some(Vec2::new(boat.width, boat.height)),
                        ..Default::default()
                    },
                    Transform {
                        translation: boat.pos,
                        rotation: if boat.facing_right {
                            Quat::IDENTITY
                        } else {
                            Quat::from_rotation_y(std::f32::consts::PI)
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
            commands.entity(*entity).despawn();
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
                    Sprite {
                        image: assets.hook.clone(),
                        custom_size: Some(Vec2::new(hook.width, hook.height)),
                        ..Default::default()
                    },
                    Transform::from_translation(hook.pos),
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
            commands.entity(*entity).despawn();
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
                    Sprite {
                        image: assets.worm_frame1.clone(),
                        custom_size: Some(Vec2::new(worm.width, worm.height)),
                        ..Default::default()
                    },
                    Transform::from_translation(worm.pos),
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
            commands.entity(*entity).despawn();
            false
        }
    });

    // --- Lines ---
    // Each line entity stores `LineEndpoints` so `sync_line_transforms_from_endpoints`
    // can recompute its transform from the current boat and hook Transforms —
    // this works both during Running and during the game-over wind-down.
    for (lid, line) in state.lines.iter() {
        if map.lines.contains_key(&lid) {
            // Already exists — transform updated by sync_line_transforms_from_endpoints.
            continue;
        }

        // Compute the stable rod-tip offset from the boat center.
        let rod_tip_offset = if let Some(boat) = state.boats.get(line.boat_id) {
            line.start_pos - boat.pos
        } else {
            Vec3::ZERO
        };

        // Find the hook entity that corresponds to this line.
        let hook_entity = state
            .hooks
            .iter()
            .find(|(_, h)| h.line_id == lid)
            .and_then(|(hid, _)| map.hooks.get(&hid).copied())
            .unwrap_or(Entity::PLACEHOLDER);

        let boat_entity = map
            .boats
            .get(&line.boat_id)
            .copied()
            .unwrap_or(Entity::PLACEHOLDER);

        // Compute an initial transform so the line is visible on the first frame.
        let delta = line.end_pos.truncate() - line.start_pos.truncate();
        let length = delta.length();
        let midpoint = (line.start_pos + line.end_pos) * 0.5;
        let initial_transform = Transform {
            translation: midpoint,
            rotation: Quat::from_rotation_z(delta.to_angle()),
            scale: Vec3::new(length, LINE_THICKNESS, 1.0),
        };

        let entity = commands
            .spawn((
                Mesh2d(assets.line_mesh.clone()),
                MeshMaterial2d(assets.line_material.clone()),
                initial_transform,
                LineMarker(lid),
                LineEndpoints {
                    boat_entity,
                    hook_entity,
                    rod_tip_offset,
                },
            ))
            .id();
        map.lines.insert(lid, entity);
    }
    map.lines.retain(|lid, entity| {
        if state.lines.contains_key(*lid) {
            true
        } else {
            commands.entity(*entity).despawn();
            false
        }
    });
}

/// Recomputes each fishing-line's `Transform` from the current Bevy `Transform`s
/// of its anchoring boat and hook. Runs in `PrepareRenderSet` so it sees
/// transforms that were updated earlier this frame — whether from
/// `sync_ecs_from_core` (Running) or from the game-over wind-down systems.
fn sync_line_transforms_from_endpoints(
    mut lines: Query<(&LineEndpoints, &mut Transform)>,
    entities: Query<&Transform, Without<LineEndpoints>>,
) {
    for (endpoints, mut line_tf) in &mut lines {
        let Ok(boat_tf) = entities.get(endpoints.boat_entity) else {
            continue;
        };
        let Ok(hook_tf) = entities.get(endpoints.hook_entity) else {
            continue;
        };

        let start = boat_tf.translation + endpoints.rod_tip_offset;
        let end = hook_tf.translation + Vec3::new(0.0, HOOK_SIZE / 2.0, 0.0);

        let delta = (end - start).truncate();
        let length = delta.length();
        let midpoint = (start + end) * 0.5;
        *line_tf = Transform {
            translation: midpoint,
            rotation: Quat::from_rotation_z(delta.to_angle()),
            scale: Vec3::new(length, LINE_THICKNESS, 1.0),
        };
    }
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
///
/// Rotation is fully derived from the core: during Running, it tracks the
/// player's facing direction; at game over, if the player starved, we flip
/// upside-down as a cosmetic "belly-up" cue (the core freezes the position).
pub fn sync_player_transform(
    core: Res<CoreState>,
    mut q: Query<&mut Transform, With<PlayerMarker>>,
) {
    let Ok(mut tf) = q.single_mut() else {
        return;
    };
    tf.translation = core.state.player.pos;

    tf.rotation = if core.state.game_over_cause == Some(GameOverCause::Starved) {
        Quat::from_rotation_x(std::f32::consts::PI)
    } else if core.state.player.facing_right {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_y(std::f32::consts::PI)
    };
}
