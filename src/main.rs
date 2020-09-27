use bevy::{prelude::*, window::WindowMode};

mod arena;
mod player;
mod shared;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Stay Off the Line!".to_string(),
            width: 800,
            height: 600,
            vsync: false,
            resizable: false,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_resource(bevy::render::pass::ClearColor(Color::rgb_u8(230, 202, 173)))
        .add_default_plugins()
        .add_startup_system(initialize_game.system())
        .add_startup_system(player::init_player.system())
        .add_startup_system(arena::initialize_arena.system())
        .add_system(player::normal_player_movement_system.system())
        .add_system(player::boost_cooldown_system.system())
        .add_system(player::start_boost_system.system())
        .add_system(player::boost_player_movement_system.system())
        .add_system(player::sink_system.system())
        .add_system(shared::movement_system.system())
        .add_system(player::player_bounds_system.system())
        .add_system(shared::flip_sprite_system.system())
        .run();
}

fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
