use bevy::prelude::*;

use crate::shared::game::Score;

pub(super) struct ScoreText;

pub(super) fn add_score_text(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    materials: &mut Assets<ColorMaterial>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                padding: Rect {
                    left: Val::Px(50.0),
                    right: Val::Px(50.0),
                    top: Val::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "Score:".to_string(),
                        font: asset_server.load("fonts/Chonkly.ttf"),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::GREEN,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(ScoreText);
        });
}

pub(super) fn update_score_text(score: Res<Score>, mut query: Query<(&mut Text, &ScoreText)>) {
    for (mut text, _) in query.iter_mut() {
        text.value = format!("Score: {:?}", score.count);
    }
}
