use bevy::prelude::*;

pub mod arena;
pub mod components;
pub mod systems;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(systems::initialize_game.system())
            .add_startup_system(arena::initialize_arena.system())
            .add_system(systems::movement_system.system())
            .add_system(systems::flip_sprite_system.system());
    }
}
