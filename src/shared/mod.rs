use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
};
pub mod animation;
pub mod arena;
pub mod collision;
pub mod game;
pub mod movement;
pub mod render;
pub mod rng;
pub mod stages;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building shared plugin...");
        app.add_startup_system(initialize_game.system())
            .add_startup_system(arena::initialize_arena.system())
            .add_system_to_stage(stages::MOVEMENT, movement::movement_system.system())
            .add_system_to_stage_front(stages::MOVEMENT, movement::flip_transform_system.system())
            .add_system_to_stage(stages::PREPARE_RENDER, animation::animation_system.system())
            .init_resource::<rng::GameRng>()
            .add_resource(game::Difficulty {
                multiplier: 1,
                timer: Timer::from_seconds(10.0, true),
            })
            .add_resource(game::Score {
                count: 0,
                timer: Timer::from_seconds(1.0, true),
            })
            .add_system(game::difficulty_scaling_system.system())
            .add_system_to_stage(stage::LAST, game::increment_score_system.system())
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                render::scale_camera_to_screen_size.system(),
            );
        // .add_system_to_stage(
        //     stages::PREPARE_RENDER,
        //     render::scale_transforms_to_screen_size.system(),
        // )
        // .add_system_to_stage(stages::POST_RENDER, render::revert_transform_scale.system());

        // if cfg!(debug_assertions) {
        //     println!("Adding diagnostic plugins for debug mode...");
        //     app.add_plugin(PrintDiagnosticsPlugin::default())
        //         .add_plugin(FrameTimeDiagnosticsPlugin::default());
        // }
    }
}

fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
