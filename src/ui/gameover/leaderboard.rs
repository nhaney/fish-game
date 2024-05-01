use bevy::prelude::*;

use crate::leaderboard::{LocalScores, ScoreSaved};
use crate::shared::game::GameRestarted;
use crate::ui::common::FontHandles;

#[derive(Resource)]
pub(super) struct HighScoreDisplayConfig {
    pub scores_to_show: usize,
}

#[derive(Component)]
pub(super) struct HighScoreDisplayRootNode {
    score_nodes: Vec<Entity>,
}

/// Spawns a node that contains the leaderboard to display.
pub(super) fn spawn_leaderboard_display(
    mut commands: Commands,
    config: Res<HighScoreDisplayConfig>,
    fonts: Res<FontHandles>,
) {
    let mut score_nodes = Vec::new();

    let leaderboard_root_node = commands
        .spawn((NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceEvenly,
                flex_grow: 1.,
                flex_shrink: 1.,
                flex_basis: Val::Px(0.),
                /*
                margin: UiRect {
                    top: Val::Percent(5.0),
                    bottom: Val::Percent(20.0),
                    left: Val::Percent(5.0),
                    ..Default::default()
                },*/
                /*size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),*/
                //flex_grow: 1.0,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        },))
        .with_children(|builder| {
            // Spawn leaderboard title.
            builder.spawn(TextBundle {
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
            });

            for i in 0..config.scores_to_show {
                let score_node = builder
                    .spawn((TextBundle {
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
                        ..Default::default()
                    },))
                    .id();
                score_nodes.push(score_node);
            }
        })
        .id();

    // Add marker component that keeps track of the score ui nodes in order.
    commands
        .entity(leaderboard_root_node)
        .insert(HighScoreDisplayRootNode { score_nodes });
}

fn change_visibility_of_scoreboard(
    is_visible: bool,
    high_score_visibility_query: &mut Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
) {
    let (mut container_visibility, _) = high_score_visibility_query
        .get_single_mut()
        .expect("Could not find leaderboard root node.");

    *container_visibility = if is_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(super) fn show_high_scores_on_score_saved(
    mut score_saved_reader: EventReader<ScoreSaved>,
    local_scores: Res<LocalScores>,
    mut high_score_visibility_query: Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
    mut high_score_text_query: Query<&mut Text>,
) {
    if let Some(score_saved_event) = score_saved_reader.read().next() {
        debug!(
            "Got score saved event: {:?}, displaying high scores...",
            score_saved_event
        );

        change_visibility_of_scoreboard(true, &mut high_score_visibility_query);

        let score_nodes = &high_score_visibility_query
            .get_single()
            .expect("Could not find leaderboard root node to display its score nodes.")
            .1
            .score_nodes;

        for (i, score_entity) in score_nodes.iter().enumerate() {
            let mut high_score_text = high_score_text_query.get_mut(*score_entity).unwrap();

            if local_scores.scores.len() > i {
                high_score_text.sections[0].value =
                    format!("{}. {}", i + 1, local_scores.scores[i]);
                if i == 0 {
                    high_score_text.sections[0].style.color = Color::GOLD;
                } else if i == 1 {
                    high_score_text.sections[0].style.color = Color::SILVER;
                } else if i == 2 {
                    // bronze color
                    high_score_text.sections[0].style.color = Color::hex("cd7f32").unwrap();
                }
            } else {
                high_score_text.sections[0].value = "".to_string();
            }
        }
    }
}

pub(super) fn hide_high_scores_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut high_score_visibility_query: Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
) {
    if restart_reader.read().next().is_some() {
        change_visibility_of_scoreboard(false, &mut high_score_visibility_query);
    }
}
