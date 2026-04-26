use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{PresentMode, WindowMode},
};
mod audio;
mod core_adapter;
mod leaderboard;
mod player;
mod shared;
mod ui;

fn main() {
    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Stay Off the Line!".to_string(),
                resolution: (1280u32, 720u32).into(),
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

    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb_u8(230, 202, 173)))
        .add_plugins((
            default_plugins,
            shared::SharedPlugin,
            core_adapter::CorePlugin,
            leaderboard::LeaderboardPlugin,
            player::PlayerPlugin,
            ui::UIPlugin,
            audio::AudioPlugin,
        ));

    app.run();
}
