use bevy::{asset::stage::ASSET_EVENTS, prelude::*};

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
        // Setup stages
        app.add_stage_after(stage::PRE_UPDATE, stages::HANDLE_EVENTS)
            .add_stage_after(stages::HANDLE_EVENTS, stages::MOVEMENT)
            .add_stage_after(stages::FINALIZE_MOVEMENT, stages::PREPARE_RENDER)
            .add_stage_before(ASSET_EVENTS, stages::CALCULATE_COLLISIONS)
            /* Startup systems
            - Spawn the camera
            - Create the arena
            */
            .add_startup_system(initialize_game)
            .add_startup_system(arena::initialize_arena)
            /* Resources
            - Seedable rng
            - Score counter
            - Difficulty manager
            - Overall state of game
            */
            .init_resource::<rng::GameRng>()
            .add_resource(game::Difficulty {
                multiplier: 1,
                timer: Timer::from_seconds(10.0, true),
            })
            .add_resource(game::Score {
                count: 0,
                timer: Timer::from_seconds(1.0, true),
            })
            .add_resource(game::GameState {
                cur_state: game::GameStates::Running,
                prev_state: game::GameStates::Running,
            })
            /* Events
            - Game state changes
            - Generic event to a transform reaching its destination
            */
            .add_event::<game::GameOver>()
            .add_event::<game::GamePaused>()
            .add_event::<game::GameUnpaused>()
            .add_event::<game::GameRestarted>()
            .add_event::<movement::DestinationReached>()
            // Timer systems
            .add_system_to_stage(stage::EVENT, game::difficulty_scaling_system)
            .add_system_to_stage(stage::EVENT, game::increment_score_system)
            // Systems that handle input and collisions
            .add_system_to_stage(stages::HANDLE_EVENTS, game::restart_game)
            .add_system_to_stage(stages::HANDLE_EVENTS, game::end_game_system)
            // Systems that finalize transform positions per frame
            .add_system_to_stage(stages::FINALIZE_MOVEMENT, movement::movement_system)
            .add_system_to_stage(stages::FINALIZE_MOVEMENT, movement::flip_transform_system)
            // Collision detection for destinations
            .add_system_to_stage(
                stages::CALCULATE_COLLISIONS,
                movement::check_distance_from_destination,
            )
            // Systems that prepare the frame for rendering and handles all final events
            .add_system_to_stage(stages::PREPARE_RENDER, animation::animation_system)
            .add_system_to_stage(stages::PREPARE_RENDER, game::finalize_score)
            .add_system_to_stage(stages::PREPARE_RENDER, render::scale_camera_to_screen_size)
            .add_system_to_stage(stages::PREPARE_RENDER, rng::reset_rng_on_restart)
            .add_system_to_stage(stages::PREPARE_RENDER, game::reset_difficulty_on_restart)
            .add_system_to_stage(stages::PREPARE_RENDER, game::reset_game_state_on_restart)
            .add_system_to_stage(stages::PREPARE_RENDER, game::reset_score_on_restart)
            .add_system_to_stage(stages::PREPARE_RENDER, game::pause_game)
            .add_system_to_stage(stages::PREPARE_RENDER, game::unpause_game)
            .add_system_to_stage(stages::PREPARE_RENDER, render::readjust_rotation)
            .add_system_to_stage("post_render", render::print_rot_after_render);

        // if cfg!(debug_assertions) {
        //     println!("Adding diagnostic plugins for debug mode...");
        //     app.add_plugin(PrintDiagnosticsPlugin::default())
        //         .add_plugin(FrameTimeDiagnosticsPlugin::default());
        // }
    }
}

fn initialize_game(commands: &mut Commands) {
    commands.spawn(Camera2dBundle::default());
}
