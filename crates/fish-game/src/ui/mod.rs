use bevy::prelude::*;

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
        app.add_plugins((gamehud::GameHudPlugin, gameover::GameOverHudPlugin));
    }
}
