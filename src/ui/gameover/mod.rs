use bevy::prelude::*;

mod leaderboard;
mod message;

use leaderboard::{HighScoreDisplayConfig, HighScoreDisplayRootNode};
use message::GameOverMessageRootNode;

use crate::shared::stages;

pub struct GameOverHudPlugin;

impl Plugin for GameOverHudPlugin {
    /// Builds the UI and adds the relevant entities, components and systems to the bevy app.
    fn build(&self, app: &mut App) {
        debug!("Building game GameOverHudPlugin...");

        app.insert_resource(HighScoreDisplayConfig { scores_to_show: 10 })
            .add_systems(Startup, (compose_gameover_hud,))
            .add_systems(
                Startup,
                (
                    leaderboard::spawn_leaderboard_display,
                    message::spawn_gameover_message_display,
                )
                    .before(compose_gameover_hud),
            )
            .add_systems(
                Update,
                (
                    message::show_game_over_text,
                    message::clear_game_over_message_on_restart,
                    leaderboard::show_high_scores_on_score_saved,
                    leaderboard::hide_high_scores_on_restart,
                )
                    .in_set(stages::PrepareRenderSet),
            );
    }
}

#[derive(Component)]
struct GameOverHudRoot;

/// Composes the root node used by the game over HUD.
fn compose_gameover_hud(
    mut commands: Commands,
    leaderboard_root_query: Query<Entity, With<HighScoreDisplayRootNode>>,
    message_root_query: Query<Entity, With<GameOverMessageRootNode>>,
) {
    let leaderboard_root_node = leaderboard_root_query
        .get_single()
        .expect("Could not find leaderboard root node to compose into gameover HUD");

    let game_over_message_root_node = message_root_query
        .get_single()
        .expect("Could not find game over message root node to compose into gameover HUD");

    let dummy = commands
        .spawn(NodeBundle {
            style: Style {
                flex_grow: 1.,
                ..default()
            },
            background_color: BackgroundColor(Color::YELLOW),
            visibility: Visibility::Visible,
            ..default()
        })
        .id();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..Default::default()
            },
            GameOverHudRoot,
        ))
        .push_children(&[leaderboard_root_node, game_over_message_root_node, dummy]);
}
