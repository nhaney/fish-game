use bevy::prelude::*;

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
    fn build(&self, app: &mut App) {
        debug!("Building shared plugin...");

        // Configure ordering of custom system sets.
        app.configure_sets(
            Update,
            (stages::EmitEventsSet).before(stages::HandleEventsSet),
        )
        .configure_sets(
            Update,
            (stages::HandleEventsSet).before(stages::MovementSet),
        )
        .configure_sets(
            Update,
            (stages::MovementSet).before(stages::FinalizeMovementSet),
        )
        .configure_sets(
            Update,
            (stages::FinalizeMovementSet).before(stages::CalculateCollisionsSet),
        )
        .configure_sets(
            Update,
            (stages::CalculateCollisionsSet).before(stages::AdjustPositionsSet),
        )
        .configure_sets(
            Update,
            (stages::AdjustPositionsSet).before(stages::PrepareRenderSet),
        )
        .configure_sets(
            Update,
            (stages::FinalizeMovementSet)
                .after(stages::MovementSet)
                .before(stages::PrepareRenderSet),
        );

        /* Startup systems
        - Spawn the camera
        - Create the arena
        */
        app.add_systems(Startup, (initialize_game, arena::initialize_arena));

        /* Resources
        - Seedable rng
        - Score counter
        - Difficulty manager
        - Overall state of game
        */
        app.init_resource::<rng::GameRng>()
            .insert_resource(game::Difficulty {
                multiplier: 1,
                timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            })
            .insert_resource(game::Score {
                count: 0,
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            })
            .insert_resource(game::GameState {
                cur_state: game::GameStates::Running,
                prev_state: game::GameStates::Running,
            });
        /* Events
        - Game state changes
        - Generic event to a transform reaching its destination
        */
        app.add_event::<game::GameOver>()
            .add_event::<game::GamePaused>()
            .add_event::<game::GameUnpaused>()
            .add_event::<game::GameRestarted>()
            .add_event::<movement::DestinationReached>();
        // Timer systems.
        app.add_systems(
            Update,
            (
                game::difficulty_scaling_system,
                game::increment_score_system,
            )
                .in_set(stages::EmitEventsSet),
        )
        // Game state transition systems.
        .add_systems(
            Update,
            (game::restart_game, game::end_game_system).in_set(stages::HandleEventsSet),
        )
        // Systems that finalize transform positions per frame
        .add_systems(
            Update,
            (movement::movement_system, movement::flip_transform_system)
                .in_set(stages::FinalizeMovementSet),
        )
        // Collision detection for destinations
        .add_systems(
            Update,
            (movement::check_distance_from_destination).in_set(stages::CalculateCollisionsSet),
        )
        // Systems that prepare the frame for rendering and handles all final events
        .add_systems(
            Update,
            (
                movement::follow_system,
                render::adjust_to_render_layer,
                animation::animation_system,
                game::finalize_score,
                render::scale_camera_to_screen_size,
                rng::reset_rng_on_restart,
                game::reset_difficulty_on_restart,
                game::reset_game_state_on_restart,
                game::reset_score_on_restart,
                game::pause_game,
                game::unpause_game,
            )
                .in_set(stages::PrepareRenderSet),
        );
    }
}

#[derive(Component)]
pub struct MainCamera;

fn initialize_game(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
