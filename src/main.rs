use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::{PresentMode, WindowMode},
    winit::winit_window_position,
};
//// test test test more testing! edit with vim...WTF!!!
mod audio;
mod leaderboard;
mod objects;
mod player;
mod shared;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(230, 202, 173)))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Stay Off the Line!".to_string(),
                        resolution: (1280., 720.).into(),
                        present_mode: PresentMode::AutoNoVsync,
                        prevent_default_event_handling: false,
                        #[cfg(target_arch = "wasm32")]
                        canvas: Some("#fish-game".to_string()),
                        resizable: true,
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::DEBUG,
                    ..default()
                }),
            LogDiagnosticsPlugin {
                debug: true,
                ..default()
            },
            //FrameTimeDiagnosticsPlugin,
            shared::SharedPlugin,
            leaderboard::LeaderboardPlugin,
            player::PlayerPlugin,
            objects::ObjectPlugins,
            ui::UIPlugin,
            audio::AudioPlugin,
        ))
        .run();
    /*
    .add_resource(WindowDescriptor {
        title: "Stay Off the Line!".to_string(),
        width: 1280.0,
        height: 720.0,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#fish-game".to_string()),
        vsync: false,
        resizable: true,
        mode: WindowMode::Windowed,
        ..Default::default()
    })
    .add_resource(bevy::log::LogSettings {
        level: bevy::log::Level::DEBUG,
        filter: "wgpu=error,bevy_webgl2=warn,bevy_ecs=info".to_string(),
    })
    .add_plugins(DefaultPlugins);*/

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
}
