use bevy::prelude::*;

use super::FontHandles;
use crate::leaderboard::{LocalScores, ScoreSaved};
use crate::shared::game::GameRestarted;

const SCORES_TO_SHOW: usize = 10;

#[derive(Component)]
pub(super) struct HighScoreDisplayNode;

#[derive(Debug, Resource)]
pub(super) struct HighScoreDisplay {
    // TODO: When implemented in bevy, only change visiblity on root node, for now this
    // doesn't work
    root_node: Entity,
    title_text_node: Entity,
    score_nodes: [Entity; SCORES_TO_SHOW],
}

pub(super) fn add_local_leaderboard_nodes(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    fonts: &FontHandles,
) -> Entity {
    let leaderboard_node = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::FlexStart,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    margin: UiRect {
                        top: Val::Percent(5.0),
                        bottom: Val::Percent(20.0),
                        left: Val::Percent(5.0),
                        ..Default::default()
                    },
                    /*size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),*/
                    flex_grow: 1.0,
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            HighScoreDisplayNode,
        ))
        .id();

    let mut score_nodes: [Entity; SCORES_TO_SHOW] = [leaderboard_node; SCORES_TO_SHOW];

    let title_text = commands
        .spawn((
            TextBundle {
                style: Style {
                    margin: UiRect {
                        top: Val::Percent(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::from_section(
                    "High scores:",
                    TextStyle {
                        font_size: 35.0,
                        font: fonts.main_font.clone(),
                        color: Color::BLACK,
                        ..Default::default()
                    },
                ),
                visibility: Visibility::Inherited,
                ..Default::default()
            },
            HighScoreDisplayNode,
        ))
        .id();

    for (i, score_node) in score_nodes.iter_mut().enumerate().take(SCORES_TO_SHOW) {
        *score_node = commands
            .spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect {
                            top: Val::Px(10.0),
                            bottom: Val::Px(10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text::from_section(
                        format!("{}. test", i + 1),
                        TextStyle {
                            font: fonts.main_font.clone(),
                            font_size: 25.0,
                            color: Color::BLACK,
                            ..Default::default()
                        },
                    ),
                    visibility: Visibility::Inherited,
                    ..Default::default()
                },
                HighScoreDisplayNode,
            ))
            .id()
    }

    commands
        .entity(leaderboard_node)
        .push_children(&[title_text]);

    commands
        .entity(leaderboard_node)
        .push_children(&score_nodes);

    commands.insert_resource(HighScoreDisplay {
        root_node: leaderboard_node,
        title_text_node: title_text,
        score_nodes,
    });

    leaderboard_node
}

fn change_visibility_of_scoreboard(
    is_visible: bool,
    display: &HighScoreDisplay,
    mut high_score_visibility_query: Query<&mut Visibility, With<HighScoreDisplayNode>>,
) {
    // TODO: Delete this when confirmed that it is working.

    /*
    for score_entity in display.score_nodes.iter() {
        let mut score_visibility = high_score_visibility_query.get_mut(*score_entity).unwrap();
        score_visibility.is_visible = is_visible;
    }

    let mut title_visibility = high_score_visibility_query
        .get_mut(display.title_text_node)
        .unwrap();
    title_visibility.is_visible = is_visible;
    */

    let mut container_visibilty = high_score_visibility_query
        .get_mut(display.root_node)
        .unwrap();

    container_visibilty = if is_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(super) fn show_high_scores_on_score_saved(
    mut score_saved_reader: EventReader<ScoreSaved>,
    high_score_display: Res<HighScoreDisplay>,
    local_scores: Res<LocalScores>,
    high_score_visibility_query: Query<&mut Visibility, With<HighScoreDisplayNode>>,
    mut high_score_text_query: Query<&mut Text>,
) {
    if let Some(score_saved_event) = score_saved_reader.read().next() {
        debug!(
            "Got score saved event: {:?}, displaying high scores...",
            score_saved_event
        );

        change_visibility_of_scoreboard(true, &high_score_display, high_score_visibility_query);

        for (i, score_entity) in high_score_display.score_nodes.iter().enumerate() {
            let mut high_score_text = high_score_text_query.get_mut(*score_entity).unwrap();

            if local_scores.scores.len() > i {
                high_score_text.sections[0].value =
                    format!("{}. {}", i + 1, local_scores.scores[i]);
                if i == 0 {
                    high_score_text.style.color = Color::GOLD;
                } else if i == 1 {
                    high_score_text.style.color = Color::SILVER;
                } else if i == 2 {
                    // bronze color
                    high_score_text.style.color = Color::hex("cd7f32").unwrap();
                }
            } else {
                high_score_text.sections[0].value = "".to_string();
            }
        }
    }
}

pub(super) fn hide_high_scores_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    high_score_display: Res<HighScoreDisplay>,
    high_score_visibility_query: Query<&mut Visibility, With<HighScoreDisplayNode>>,
) {
    if restart_reader.next().is_some() {
        change_visibility_of_scoreboard(false, &high_score_display, high_score_visibility_query);
    }
}
