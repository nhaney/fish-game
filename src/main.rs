use bevy::{
    asset::AssetMetaCheck,
    //    diagnostic::LogDiagnosticsPlugin,
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
    let default_plugins = DefaultPlugins
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
            #[cfg(debug_assertions)]
            level: bevy::log::Level::DEBUG,
            #[cfg(not(debug_assertions))]
            level: bevy::log::Level::ERROR,
            ..default()
        })
        .set(ImagePlugin::default_nearest());

    /* TODO: Do we need this plugin?
    .add(LogDiagnosticsPlugin {
        debug: true,
        ..default()
    });*/

    let mut app = App::new();

    app.insert_resource(ClearColor(Color::rgb_u8(230, 202, 173)))
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            default_plugins,
            shared::SharedPlugin,
            leaderboard::LeaderboardPlugin,
            player::PlayerPlugin,
            objects::ObjectPlugins,
            ui::UIPlugin,
            audio::AudioPlugin,
        ));

    app.run();
}
