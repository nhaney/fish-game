use bevy::prelude::*;

use super::FontHandles;
use crate::leaderboard::LocalScores;
use crate::shared::game::{GameOver, GameRestarted, Score};

#[derive(Component)]
pub(super) struct ScoreText;

pub(super) fn add_score_text(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    fonts: &FontHandles,
) -> Entity {
    let root_score_node = commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Score:".to_string(),
                        TextStyle {
                            font: fonts.main_font.clone(),
                            font_size: 60.0,
                            color: Color::GREEN,
                            ..Default::default()
                        },
                    ),
                    style: Style {
                        margin: UiRect {
                            top: Val::Percent(5.0),
                            left: Val::Percent(5.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ScoreText,
            ));
        })
        .id();

    root_score_node
}

pub(super) fn update_score_text(
    score: Res<Score>,
    local_scores: Res<LocalScores>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Score: {:?}", score.count);

        if let Some(high_score) = local_scores.high_score() {
            if score.count > high_score {
                text.sections[0].style.color = Color::GOLD;
            }
        } else {
            text.sections[0].style.color = Color::GOLD;
        }
    }
}

pub(super) fn change_color_on_game_over(
    mut game_over_reader: EventReader<GameOver>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if game_over_reader.read().next().is_some() {
        for mut score_text in score_text_query.iter_mut() {
            score_text.sections[0].style.color = Color::RED;
        }
    }
}

pub(super) fn revert_color_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
) {
    if restart_reader.read().next().is_some() {
        for mut score_text in score_text_query.iter_mut() {
            score_text.sections[0].style.color = Color::GREEN;
        }
    }
}
