use bevy::{prelude::*, window::WindowMode};

mod leaderboard;
mod objects;
mod player;
mod shared;
mod ui;

fn main() {
    let mut app = App::build();

    app.add_resource(WindowDescriptor {
        title: "Stay Off the Line!".to_string(),
        width: 1289,
        height: 720,
        vsync: false,
        resizable: true,
        mode: WindowMode::Windowed,
        ..Default::default()
    })
    .add_resource(bevy::render::pass::ClearColor(Color::rgb_u8(230, 202, 173)))
    .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.add_plugin(shared::SharedPlugin)
        .add_plugin(leaderboard::LeaderboardPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(objects::ObjectPlugins)
        .add_plugin(ui::UIPlugin)
        .run();
}
