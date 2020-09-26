/*
Components and systems shared by more than one part of the game
*/
use bevy::prelude::*;

pub struct Velocity(pub Vec3);

pub struct SideScrollDirection(pub bool);

impl SideScrollDirection {
    pub fn is_right(&self) -> bool {
        self.0
    }

    pub fn is_left(&self) -> bool {
        !self.0
    }
}

pub struct Collider {
    pub width: f32,
    pub height: f32,
}

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
