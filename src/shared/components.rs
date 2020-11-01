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

/// Represents one frame of animation
pub struct AnimationFrame {
    atlas_index: u32,
    time: f32,
}

/// Represents an entire animation
pub struct Animation {
    should_loop: bool,
    frames: Vec<AnimationFrame>,
}

/// Component that represents the current state of animation
pub struct AnimationState {
    animation: Animation,
    timer: Timer,
    frame_index: u32,
    speed_multiplier: f32,
}
