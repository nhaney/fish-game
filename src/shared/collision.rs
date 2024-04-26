use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
}

impl Collider {
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub fn as_vec2_half_size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}
