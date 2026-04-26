use glam::Vec3;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use crate::config::{ArenaConfig, FishGameConfig};
use crate::math;

new_key_type! {
    pub struct BoatId;
    pub struct HookId;
    pub struct LineId;
    pub struct WormId;
}

pub const ROD_LENGTH: f32 = 5.0;
pub const POLE_HEIGHT: f32 = 10.0;
pub const HOOK_SIZE: f32 = 16.0;
pub const WORM_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoatType {
    Dinghy,
    Fishingboat,
    Speedboat,
    Yacht,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoatStats {
    pub boat_type: BoatType,
    pub num_poles: u8,
    pub speed: f32,
    pub width: f32,
    pub height: f32,
    pub worm_chance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boat {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub facing_right: bool,
    pub width: f32,
    pub height: f32,
    pub boat_type: BoatType,
    pub hook_ids: Vec<HookId>,
    pub line_ids: Vec<LineId>,
    pub worm_ids: Vec<WormId>,
    /// Set to true after GameOver: boat is leaving the scene, no more reel-in.
    pub exiting: bool,
    /// If true, this boat "caught the fish" — stays on screen after GameOver.
    pub winner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub pos: Vec3,
    pub boat_id: BoatId,
    pub line_id: LineId,
    pub width: f32,
    pub height: f32,
    /// During reel-in, the hook moves toward `reel_destination` at
    /// `reel_velocity`. Both None during normal play (hook drifts with boat).
    pub reel_velocity: Option<Vec3>,
    pub reel_destination: Option<Vec3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub boat_id: BoatId,
    /// World position of the tip of the fishing rod. Moves with the boat.
    pub start_pos: Vec3,
    /// World position of where the line meets the hook. Follows hook motion.
    pub end_pos: Vec3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worm {
    pub pos: Vec3,
    pub boat_id: BoatId,
    pub line_id: LineId,
    pub width: f32,
    pub height: f32,
}

/// Random boat stats keyed on current difficulty — ported verbatim from
/// `objects/boat.rs::boat_stats_factory`.
pub fn roll_boat_stats(difficulty: u8, rng: &mut ChaCha8Rng) -> BoatStats {
    let kind = match rng.gen_range(1..difficulty + 1) {
        1 => BoatType::Dinghy,
        2 => BoatType::Fishingboat,
        3 => BoatType::Speedboat,
        4 => BoatType::Yacht,
        _ => BoatType::Dinghy,
    };

    match kind {
        BoatType::Dinghy => BoatStats {
            boat_type: kind,
            num_poles: 1,
            speed: (rng.gen_range(30..40) + (5 * difficulty)) as f32,
            width: 45.0,
            height: 10.0,
            worm_chance: 0.5,
        },
        BoatType::Fishingboat => BoatStats {
            boat_type: kind,
            num_poles: rng.gen_range(1..3) + difficulty,
            speed: (rng.gen_range(40..50) + (5 * difficulty)) as f32,
            width: 65.0,
            height: 24.0,
            worm_chance: 0.8,
        },
        BoatType::Speedboat => BoatStats {
            boat_type: kind,
            num_poles: rng.gen_range(1..2) + difficulty,
            speed: (rng.gen_range(75..100) + (5 * difficulty)) as f32,
            width: 75.0,
            height: 16.0,
            worm_chance: 0.4,
        },
        BoatType::Yacht => BoatStats {
            boat_type: kind,
            num_poles: rng.gen_range(3..6) + difficulty,
            speed: (rng.gen_range(60..75) + (5 * difficulty)) as f32,
            width: 128.0,
            height: 64.0,
            worm_chance: 0.25,
        },
    }
}

/// Outcome of a boat spawn — the new boat plus every child entity created
/// alongside it. Returned so the caller can emit `*Spawned` events without
/// diffing slotmap membership.
pub struct SpawnedBoat {
    pub boat_id: BoatId,
    pub hook_ids: Vec<HookId>,
    pub line_ids: Vec<LineId>,
    pub worm_ids: Vec<WormId>,
}

/// Outcome of a boat despawn — identifiers of everything removed so the
/// caller can emit `*Despawned` events.
pub struct DespawnedBoat {
    pub boat_id: BoatId,
    pub hook_ids: Vec<HookId>,
    pub line_ids: Vec<LineId>,
    pub worm_ids: Vec<WormId>,
}

/// Spawn a boat and all its children (rods, lines, hooks, worms) into the
/// respective slotmaps. RNG draws here must match the original
/// `spawn_boat`/`spawn_lines` order for replay compatibility with saved games
/// built against the old Bevy version; new replays just need to be self-
/// consistent across targets.
pub fn spawn_boat(
    stats: BoatStats,
    arena: &ArenaConfig,
    rng: &mut ChaCha8Rng,
    boats: &mut SlotMap<BoatId, Boat>,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &mut SlotMap<LineId, Line>,
    worms: &mut SlotMap<WormId, Worm>,
) -> SpawnedBoat {
    let facing_right: bool = rng.gen();
    let boat_start_pos = Vec3::new(
        if facing_right {
            -(arena.width / 2.0) - stats.width + 1.0
        } else {
            (arena.width / 2.0) + stats.width - 1.0
        },
        (arena.height / 2.0) + arena.offset,
        0.0,
    );

    let velocity = Vec3::new(
        if facing_right { stats.speed } else { -stats.speed },
        0.0,
        0.0,
    );

    let boat_id = boats.insert(Boat {
        pos: boat_start_pos,
        velocity,
        facing_right,
        width: stats.width,
        height: stats.height,
        boat_type: stats.boat_type,
        hook_ids: Vec::new(),
        line_ids: Vec::new(),
        worm_ids: Vec::new(),
        exiting: false,
        winner: false,
    });

    let mut new_hook_ids = Vec::new();
    let mut new_line_ids = Vec::new();
    let mut new_worm_ids = Vec::new();

    for i in 1..stats.num_poles + 1 {
        // Rod position along the boat, in boat-local space but translated to world
        // up-front since the boat's x is known.
        let rod_offset = i as f32 * (stats.width / (stats.num_poles + 1) as f32);
        let rod_start_local = Vec3::new(
            -(stats.width / 2.0) + rod_offset,
            stats.height / 2.0,
            0.0,
        );
        let rod_angle_local = Vec3::new(rod_start_local.x, rod_start_local.y + POLE_HEIGHT, 0.0);
        let line_start_local =
            Vec3::new(rod_angle_local.x - ROD_LENGTH, rod_angle_local.y, 0.0);

        let line_length = rng.gen_range(50 + stats.height as u32..325) as f32;
        let line_angle = rng.gen_range(225..271) as f32;
        let angle_rad = core::f32::consts::PI * (line_angle / 180.0);
        let line_end_local = Vec3::new(
            line_start_local.x + line_length * math::cosf(angle_rad),
            line_start_local.y + line_length * math::sinf(angle_rad),
            0.0,
        );

        let line_mid_local = Vec3::new(
            (line_start_local.x + line_end_local.x) / 2.0,
            (line_start_local.y + line_end_local.y) / 2.0,
            0.0,
        );

        let spawn_worm = rng.gen_bool(stats.worm_chance as f64);
        let worm_distance_from_mid = if spawn_worm {
            rng.gen_range(0..(line_length / 2.0) as u32) as f32
        } else {
            0.0
        };
        let worm_anim_speed = rng.gen::<f32>() * 2.0;

        // Convert to world. Boat rotation around Y flips the sign of x in the
        // children; preserve that.
        let line_start_world = apply_boat_facing(boat_start_pos, line_start_local, facing_right);
        let line_end_world = apply_boat_facing(boat_start_pos, line_end_local, facing_right);
        let line_mid_world = apply_boat_facing(boat_start_pos, line_mid_local, facing_right);

        let line_id = lines.insert(Line {
            boat_id,
            start_pos: line_start_world,
            end_pos: line_end_world,
        });

        let mut hook_pos = line_end_world;
        hook_pos.y -= HOOK_SIZE / 2.0;
        let hook_id = hooks.insert(Hook {
            pos: hook_pos,
            boat_id,
            line_id,
            width: HOOK_SIZE,
            height: HOOK_SIZE,
            reel_velocity: None,
            reel_destination: None,
        });

        new_hook_ids.push(hook_id);
        new_line_ids.push(line_id);

        if spawn_worm {
            let to_end_dir = math::vec3_normalize_or_zero(line_mid_world - line_end_world);
            let worm_pos = line_mid_world - to_end_dir * worm_distance_from_mid;
            let worm_id = worms.insert(Worm {
                pos: worm_pos,
                boat_id,
                line_id,
                width: WORM_SIZE,
                height: WORM_SIZE,
            });
            new_worm_ids.push(worm_id);
        }

        let _ = worm_anim_speed; // consumed for RNG parity; animation lives in presentation
    }

    let boat = boats.get_mut(boat_id).unwrap();
    boat.hook_ids = new_hook_ids.clone();
    boat.line_ids = new_line_ids.clone();
    boat.worm_ids = new_worm_ids.clone();

    SpawnedBoat {
        boat_id,
        hook_ids: new_hook_ids,
        line_ids: new_line_ids,
        worm_ids: new_worm_ids,
    }
}

/// Apply the boat's Y-rotation-around-pi to a local child offset when building
/// its world position. Facing right = identity; facing left = flip x.
#[inline]
fn apply_boat_facing(boat_pos: Vec3, local: Vec3, facing_right: bool) -> Vec3 {
    let flipped_x = if facing_right { local.x } else { -local.x };
    Vec3::new(boat_pos.x + flipped_x, boat_pos.y + local.y, boat_pos.z + local.z)
}

/// Move boats and their attached entities by `dt`. Boats that reach off-screen
/// are collected (caller despawns them along with their children).
pub fn step_boats(
    dt: f32,
    boats: &mut SlotMap<BoatId, Boat>,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &mut SlotMap<LineId, Line>,
    worms: &mut SlotMap<WormId, Worm>,
) {
    for (_, boat) in boats.iter_mut() {
        let dx = boat.velocity * dt;
        boat.pos += dx;

        for &hook_id in &boat.hook_ids {
            if let Some(hook) = hooks.get_mut(hook_id) {
                if hook.reel_velocity.is_none() {
                    hook.pos += dx;
                }
            }
        }
        for &worm_id in &boat.worm_ids {
            if let Some(worm) = worms.get_mut(worm_id) {
                worm.pos += dx;
            }
        }
        for &line_id in &boat.line_ids {
            if let Some(line) = lines.get_mut(line_id) {
                line.start_pos += dx;
                // end_pos is refreshed from the hook pos below.
            }
        }
    }
}

/// Move hooks that are in reel-in mode. Stops once they reach their destination.
pub fn step_reeling_hooks(
    dt: f32,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &mut SlotMap<LineId, Line>,
) {
    for (_, hook) in hooks.iter_mut() {
        if let (Some(vel), Some(dest)) = (hook.reel_velocity, hook.reel_destination) {
            hook.pos += vel * dt;

            if math::vec3_length(dest - hook.pos) < 10.0 {
                hook.pos = dest;
                hook.reel_velocity = None;
                hook.reel_destination = None;
            }
        }

        // Keep the line's end synced with the hook (line attaches to top of hook).
        if let Some(line) = lines.get_mut(hook.line_id) {
            line.end_pos.x = hook.pos.x;
            line.end_pos.y = hook.pos.y + HOOK_SIZE / 2.0;
            line.end_pos.z = hook.pos.z;
        }
    }
}

/// Despawn boats that have travelled fully off-screen. A boat is considered
/// off-screen when every one of its hooks is off-screen; iterating hooks by
/// the boat's own `hook_ids` list (not by filtering the full slotmap) keeps
/// this deterministic without any `HashMap`.
pub fn despawn_offscreen_boats(
    arena: &ArenaConfig,
    boats: &mut SlotMap<BoatId, Boat>,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &mut SlotMap<LineId, Line>,
    worms: &mut SlotMap<WormId, Worm>,
) -> Vec<DespawnedBoat> {
    let arena_half_width = arena.width / 2.0;
    let mut boats_to_despawn: Vec<BoatId> = Vec::new();

    for (boat_id, boat) in boats.iter() {
        let boat_x = boat.pos.x;
        let boat_off_screen = (boat_x + boat.width) < -arena_half_width
            || (boat_x - boat.width) > arena_half_width;
        if !boat_off_screen {
            continue;
        }

        // All hooks must also be off-screen, matching the original behavior
        // where the boat holds its children until they also pass off-screen.
        if boat.hook_ids.is_empty() {
            boats_to_despawn.push(boat_id);
            continue;
        }
        let all_hooks_off = boat.hook_ids.iter().all(|hid| {
            if let Some(h) = hooks.get(*hid) {
                let hx = h.pos.x;
                (hx + h.width) < -arena_half_width || (hx - h.width) > arena_half_width
            } else {
                true
            }
        });
        if all_hooks_off {
            boats_to_despawn.push(boat_id);
        }
    }

    let mut despawned = Vec::with_capacity(boats_to_despawn.len());
    for boat_id in boats_to_despawn {
        if let Some(boat) = boats.remove(boat_id) {
            for &hid in &boat.hook_ids {
                hooks.remove(hid);
            }
            for &lid in &boat.line_ids {
                lines.remove(lid);
            }
            for &wid in &boat.worm_ids {
                worms.remove(wid);
            }
            despawned.push(DespawnedBoat {
                boat_id,
                hook_ids: boat.hook_ids,
                line_ids: boat.line_ids,
                worm_ids: boat.worm_ids,
            });
        }
    }
    despawned
}

/// On GameOver: turn every non-winning boat around and double its speed. The
/// winner boat (the one holding the hooked fish) stops moving.
pub fn trigger_boat_exit(winning_boat: Option<BoatId>, boats: &mut SlotMap<BoatId, Boat>) {
    for (id, boat) in boats.iter_mut() {
        boat.exiting = true;
        if Some(id) == winning_boat {
            boat.winner = true;
            boat.velocity = Vec3::ZERO;
            continue;
        }

        let speed_abs = boat.velocity.x.abs().max(1.0) * 2.0;
        if boat.pos.x < 0.0 {
            boat.facing_right = false;
            boat.velocity.x = -speed_abs;
        } else {
            boat.facing_right = true;
            boat.velocity.x = speed_abs;
        }
    }
}

/// Start reeling a specific hook back to its rod tip (where the line starts).
/// Called when the player is hooked.
pub fn start_reel_in(
    hook_id: HookId,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &SlotMap<LineId, Line>,
) {
    let Some(hook) = hooks.get_mut(hook_id) else { return; };
    let Some(line) = lines.get(hook.line_id) else { return; };

    let dir = math::vec3_normalize_or_zero(line.start_pos - hook.pos);
    hook.reel_velocity = Some(dir * 300.0);
    hook.reel_destination = Some(line.start_pos);
}

/// Remove a single worm (player ate it). Returns `true` if the worm actually
/// existed and was removed, so the caller knows whether to emit a
/// `WormDespawned` event.
pub fn despawn_worm(
    worm_id: WormId,
    worms: &mut SlotMap<WormId, Worm>,
    boats: &mut SlotMap<BoatId, Boat>,
) -> bool {
    if let Some(worm) = worms.remove(worm_id) {
        if let Some(boat) = boats.get_mut(worm.boat_id) {
            boat.worm_ids.retain(|&w| w != worm_id);
        }
        true
    } else {
        false
    }
}

/// Despawn every worm (called on GameOver to mirror original cosmetic clear).
/// Returns the IDs of every worm that was removed so the caller can emit
/// `WormDespawned` events.
pub fn despawn_all_worms(
    worms: &mut SlotMap<WormId, Worm>,
    boats: &mut SlotMap<BoatId, Boat>,
) -> Vec<WormId> {
    let removed: Vec<WormId> = worms.iter().map(|(id, _)| id).collect();
    worms.clear();
    for (_, boat) in boats.iter_mut() {
        boat.worm_ids.clear();
    }
    removed
}

/// How many boats to spawn this tick based on current difficulty, matching
/// the original `rng.gen_range(1..difficulty.multiplier + 1)`.
pub fn roll_boats_per_spawn(difficulty: u8, rng: &mut ChaCha8Rng) -> u8 {
    rng.gen_range(1..difficulty + 1)
}

/// Convenience: spawn a single boat using the current difficulty and arena.
pub fn spawn_random_boat(
    config: &FishGameConfig,
    difficulty: u8,
    rng: &mut ChaCha8Rng,
    boats: &mut SlotMap<BoatId, Boat>,
    hooks: &mut SlotMap<HookId, Hook>,
    lines: &mut SlotMap<LineId, Line>,
    worms: &mut SlotMap<WormId, Worm>,
) -> SpawnedBoat {
    let stats = roll_boat_stats(difficulty, rng);
    spawn_boat(stats, &config.arena, rng, boats, hooks, lines, worms)
}
