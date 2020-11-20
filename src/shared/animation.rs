use super::movement::SideScrollDirection;
use bevy::prelude::*;

/**
Represents one frame of animation. The atlas index references the TextureAtlas
handle on the entity.
*/
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AnimationFrame {
    pub atlas_index: usize,
    pub time: f32,
}

/// Represents an entire animation
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub should_loop: bool,
    pub frames: Vec<AnimationFrame>,
}

/// Component that represents the current state of animation
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub animation: Animation,
    pub timer: Timer,
    pub frame_index: u32,
    pub speed_multiplier: f32,
}

/// Transitions the animation state if it is time for the next frame
pub(super) fn animation_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite)>,
) {
    for (mut animation_state, mut texture_atlas_sprite) in query.iter_mut() {
        animation_state.timer.tick(time.delta_seconds);

        if animation_state.timer.finished {
            let cur_animation = &animation_state.animation;
            let cur_frame = animation_state.frame_index;
            let num_frames = cur_animation.frames.len();

            if (cur_frame + 1) == num_frames as u32 && !cur_animation.should_loop {
                continue;
            }

            let next_frame_index =
                (animation_state.frame_index + 1) % animation_state.animation.frames.len() as u32;
            let next_frame = cur_animation.frames[next_frame_index as usize];

            animation_state.frame_index = next_frame_index;

            animation_state.timer.duration = next_frame.time;
            animation_state.timer.reset();

            texture_atlas_sprite.index = animation_state.frame_index;
        }
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
