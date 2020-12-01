use bevy::prelude::*;

use super::game::GameState;

/**
Represents one frame of animation.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationFrame {
    pub material_handle: Handle<ColorMaterial>,
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
    pub frame_index: usize,
    pub speed_multiplier: f32,
}

// TODO: Add a sprite material when this component is added
impl AnimationState {
    pub fn from_animation(animation: &Animation) -> Self {
        AnimationState {
            animation: animation.clone(),
            timer: Timer::from_seconds(animation.frames[0].time, false),
            frame_index: 0,
            speed_multiplier: 1.0,
        }
    }
}

/// Transitions the animation state if it is time for the next frame
pub(super) fn animation_system(
    time: Res<Time>,
    game_state: Res<GameState>,
    mut query: Query<(&mut AnimationState, &mut Handle<ColorMaterial>)>,
) {
    if !game_state.is_running() {
        return;
    }

    for (mut animation_state, mut material_handle) in query.iter_mut() {
        animation_state.timer.tick(time.delta_seconds());

        if animation_state.timer.finished() {
            let cur_animation = &animation_state.animation;
            let cur_frame = animation_state.frame_index;
            let num_frames = cur_animation.frames.len();

            if (cur_frame + 1) == num_frames && !cur_animation.should_loop {
                continue;
            }

            let next_frame_index =
                (animation_state.frame_index + 1) % animation_state.animation.frames.len();
            let next_frame = cur_animation.frames[next_frame_index].clone();

            animation_state.frame_index = next_frame_index;

            animation_state.timer.set_duration(next_frame.time);
            animation_state.timer.reset();

            *material_handle = next_frame.material_handle.clone();
        }
    }
}
