use crate::shared::stages;
use bevy::prelude::*;

pub(crate) mod boat;

pub struct ObjectPlugins;

impl Plugin for ObjectPlugins {
    fn build(&self, app: &mut App) {
        debug!("Building object plugin...");
        app.insert_resource(boat::BoatSpawner {
            spawn_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        })
        .init_resource::<boat::BoatMaterials>()
        .add_systems(
            Update,
            (boat::boat_exit_system, boat::boat_spawner_system).in_set(stages::EmitEventsSet),
        )
        .add_systems(
            Update,
            (
                boat::player_bonked_handler,
                boat::player_hooked_handler,
                boat::boat_exit_system,
                boat::worm_eaten_system,
                boat::despawn_worms_on_game_over,
            )
                .in_set(stages::HandleEventsSet),
        )
        .add_systems(
            Update,
            (boat::despawn_boat_system,).in_set(stages::CalculateCollisionsSet),
        )
        .add_systems(
            Update,
            (
                boat::redraw_line_when_hook_moves,
                boat::reset_boats_on_restart,
            )
                .in_set(stages::PrepareRenderSet),
        );
    }
}
