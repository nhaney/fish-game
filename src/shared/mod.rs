use bevy::prelude::*;

pub mod animation;
pub mod arena;
pub mod collision;
pub mod movement;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(initialize_game.system())
            .add_startup_system(arena::initialize_arena.system())
            .add_system(movement::movement_system.system())
            .add_system(animation::flip_sprite_system.system())
            .add_system(animation::animation_system.system());
    }
}

fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
