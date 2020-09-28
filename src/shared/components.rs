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
