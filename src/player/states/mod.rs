use bevy::prelude::*;

mod boost;
pub(crate) mod idle;
mod swim;

pub struct PlayerStatePlugin;

impl Plugin for PlayerStatePlugin {
    fn build(&self, app: &mut AppBuilder) {
        // state change events
        app.add_event::<boost::BoostStarted>()
            .add_event::<idle::IdleStarted>()
            .add_event::<swim::SwimStarted>()
            // boost state systems
            .add_system(boost::boost_player_movement_system.system())
            .add_system(boost::boost_starter.system())
            .add_system(boost::boost_cooldown_system.system())
            // idle state systems
            .add_system(idle::idle_starter.system())
            .add_system(idle::idle_player_movement_system.system())
            // swim state systems
            .add_system(swim::swim_starter.system())
            .add_system(swim::swim_player_movement_system.system());
    }
}
