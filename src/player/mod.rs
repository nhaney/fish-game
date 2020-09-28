use bevy::prelude::*;

mod components;
mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(systems::init_player.system())
            .add_system(systems::normal_player_movement_system.system())
            .add_system(systems::boost_cooldown_system.system())
            .add_system(systems::start_boost_system.system())
            .add_system(systems::boost_player_movement_system.system())
            .add_system(systems::sink_system.system())
            .add_system(systems::player_bounds_system.system())
            .run();
    }
}
