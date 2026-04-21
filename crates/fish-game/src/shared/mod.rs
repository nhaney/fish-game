use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

pub mod animation;
pub mod arena;
pub mod collision;
pub mod game;
pub mod render;
pub mod stages;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        info!("Building shared plugin...");

        // Add plugin to render shapes with bevy_prototype_lyon.
        app.add_plugins(ShapePlugin);

        app.init_resource::<render::FontHandles>();

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

        // Startup: camera + arena.
        app.add_systems(Startup, (initialize_game, arena::initialize_arena));

        // Gameplay-lifecycle events. The adapter emits them; UI / animations /
        // audio consume them.
        app.add_event::<game::GameOver>()
            .add_event::<game::GamePaused>()
            .add_event::<game::GameUnpaused>()
            .add_event::<game::GameRestarted>();

        // Presentation-only systems.
        app.add_systems(
            Update,
            (
                render::adjust_to_render_layer,
                animation::animation_system,
                render::scale_camera_to_screen_size,
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
