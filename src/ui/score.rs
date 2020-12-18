use bevy::prelude::*;

use crate::leaderboard::LocalScores;
use crate::shared::game::{GameOver, GameRestarted, Score};

pub(super) struct ScoreText;

pub(super) fn add_score_text(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let root_score_node = commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    text: Text {
                        value: "Score:".to_string(),
                        font: asset_server.load("fonts/Chonkly.ttf"),
                        style: TextStyle {
                            font_size: 60.0,
                            color: Color::GREEN,
                            ..Default::default()
                        },
                    },
                    style: Style {
                        margin: Rect {
                            top: Val::Percent(5.0),
                            left: Val::Percent(5.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(ScoreText);
        })
        .current_entity()
        .unwrap();

    root_score_node
}

pub(super) fn update_score_text(
    score: Res<Score>,
    local_scores: Res<LocalScores>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    for mut text in query.iter_mut() {
        text.value = format!("Score: {:?}", score.count);

        if let Some(high_score) = local_scores.high_score() {
            if score.count > high_score {
                text.style.color = Color::GOLD;
            }
        } else {
            text.style.color = Color::GOLD;
        }
    }
}

pub(super) fn change_color_on_game_over(
    game_over_events: Res<Events<GameOver>>,
    mut game_over_reader: Local<EventReader<GameOver>>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if let Some(_) = game_over_reader.earliest(&game_over_events) {
        for mut score_text in score_text_query.iter_mut() {
            score_text.style.color = Color::RED;
        }
    }
}

pub(super) fn revert_color_on_restart(
    restart_events: Res<Events<GameRestarted>>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if let Some(_) = restart_reader.earliest(&restart_events) {
        for mut score_text in score_text_query.iter_mut() {
            score_text.style.color = Color::GREEN;
        }
    }
}
