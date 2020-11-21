use crate::shared::stages;
use bevy::prelude::*;

pub(crate) mod boat;

pub struct ObjectPlugins;

impl Plugin for ObjectPlugins {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building object plugin...");
        app.add_resource(boat::BoatSpawner {
            spawn_timer: Timer::from_seconds(5.0, true),
        })
        .add_system_to_stage(stage::EVENT, boat::boat_spawner_system.system())
        .add_system_to_stage(stages::CORRECT_MOVEMENT, boat::despawn_boat_system.system())
        .add_system_to_stage(stage::LAST, boat::boat_exit_system.system())
        .add_system_to_stage(stage::LAST, boat::worm_eaten_system.system());
    }
}
