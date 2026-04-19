use bevy::prelude::*;

use super::attributes::{Player, Sink};
use crate::shared::{
    game::GameState,
    movement::{SideScrollDirection, Velocity},
};

/**
Reads keyboard input and adjusts players velocity based on it. Returns
the target speed of the player.
*/
// TODO: Change to use specific player command events
pub(super) fn move_player_from_input(
    keyboard_input: &ButtonInput<KeyCode>,
    player: &Player,
    velocity: &mut Velocity,
    facing: &mut SideScrollDirection,
) -> Vec3 {
    let mut target_speed = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        target_speed.x -= player.stats.speed;
        facing.0 = false;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        target_speed.x += player.stats.speed;
        facing.0 = true;
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        target_speed.y += player.stats.speed;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        target_speed.y -= player.stats.speed;
    }

    // determine whether to apply traction or regular acceleration
    let a = if target_speed == Vec3::ZERO {
        player.stats.traction
    } else {
        player.stats.acceleration
    };

    // calculate new player velocity based on acceleration
    velocity.0 = a * target_speed + (1.0 - a) * velocity.0;

    if velocity.0.length() < player.stats.stop_threshold {
        velocity.0 = Vec3::ZERO;
    }

    target_speed
}

/// sinks the player based on their weight
pub(super) fn sink_system(game_state: Res<GameState>, mut query: Query<(&mut Velocity, &Sink)>) {
    if !game_state.is_running() {
        return;
    }

    for (mut velocity, sink) in query.iter_mut() {
        velocity.0.y -= sink.weight;
    }
}
