//! Presentation-layer wind-down after `GamePhase::GameOver`.
//!
//! Once the core declares game over and freezes, this module takes over:
//!   - Non-winning boats reverse at 2× speed and slide off the arena.
//!   - The winning boat stops; its hook reels back to the rod tip at 300 u/s.
//!   - Boats are despawned once fully off-screen.
//!
//! All math is driven by Bevy `Transform`s, not the frozen core slotmaps.
//! `reel::ReelMotion` / `reel::advance` from `fish-game-core` provide the
//! hook movement primitive so it can be reused for future mid-game moving-hook
//! features without re-deriving it.

use bevy::prelude::*;
use fish_game_core::reel::{advance as reel_advance, ReelMotion};

use crate::core_adapter::{CoreEntityMap, CoreState};
use crate::shared::arena::Arena;
use crate::shared::game::{GameOver, GameRestarted};
use crate::shared::stages;

const BOAT_EXIT_SPEED_MULT: f32 = 2.0;
const REEL_SPEED: f32 = 300.0;

/// Velocity component applied to boats during the game-over wind-down.
/// Non-winning boats get a reversed, doubled exit velocity; the winner gets
/// `Vec3::ZERO` (it freezes in place while its hook reels in).
#[derive(Component)]
pub struct BoatVelocity(pub Vec3);

/// Marks the hook that is reeling back to the rod tip after game over.
#[derive(Component)]
pub struct ReelingHook {
    pub motion: ReelMotion,
}

/// Tracks the Bevy child entities that belong to a boat so the despawn
/// system can clean them up when the boat leaves the arena.
#[derive(Component)]
pub struct BoatChildren {
    pub hooks: Vec<Entity>,
    pub worms: Vec<Entity>,
    pub lines: Vec<Entity>,
}

pub struct GameOverWindDownPlugin;

impl Plugin for GameOverWindDownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            snapshot_game_over_state.in_set(stages::HandleEventsSet),
        )
        .add_systems(
            Update,
            (drift_boats_off_arena, advance_reeling_hooks).in_set(stages::MovementSet),
        )
        .add_systems(
            Update,
            (
                despawn_offscreen_boats_bevy,
                cleanup_wind_down_on_restart,
            )
                .in_set(stages::PrepareRenderSet),
        );
    }
}

/// One-shot system that fires the frame `GameOver` is received.
///
/// Reads the frozen core state to snapshot each boat's last known velocity,
/// then adds `BoatVelocity` / `BoatChildren` to every boat entity and
/// `ReelingHook` to every hook on the winning boat. After this, `sync_ecs_from_core`
/// returns early so Bevy transforms are owned entirely by this module.
pub fn snapshot_game_over_state(
    mut commands: Commands,
    mut reader: MessageReader<GameOver>,
    core: Res<CoreState>,
    mut map: ResMut<CoreEntityMap>,
    mut transforms: Query<&mut Transform>,
) {
    if reader.read().next().is_none() {
        return;
    }

    let state = &core.state;
    let winning_boat_id = state.game_over_boat;

    // Worms were removed from state.worms by enter_game_over, but
    // sync_ecs_from_core returns early in GameOver mode and never runs the
    // retain/despawn pass. Explicitly clean up lingering worm entities here.
    for &worm_entity in map.worms.values() {
        commands.entity(worm_entity).despawn();
    }
    map.worms.clear();

    for (bid, boat) in state.boats.iter() {
        let Some(&boat_entity) = map.boats.get(&bid) else {
            continue;
        };

        // Sync the final core position into the Bevy transform so the
        // wind-down starts from the correct position (sync_ecs_from_core
        // already returned early this tick since phase == GameOver).
        if let Ok(mut tf) = transforms.get_mut(boat_entity) {
            tf.translation = boat.pos;
            tf.rotation = if boat.facing_right {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_y(std::f32::consts::PI)
            };
        }

        // Collect children for later bulk despawn.
        let hook_entities: Vec<Entity> = boat
            .hook_ids
            .iter()
            .filter_map(|hid| map.hooks.get(hid).copied())
            .collect();
        let worm_entities: Vec<Entity> = boat
            .worm_ids
            .iter()
            .filter_map(|wid| map.worms.get(wid).copied())
            .collect();
        let line_entities: Vec<Entity> = boat
            .line_ids
            .iter()
            .filter_map(|lid| map.lines.get(lid).copied())
            .collect();

        // Sync hook positions from core.
        for &hid in &boat.hook_ids {
            if let Some(hook) = state.hooks.get(hid) {
                if let Some(&hook_entity) = map.hooks.get(&hid) {
                    if let Ok(mut tf) = transforms.get_mut(hook_entity) {
                        tf.translation = hook.pos;
                    }
                }
            }
        }

        commands.entity(boat_entity).insert(BoatChildren {
            hooks: hook_entities.clone(),
            worms: worm_entities,
            lines: line_entities,
        });

        if Some(bid) == winning_boat_id {
            // Winner: freeze in place, reel every hook back to its rod tip.
            commands.entity(boat_entity).insert(BoatVelocity(Vec3::ZERO));

            for &hid in &boat.hook_ids {
                let Some(hook) = state.hooks.get(hid) else {
                    continue;
                };
                let Some(line) = state.lines.get(hook.line_id) else {
                    continue;
                };
                let Some(&hook_entity) = map.hooks.get(&hid) else {
                    continue;
                };
                let motion = ReelMotion::toward(hook.pos, line.start_pos, REEL_SPEED);
                commands.entity(hook_entity).insert(ReelingHook { motion });
            }
        } else {
            // Non-winner: reverse and double speed, sliding off the arena.
            let speed_abs = boat.velocity.x.abs().max(1.0) * BOAT_EXIT_SPEED_MULT;
            let exit_x = if boat.pos.x < 0.0 {
                -speed_abs
            } else {
                speed_abs
            };
            commands
                .entity(boat_entity)
                .insert(BoatVelocity(Vec3::new(exit_x, 0.0, 0.0)));
        }
    }
}

/// Translates boats with `BoatVelocity` (and their child hooks/worms) each
/// frame. Lines are updated by `sync_line_transforms_from_endpoints`.
pub fn drift_boats_off_arena(
    time: Res<Time>,
    mut boats: Query<(&BoatVelocity, &BoatChildren, &mut Transform)>,
    // Exclude BoatVelocity (would alias the boats query) and ReelingHook
    // (those hooks move themselves in advance_reeling_hooks).
    mut children: Query<&mut Transform, (Without<BoatVelocity>, Without<ReelingHook>)>,
) {
    for (vel, boat_children, mut boat_tf) in &mut boats {
        let dx = vel.0 * time.delta_secs();
        boat_tf.translation += dx;

        for &child_entity in boat_children
            .hooks
            .iter()
            .chain(boat_children.worms.iter())
        {
            // Skip hooks that are currently reeling (they move themselves).
            if let Ok(mut child_tf) = children.get_mut(child_entity) {
                child_tf.translation += dx;
            }
        }
    }
}

/// Advances `ReelingHook` entities via `reel::advance`. On arrival, removes
/// the component so the hook freezes at the rod tip.
pub fn advance_reeling_hooks(
    time: Res<Time>,
    mut commands: Commands,
    mut hooks: Query<(Entity, &mut ReelingHook, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut reel, mut tf) in &mut hooks {
        let (new_pos, arrived) = reel_advance(tf.translation, reel.motion, dt);
        tf.translation = new_pos;
        if arrived {
            commands.entity(entity).remove::<ReelingHook>();
        }
    }
}

/// Despawns boats (and their `BoatChildren`) once they travel off-screen.
pub fn despawn_offscreen_boats_bevy(
    mut commands: Commands,
    boats: Query<(Entity, &Transform, &BoatChildren), With<BoatVelocity>>,
    arena: Res<Arena>,
) {
    // Use a generous margin so the full sprite clears the screen edge.
    let half_w = arena.width / 2.0 + 200.0;
    for (entity, tf, children) in &boats {
        let x = tf.translation.x;
        if x < -half_w || x > half_w {
            commands.entity(entity).despawn();
            for &child in children
                .hooks
                .iter()
                .chain(children.worms.iter())
                .chain(children.lines.iter())
            {
                commands.entity(child).despawn();
            }
        }
    }
}

/// Removes `BoatVelocity` / `ReelingHook` / `BoatChildren` from any surviving
/// entities when the game is restarted mid-wind-down. The entities themselves
/// are despawned by `sync_ecs_from_core`'s retain logic on the restart tick.
pub fn cleanup_wind_down_on_restart(
    mut commands: Commands,
    mut reader: MessageReader<GameRestarted>,
    boats: Query<Entity, With<BoatVelocity>>,
    reeling: Query<Entity, With<ReelingHook>>,
) {
    if reader.read().next().is_none() {
        return;
    }
    for entity in &boats {
        commands
            .entity(entity)
            .remove::<BoatVelocity>()
            .remove::<BoatChildren>();
    }
    for entity in &reeling {
        commands.entity(entity).remove::<ReelingHook>();
    }
}

