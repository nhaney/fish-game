use super::components::{SideScrollDirection, Velocity};
use bevy::prelude::*;

pub fn movement_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        *transform.translation.x_mut() += time.delta_seconds * velocity.0.x();
        *transform.translation.y_mut() += time.delta_seconds * velocity.0.y();
    }
}

pub fn flip_sprite_system(mut query: Query<(&SideScrollDirection, &mut Transform)>) {
    for (direction, mut transform) in query.iter_mut() {
        if direction.is_left() {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        } else {
            transform.rotation = Quat::from_rotation_y(0.0);
        }
    }
}

pub fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
