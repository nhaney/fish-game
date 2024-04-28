use bevy::prelude::*;

use crate::shared::stages;

mod game_over;
mod leaderboard;
mod pause;
mod player;
mod score;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        debug!("Building UI plugin...");
        // pause button sprite materials
        app.init_resource::<pause::PauseButtonMaterials>()
            .init_resource::<FontHandles>()
            // Startup systems - create ui elements
            .add_systems(Startup, (player::add_countdown_text, setup_ui))
            // Systems that react to input
            .add_systems(
                Update,
                (pause::pause_button_system,).in_set(stages::HandleEventsSet),
            )
            // Systems that update ui based on current state of the game before rendering
            // Note: These must be in update because UI updates happen before POST_UPDATE
            // TODO: update comment above to me more accurate when I know how this works ^
            .add_systems(
                Update,
                (
                    score::update_score_text,
                    score::change_color_on_game_over,
                    score::revert_color_on_restart,
                    game_over::show_game_over_text,
                    player::update_coundown_text_system,
                    player::show_countdown_on_restart,
                    player::hide_countdown_on_game_over,
                    player::reposition_countdown_text_system,
                    game_over::clear_game_over_message_on_restart,
                    pause::reset_pause_button_on_restart,
                    leaderboard::show_high_scores_on_score_saved,
                    leaderboard::hide_high_scores_on_restart,
                )
                    .in_set(stages::PrepareRenderSet),
            );
    }
}

fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    fonts: Res<FontHandles>,
    pause_button_materials: Res<pause::PauseButtonMaterials>,
) {
    // TODO: do we need this still?
    // commands.spawn(Camera2dBundle::default());

    let root_ui_node = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    debug!("Adding score text to UI...");
    let score_node = score::add_score_text(&mut commands, &mut materials, &fonts);

    debug!("Adding blank game over text to UI...");
    let game_over_node = game_over::add_game_over_text(&mut commands, &mut materials, &fonts);

    debug!("Adding pause button to UI...");
    let pause_button_node =
        pause::add_pause_button(&mut commands, &pause_button_materials, &mut materials);

    debug!("Adding high scores to UI...");
    let leaderboard_node =
        leaderboard::add_local_leaderboard_nodes(&mut commands, &mut materials, &fonts);

    commands
        .entity(score_node)
        .push_children(&[leaderboard_node]);

    commands
        .entity(root_ui_node)
        .push_children(&[score_node, game_over_node, pause_button_node]);
}

#[derive(Debug, Clone, Resource)]
pub(super) struct FontHandles {
    main_font: Handle<Font>,
}

impl FromWorld for FontHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        debug!("Loading fonts...");
        Self {
            main_font: asset_server.load("fonts/Chonkly.ttf"),
        }
    }
}
