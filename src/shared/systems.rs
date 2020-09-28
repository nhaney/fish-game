use super::components::{SideScrollDirection, Velocity};
use bevy::prelude::*;

pub fn movement_system(time: Res<Time>, velocity: &Velocity, mut transform: Mut<Transform>) {
    let translation = transform.translation_mut();

    *translation.x_mut() += time.delta_seconds * velocity.0.x();
    *translation.y_mut() += time.delta_seconds * velocity.0.y();
}

pub fn flip_sprite_system(direction: &SideScrollDirection, mut transform: Mut<Transform>) {
    if direction.is_left() {
        transform.set_rotation(Quat::from_rotation_y(std::f32::consts::PI));
    } else {
        transform.set_rotation(Quat::from_rotation_y(0.0));
    }
}

pub fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
