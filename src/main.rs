use bevy::{
    diagnostic::LogDiagnosticsPlugin,
    prelude::*,
    window::{PresentMode, WindowMode},
};
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
                })
                .set(ImagePlugin::default_nearest()),
            LogDiagnosticsPlugin {
                debug: true,
                ..default()
            },
            shared::SharedPlugin,
            leaderboard::LeaderboardPlugin,
            player::PlayerPlugin,
            objects::ObjectPlugins,
            ui::UIPlugin,
            audio::AudioPlugin,
        ))
        .run();
}
