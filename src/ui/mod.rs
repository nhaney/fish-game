use bevy::prelude::*;

use crate::shared::stages;

mod game_over;
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
            .add_startup_system(setup)
            .add_startup_system(player::add_countdown_text)
            // Systems that react to input
            .add_system_to_stage(stages::HANDLE_EVENTS, pause::pause_button_system)
            // Systems that update ui based on current state of the game before rendering
            // Note: These must be in update because UI updates happen before POST_UPDATE
            .add_system_to_stage(stages::PREPARE_RENDER, score::update_score_text)
            .add_system_to_stage(stages::PREPARE_RENDER, game_over::show_game_over_text)
            .add_system_to_stage(stages::PREPARE_RENDER, player::update_coundown_text_system)
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                player::reposition_countdown_text_system,
            )
            .add_system_to_stage(
                stages::PREPARE_RENDER,
                game_over::clear_game_over_message_on_restart,
            )
            .add_system_to_stage(stages::PREPARE_RENDER, pause::reset_pause_button_on_restart);
    }
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    pause_button_materials: Res<pause::PauseButtonMaterials>,
) {
    commands
        .spawn(CameraUiBundle::default())
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            println!("Adding score text to UI...");
            score::add_score_text(parent, &asset_server, &mut materials);
            println!("Adding blank game over text to UI...");
            game_over::add_game_over_text(parent, &asset_server, &mut materials);
            println!("Adding pause button to UI...");
            pause::add_pause_button(parent, &pause_button_materials, &mut materials);
        });
}
