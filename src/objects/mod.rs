use crate::shared::stages;
use bevy::prelude::*;

pub(crate) mod boat;

pub struct ObjectPlugins;

impl Plugin for ObjectPlugins {
    fn build(&self, app: &mut AppBuilder) {
        debug!("Building object plugin...");
        app.add_resource(boat::BoatSpawner {
            spawn_timer: Timer::from_seconds(5.0, true),
        })
        .init_resource::<boat::BoatMaterials>()
        .add_system_to_stage(stage::EVENT, boat::boat_spawner_system.system())
        // collision handlers
        .add_system_to_stage(stages::HANDLE_EVENTS, boat::player_hooked_handler.system())
        .add_system_to_stage(stages::HANDLE_EVENTS, boat::player_bonked_handler.system())
        .add_system_to_stage(stages::HANDLE_EVENTS, boat::boat_exit_system.system())
        .add_system_to_stage(stages::HANDLE_EVENTS, boat::worm_eaten_system.system())
        .add_system_to_stage(
            stages::HANDLE_EVENTS,
            boat::despawn_worms_on_game_over.system(),
        )
        // collision detection
        .add_system_to_stage(
            stages::CALCULATE_COLLISIONS,
            boat::despawn_boat_system.system(),
        )
        // final event handlers and presentation
        .add_system_to_stage(
            stages::PREPARE_RENDER,
            boat::redraw_line_when_hook_moves.system(),
        )
        .add_system_to_stage(
            stages::PREPARE_RENDER,
            boat::reset_boats_on_restart.system(),
        );
    }
}
