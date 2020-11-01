use bevy::prelude::*;

use super::components::{Player, Sink};
use crate::shared::{
    arena::Arena,
    components::{Collider, SideScrollDirection, Velocity},
};

/**
Reads keyboard input and adjusts players velocity based on it. Returns
the target speed of the player.
*/
// TODO: Change to use specific player command events
pub fn move_player_from_input(
    keyboard_input: &Input<KeyCode>,
    player: &Player,
    mut velocity: &mut Velocity,
    mut facing: &mut SideScrollDirection,
) -> Vec3 {
    let mut target_speed = Vec3::zero();

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        *target_speed.x_mut() -= player.stats.speed;
        facing.0 = false;
    }

    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        *target_speed.x_mut() += player.stats.speed;
        facing.0 = true;
    }

    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        *target_speed.y_mut() += player.stats.speed;
    }

    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        *target_speed.y_mut() -= player.stats.speed;
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
pub fn sink_system(mut velocity: Mut<Velocity>, sink: &Sink) {
    *velocity.0.y_mut() -= sink.weight;
}

/// Keeps player in bounds of arena
pub fn player_bounds_system(
    arena: Res<Arena>,
    _player: &Player,
    mut transform: Mut<Transform>,
    collider: &Collider,
) {
    let mut new_pos = transform.translation().clone();

    let arena_half_width = arena.width / 2.0;
    let arena_half_height = arena.height / 2.0;

    let player_half_width = collider.width / 2.0;
    let player_half_height = collider.height / 2.0;

    if new_pos.x() - player_half_width < -arena_half_width {
        *new_pos.x_mut() = -arena_half_width + player_half_width;
    }

    if new_pos.x() + player_half_width > arena_half_width {
        *new_pos.x_mut() = arena_half_width - player_half_width;
    }

    if new_pos.y() - player_half_height < -arena_half_height {
        *new_pos.y_mut() = -arena_half_height + player_half_height;
    }

    if new_pos.y() + player_half_height > (arena_half_height + arena.offset) {
        *new_pos.y_mut() = (arena_half_height + arena.offset) - player_half_height;
    }

    transform.set_translation(new_pos);
}
