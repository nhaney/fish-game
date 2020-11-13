use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy, Properties)]
pub struct GameTransform {
    pub cur_transform: Transform,
    pub prev_transform: Transform,
}

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

pub fn movement_system(time: Res<Time>, mut query: Query<(&Velocity, &mut GameTransform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.prev_transform = transform.cur_transform;

        transform.cur_transform.translation += time.delta_seconds * velocity.0;
    }
}
