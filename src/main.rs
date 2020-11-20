use bevy::{prelude::*, window::WindowMode};

mod objects;
mod player;
mod shared;

use shared::stages;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Stay Off the Line!".to_string(),
            width: 640,
            height: 360,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .add_resource(bevy::render::pass::ClearColor(Color::rgb_u8(230, 202, 173)))
        .add_stage_before(stage::PRE_UPDATE, stages::PREPARE_RENDER)
        .add_stage_before(stages::PREPARE_RENDER, stages::CORRECT_MOVEMENT)
        .add_stage_before(stages::CORRECT_MOVEMENT, stages::MOVEMENT)
        .add_stage_before(stages::MOVEMENT, stages::CALCULATE_VELOCITY)
        .add_plugins(DefaultPlugins)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(objects::ObjectPlugins)
        .add_plugin(shared::SharedPlugin)
        .run();
}
