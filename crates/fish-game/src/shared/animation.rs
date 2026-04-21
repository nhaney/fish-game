use bevy::prelude::*;
use std::time::Duration;

use crate::core_adapter::{CoreControl, CoreState};
use fish_game_core::GamePhase;

/**
Represents one frame of animation.
**/
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationFrame {
    pub material_handle: Handle<Image>,
    pub time: f32,
}

/// Represents an entire animation
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub should_loop: bool,
    pub frames: Vec<AnimationFrame>,
}

/// Component that represents the current state of animation
#[derive(Debug, Clone, Component)]
pub struct AnimationState {
    pub animation: Animation,
    pub timer: Timer,
    pub frame_index: usize,
    pub speed_multiplier: f32,
}

// TODO: Add a sprite material when this component is added
impl AnimationState {
    pub fn from_animation(animation: &Animation, speed_multiplier: f32) -> Self {
        debug!("speed multiplier: {:?}", speed_multiplier);
        AnimationState {
            animation: animation.clone(),
            timer: Timer::from_seconds(animation.frames[0].time, TimerMode::Once),
            frame_index: 0,
            speed_multiplier,
        }
    }
}

/// Transitions the animation state if it is time for the next frame
pub(super) fn animation_system(
    time: Res<Time>,
    core: Res<CoreState>,
    control: Res<CoreControl>,
    mut query: Query<(&mut AnimationState, &mut Sprite)>,
) {
    if control.paused || core.state.phase != GamePhase::Running {
        return;
    }

    for (mut animation_state, mut sprite) in query.iter_mut() {
        let speed_multiplier = animation_state.speed_multiplier;
        animation_state.timer.tick(Duration::from_secs_f32(
            time.delta().as_secs_f32() * speed_multiplier,
        ));

        if animation_state.timer.is_finished() {
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

            animation_state
                .timer
                .set_duration(Duration::from_secs_f32(next_frame.time));
            animation_state.timer.reset();

            sprite.image = next_frame.material_handle.clone();
        }
    }
}
