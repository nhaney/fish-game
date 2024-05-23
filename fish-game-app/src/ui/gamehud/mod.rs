use bevy::prelude::*;

mod pause;
mod score;

use pause::PauseButton;
use score::ScoreText;

use crate::shared::stages;

pub struct GameHudPlugin;

impl Plugin for GameHudPlugin {
    /// Builds the UI and adds the relevant entities, components and systems to the bevy app.
    fn build(&self, app: &mut App) {
        debug!("Building game GameHudPlugin...");
        app.init_resource::<pause::PauseButtonMaterials>()
            .add_systems(Startup, (compose_game_hud,))
            .add_systems(
                Startup,
                (score::setup_score_display, pause::setup_pause_button).before(compose_game_hud),
            )
            .add_systems(
                Update,
                (pause::pause_button_system,).in_set(stages::HandleEventsSet),
            )
            .add_systems(
                Update,
                (
                    score::update_score_text,
                    score::change_color_on_game_over,
                    score::revert_color_on_restart,
                    pause::reset_pause_button_on_restart,
                ),
            );
    }
}

#[derive(Component)]
struct GameHudRoot;

/// Sets up the root node that contains the flexbox that all game hud elements will be children of.
fn compose_game_hud(
    mut commands: Commands,
    score_root_query: Query<Entity, With<ScoreText>>,
    pause_root_query: Query<Entity, With<PauseButton>>,
) {
    let score_text_root_node = score_root_query
        .get_single()
        .expect("Could not find score text root node to compose into game HUD");

    let pause_root_node = pause_root_query
        .get_single()
        .expect("Could not find game over message root node to compose into gameover HUD");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            },
            GameHudRoot,
        ))
        .push_children(&[score_text_root_node, pause_root_node]);
}
