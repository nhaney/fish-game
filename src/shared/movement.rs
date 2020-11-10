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

pub fn movement_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        *transform.translation.x_mut() += time.delta_seconds * velocity.0.x();
        *transform.translation.y_mut() += time.delta_seconds * velocity.0.y();
    }
}
