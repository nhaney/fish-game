use bevy::prelude::*;

use common::FontHandles;

mod common;
mod gamehud;
mod gameover;

/// Plugin that encompasses the entire UI for the game.
/// The UI includes everything that is overlaid on top of the game in its various scenes such as
/// score display, pause buttons, etc. The UI excludes sprites and the actual rendering of the game itself.
pub struct UIPlugin;

impl Plugin for UIPlugin {
    /// Builds the UI and adds the relevant entities, components and systems to the bevy app.
    fn build(&self, app: &mut App) {
        debug!("Building UI plugin...");

        app.init_resource::<FontHandles>()
            .add_plugins((gamehud::GameHudPlugin, gameover::GameOverHudPlugin));

        /*
         * OLD SYSTEMS:
        .add_systems(
            Update,
            (
                game_over::show_game_over_text,
                player::update_coundown_text_system,
                player::show_countdown_on_restart,
                player::hide_countdown_on_game_over,
                player::reposition_countdown_text_system,
                game_over::clear_game_over_message_on_restart,
                leaderboard::show_high_scores_on_score_saved,
                leaderboard::hide_high_scores_on_restart,
            )
                .in_set(stages::PrepareRenderSet),
        );
        */
    }
}
