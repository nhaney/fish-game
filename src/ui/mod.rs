use bevy::prelude::*;

use crate::shared::stages;

mod game_over;
mod leaderboard;
mod pause;
mod player;
mod score;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building UI plugin...");
        // pause button sprite materials
        app.init_resource::<pause::PauseButtonMaterials>()
            // Startup systems - create ui elements
            .add_startup_system(player::add_countdown_text.system())
            .add_startup_system(setup_ui.system())
            // Systems that react to input
            .add_system_to_stage(stages::HANDLE_EVENTS, pause::pause_button_system.system())
            // Systems that update ui based on current state of the game before rendering
            // Note: These must be in update because UI updates happen before POST_UPDATE
            .add_system_to_stage(stages::PREPARE_RENDER, score::update_score_text.system())
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                score::change_color_on_game_over.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                score::revert_color_on_restart.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                game_over::show_game_over_text.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                player::update_coundown_text_system.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                player::show_countdown_on_restart.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                player::hide_countdown_on_game_over.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                player::reposition_countdown_text_system.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                game_over::clear_game_over_message_on_restart.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                pause::reset_pause_button_on_restart.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                leaderboard::show_high_scores_on_score_saved.system(),
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                leaderboard::hide_high_scores_on_restart.system(),
            );
    }
}

fn setup_ui(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    pause_button_materials: Res<pause::PauseButtonMaterials>,
) {
    commands.spawn(CameraUiBundle::default());

    let root_ui_node = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .current_entity()
        .unwrap();

    println!("Adding score text to UI...");
    let score_node = score::add_score_text(commands, &asset_server, &mut materials);

    println!("Adding blank game over text to UI...");
    let game_over_node = game_over::add_game_over_text(commands, &asset_server, &mut materials);

    println!("Adding pause button to UI...");
    let pause_button_node =
        pause::add_pause_button(commands, &pause_button_materials, &mut materials);

    println!("Adding high scores to UI...");
    let leaderboard_node =
        leaderboard::add_local_leaderboard_nodes(commands, &asset_server, &mut materials);
    commands.push_children(score_node, &[leaderboard_node]);

    commands.push_children(
        root_ui_node,
        &[score_node, game_over_node, pause_button_node],
    );
}
