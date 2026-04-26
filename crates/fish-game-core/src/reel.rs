//! Pure reel-in motion. No Bevy, no state, no time source — caller owns
//! all of that. Used by the presentation-layer game-over wind-down today,
//! and reusable for future mid-game moving-hook features.

use crate::math::{vec3_length, vec3_normalize_or_zero};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReelMotion {
    pub destination: Vec3,
    pub velocity: Vec3,
    pub arrival_radius: f32,
}

impl ReelMotion {
    pub fn toward(from: Vec3, to: Vec3, speed: f32) -> Self {
        let dir = vec3_normalize_or_zero(to - from);
        Self {
            destination: to,
            velocity: dir * speed,
            arrival_radius: 10.0,
        }
    }
}

/// Advance `pos` toward `motion.destination` by `motion.velocity * dt`.
/// Returns `(new_pos, arrived)`. If arrived, the position is snapped to the
/// destination and the caller should drop / freeze the motion.
pub fn advance(pos: Vec3, motion: ReelMotion, dt: f32) -> (Vec3, bool) {
    let next = pos + motion.velocity * dt;
    let to_dest = motion.destination - next;
    if vec3_length(to_dest) <= motion.arrival_radius {
        (motion.destination, true)
    } else {
        (next, false)
    }
}
