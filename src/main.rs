use bevy::{prelude::*, window::WindowMode};

mod player;
mod shared;

/// An implementation of the classic game "Breakout"
fn main() {
    App::build()
        .add_default_plugins()
        .add_resource(WindowDescriptor {
            title: "Stay Off the Line".to_string(),
            width: 800,
            height: 600,
            vsync: false,
            resizable: false,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_startup_system(initialize_game.system())
        .add_startup_system(player::init_player.system())
        .add_system(player::player_movement_system.system())
        .add_system(player::sink_system.system())
        .add_system(shared::movement_system.system())
        .add_system(shared::flip_sprite_system.system())
        .run();
}

fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
