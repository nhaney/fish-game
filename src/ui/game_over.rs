use bevy::prelude::*;

pub(super) fn add_game_over_text(parent: &mut ChildBuilder, asset_server: &AssetServer) {
    parent
        .spawn(NodeComponents {
            style: Style {
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextComponents {
                style: Style {
                    ..Default::default()
                },
                text: Text {
                    value: "test".to_string(),
                    font: asset_server.load("fonts/Chonkly.ttf"),
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::RED,
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
}
