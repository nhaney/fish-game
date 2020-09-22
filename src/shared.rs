/*
Components and systems shared by more than one part of the game
*/
use bevy::prelude::*;

pub struct Velocity(pub Vec3);

pub fn movement_system(time: Res<Time>, velocity: &Velocity, mut transform: Mut<Transform>) {
    let translation = transform.translation_mut();

    *translation.x_mut() += time.delta_seconds * velocity.0.x();
    *translation.y_mut() += time.delta_seconds * velocity.0.y();
}
