use bevy::color::palettes::css::{GOLD, GREEN, RED};
use bevy::prelude::*;

use crate::core_adapter::CoreState;
use crate::leaderboard::LocalScores;
use crate::shared::game::{GameOver, GameRestarted};
use crate::shared::render::FontHandles;

#[derive(Component)]
pub(super) struct ScoreText;

pub fn setup_score_display(mut commands: Commands, fonts: Res<FontHandles>) {
    commands.spawn((
        Text::new("Score:"),
        TextFont {
            font: fonts.main_font.clone(),
            font_size: 60.0,
            ..Default::default()
        },
        TextColor(Color::from(GREEN)),
        Node {
            margin: UiRect {
                left: Val::Percent(5.0),
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
    mut query: Query<(&mut Text, &mut TextColor), With<ScoreText>>,
) {
    let current = core.state.score.count;
    for (mut text, mut color) in query.iter_mut() {
        **text = format!("Score: {:?}", current);

        if let Some(high_score) = local_scores.high_score() {
            if current > high_score {
                color.0 = Color::from(GOLD);
            }
        } else {
            color.0 = Color::from(GOLD);
        }
    }
}

pub(super) fn change_color_on_game_over(
    mut game_over_reader: MessageReader<GameOver>,
    mut score_text_query: Query<&mut TextColor, With<ScoreText>>,
) {
    if game_over_reader.read().next().is_some() {
        for mut color in score_text_query.iter_mut() {
            color.0 = Color::from(RED);
        }
    }
}

pub(super) fn revert_color_on_restart(
    mut restart_reader: MessageReader<GameRestarted>,
    mut score_text_query: Query<&mut TextColor, With<ScoreText>>,
) {
    if restart_reader.read().next().is_some() {
        for mut color in score_text_query.iter_mut() {
            color.0 = Color::from(GREEN);
        }
    }
}
