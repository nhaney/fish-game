use bevy::prelude::*;

use super::game::{GameState, GameStates};

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

pub fn movement_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&Velocity, &mut Transform)>,
) {
    if let GameStates::Paused = game_state.cur_state {
        return;
    }

    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += time.delta_seconds * velocity.0.x;
        transform.translation.y += time.delta_seconds * velocity.0.y;
        transform.translation.z = 1.0;
    }
}

pub fn flip_transform_system(mut query: Query<(&SideScrollDirection, &mut Transform)>) {
    for (direction, mut transform) in query.iter_mut() {
        if direction.is_left() {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        } else {
            transform.rotation = Quat::from_rotation_y(0.0);
        }
    }
}
