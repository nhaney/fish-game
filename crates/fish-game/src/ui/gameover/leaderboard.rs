use bevy::color::palettes::css::{BLACK, GOLD, SILVER};
use bevy::prelude::*;

use crate::leaderboard::{LocalScores, ScoreSaved};
use crate::shared::game::GameRestarted;
use crate::shared::render::FontHandles;

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
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::SpaceEvenly,
                flex_grow: 1.,
                flex_shrink: 1.,
                flex_basis: Val::Px(0.),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|builder| {
            builder.spawn((
                Text::new("High scores:"),
                TextFont {
                    font_size: 35.0,
                    font: fonts.main_font.clone(),
                    ..Default::default()
                },
                TextColor(Color::from(BLACK)),
                Node {
                    margin: UiRect {
                        top: Val::Percent(10.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Visibility::Inherited,
            ));

            for i in 0..config.scores_to_show {
                let score_node = builder
                    .spawn((
                        Text::new(format!("{}. test", i + 1)),
                        TextFont {
                            font: fonts.main_font.clone(),
                            font_size: 25.0,
                            ..Default::default()
                        },
                        TextColor(Color::from(BLACK)),
                        Node {
                            margin: UiRect {
                                top: Val::Px(10.0),
                                bottom: Val::Px(10.0),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ))
                    .id();
                score_nodes.push(score_node);
            }
        })
        .id();

    commands
        .entity(leaderboard_root_node)
        .insert(HighScoreDisplayRootNode { score_nodes });
}

fn change_visibility_of_scoreboard(
    is_visible: bool,
    high_score_visibility_query: &mut Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
) {
    let (mut container_visibility, _) = high_score_visibility_query
        .single_mut()
        .expect("Could not find leaderboard root node.");

    *container_visibility = if is_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

pub(super) fn show_high_scores_on_score_saved(
    mut score_saved_reader: MessageReader<ScoreSaved>,
    local_scores: Res<LocalScores>,
    mut high_score_visibility_query: Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
    mut high_score_text_query: Query<(&mut Text, &mut TextColor)>,
) {
    if let Some(score_saved_event) = score_saved_reader.read().next() {
        debug!(
            "Got score saved event: {:?}, displaying high scores...",
            score_saved_event
        );

        change_visibility_of_scoreboard(true, &mut high_score_visibility_query);

        let score_nodes = &high_score_visibility_query
            .single()
            .expect("Could not find leaderboard root node to display its score nodes.")
            .1
            .score_nodes;

        for (i, score_entity) in score_nodes.iter().enumerate() {
            let (mut high_score_text, mut high_score_color) =
                high_score_text_query.get_mut(*score_entity).unwrap();

            if local_scores.scores.len() > i {
                **high_score_text = format!("{}. {}", i + 1, local_scores.scores[i]);
                if i == 0 {
                    high_score_color.0 = Color::from(GOLD);
                } else if i == 1 {
                    high_score_color.0 = Color::from(SILVER);
                } else if i == 2 {
                    // bronze color
                    high_score_color.0 = Srgba::hex("cd7f32").map(Color::from).unwrap();
                }
            } else {
                **high_score_text = "".to_string();
            }
        }
    }
}

pub(super) fn hide_high_scores_on_restart(
    mut restart_reader: MessageReader<GameRestarted>,
    mut high_score_visibility_query: Query<(&mut Visibility, &HighScoreDisplayRootNode)>,
) {
    if restart_reader.read().next().is_some() {
        change_visibility_of_scoreboard(false, &mut high_score_visibility_query);
    }
}
