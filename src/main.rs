use bevy::{prelude::*, window::WindowMode};

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
        .add_plugin(shared::SharedPlugin)
        .add_plugin(player::PlayerPlugin)
        .run();
}

struct GameState {
    paused: bool,
    game_over: bool,
    game_timer: Timer,
    score: f32,
}
