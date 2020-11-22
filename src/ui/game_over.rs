use bevy::prelude::*;

pub(super) fn add_game_over_text(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    materials: &mut Assets<ColorMaterial>,
) {
    parent
        .spawn(NodeComponents {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            draw: Draw {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextComponents {
                style: Style {
                    size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                text: Text {
                    value: "HOOKED\nPress [R] to restart".to_string(),
                    font: asset_server.load("fonts/Chonkly.ttf"),
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::RED,
                        alignment: TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        }..Default::default(),
                    },
                },
                ..Default::default()
            });
        });
}
