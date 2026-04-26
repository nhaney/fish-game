use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::config::PlayerStatsConfig;
use crate::input::FishGameInput;
use crate::math;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerState {
    Idle,
    Swim,
    Boost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub facing_right: bool,
    pub state: PlayerState,
    pub boosts_remaining: u8,
    pub hunger_ticks_remaining: u32,
    /// None when no active boost. Velocity/ticks-left + the state to return to
    /// when the boost ends.
    pub boost_data: Option<ActiveBoost>,
    /// None when boost is available. The cooldown timer + did-release flag
    /// work together: cooldown must expire AND the player must release the
    /// boost key before they can boost again.
    pub boost_cooldown: Option<BoostCooldown>,
    /// If true, boost is blocked this tick regardless of cooldown state. This
    /// mirrors the `blocked_transitions` set from the Bevy code — collapsed to
    /// a single flag because boost is the only blockable transition.
    pub boost_blocked: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ActiveBoost {
    pub velocity: Vec3,
    pub ticks_remaining: u32,
    pub prev_state: PlayerState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoostCooldown {
    pub ticks_remaining: u32,
    pub did_release: bool,
}

impl Player {
    pub fn new(stats: &PlayerStatsConfig) -> Self {
        Self {
            pos: Vec3::ZERO,
            velocity: Vec3::ZERO,
            facing_right: true,
            state: PlayerState::Idle,
            boosts_remaining: stats.max_boosts,
            hunger_ticks_remaining: stats.hunger_ticks,
            boost_data: None,
            boost_cooldown: None,
            boost_blocked: false,
        }
    }
}

pub fn can_transition_to(current: PlayerState, target: PlayerState, boost_blocked: bool) -> bool {
    if current == target {
        return false;
    }
    if target == PlayerState::Boost && boost_blocked {
        return false;
    }
    match current {
        PlayerState::Idle => target == PlayerState::Swim || target == PlayerState::Boost,
        PlayerState::Swim => target == PlayerState::Idle || target == PlayerState::Boost,
        PlayerState::Boost => target == PlayerState::Idle || target == PlayerState::Swim,
    }
}

/// Read input and write the player's target velocity, updating facing.
/// Returns the raw target speed vector (used by boost direction logic).
pub fn apply_input(
    stats: &PlayerStatsConfig,
    input: &FishGameInput,
    velocity: &mut Vec3,
    facing_right: &mut bool,
) -> Vec3 {
    let mut target_speed = Vec3::ZERO;

    if input.move_left {
        target_speed.x -= stats.speed;
        *facing_right = false;
    }
    if input.move_right {
        target_speed.x += stats.speed;
        *facing_right = true;
    }
    if input.move_up {
        target_speed.y += stats.speed;
    }
    if input.move_down {
        target_speed.y -= stats.speed;
    }

    // Apply traction when stopping, acceleration when moving.
    let a = if target_speed == Vec3::ZERO {
        stats.traction
    } else {
        stats.acceleration
    };

    *velocity = a * target_speed + (1.0 - a) * *velocity;

    if math::vec3_length(*velocity) < stats.stop_threshold {
        *velocity = Vec3::ZERO;
    }

    target_speed
}

/// Try to start a boost. Returns `true` if the boost started (so the caller
/// can emit the boost event/transition). If the supply is empty, we still set
/// a zero-length cooldown so the player has to release + repress before the
/// next attempt — matching the original behavior.
#[allow(clippy::too_many_arguments)]
pub fn try_start_boost(
    player: &mut Player,
    stats: &PlayerStatsConfig,
    target_speed: Vec3,
) -> bool {
    if !can_transition_to(player.state, PlayerState::Boost, player.boost_blocked) {
        return false;
    }

    if player.boosts_remaining > 0 {
        player.boosts_remaining -= 1;

        let boost_dir = if target_speed == Vec3::ZERO {
            if player.facing_right {
                Vec3::X
            } else {
                -Vec3::X
            }
        } else {
            math::vec3_normalize_or_zero(target_speed)
        };

        let prev_state = player.state;
        player.state = PlayerState::Boost;
        player.boost_data = Some(ActiveBoost {
            velocity: boost_dir * stats.boost_speed,
            ticks_remaining: stats.boost_duration_ticks,
            prev_state,
        });
        player.boost_cooldown = Some(BoostCooldown {
            ticks_remaining: stats.boost_cooldown_ticks,
            did_release: false,
        });
        true
    } else {
        // No boost available: still require a release before retrying.
        player.boost_cooldown = Some(BoostCooldown {
            ticks_remaining: 0,
            did_release: false,
        });
        false
    }
}
