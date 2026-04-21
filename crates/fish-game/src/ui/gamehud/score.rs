use bevy::prelude::*;

use crate::core_adapter::CoreState;
use crate::leaderboard::LocalScores;
use crate::shared::game::{GameOver, GameRestarted};
use crate::shared::render::FontHandles;

#[derive(Component)]
pub(super) struct ScoreText;

pub fn setup_score_display(mut commands: Commands, fonts: Res<FontHandles>) {
    commands.spawn((
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
                    left: Val::Percent(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ScoreText,
    ));
}

pub(super) fn update_score_text(
    core: Res<CoreState>,
    local_scores: Res<LocalScores>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    let current = core.state.score.count;
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Score: {:?}", current);

        if let Some(high_score) = local_scores.high_score() {
            if current > high_score {
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
