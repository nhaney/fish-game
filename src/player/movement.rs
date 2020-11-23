use bevy::prelude::*;

use super::attributes::{Player, Sink};
use crate::shared::movement::{SideScrollDirection, Velocity};

/**
Reads keyboard input and adjusts players velocity based on it. Returns
the target speed of the player.
*/
// TODO: Change to use specific player command events
pub(super) fn move_player_from_input(
    keyboard_input: &Input<KeyCode>,
    player: &Player,
    mut velocity: &mut Velocity,
    mut facing: &mut SideScrollDirection,
) -> Vec3 {
    let mut target_speed = Vec3::zero();

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        target_speed.x -= player.stats.speed;
        facing.0 = false;
    }

    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        target_speed.x += player.stats.speed;
        facing.0 = true;
    }

    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        target_speed.y += player.stats.speed;
    }

    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        target_speed.y -= player.stats.speed;
    }

    // determine whether to apply traction or regular acceleration
    let a = if target_speed == Vec3::zero() {
        player.stats.traction
    } else {
        player.stats.acceleration
    };

    // calculate new player velocity based on acceleration
    velocity.0 = a * target_speed + (1.0 - a) * velocity.0;

    if velocity.0.length() < player.stats.stop_threshold {
        velocity.0 = Vec3::zero();
    }

    target_speed
}

/// sinks the player based on their weight
pub(super) fn sink_system(mut query: Query<(&mut Velocity, &Sink)>) {
    for (mut velocity, sink) in query.iter_mut() {
        velocity.0.y -= sink.weight;
    }
}
