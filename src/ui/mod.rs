use bevy::prelude::*;

mod game_over;
mod pause;
mod player;
mod score;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building UI plugin...");
        app.add_startup_system(setup)
            // score text
            .add_system(score::update_score_text)
            // pause button
            .init_resource::<pause::PauseButtonMaterials>()
            .add_system(pause::pause_button_system)
            // game over text
            .add_system(game_over::show_game_over_text)
            // player countdown text
            // TODO: Clean this up, put it in player plugin?
            .add_startup_system(player::add_countdown_text)
            .add_system(player::update_coundown_text_system)
            .add_system(player::reposition_countdown_text_system)
            .add_system_to_stage(stage::LAST, game_over::clear_game_over_message_on_restart)
            .add_system_to_stage(stage::LAST, pause::reset_pause_button_on_restart);
    }
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    pause_button_materials: Res<pause::PauseButtonMaterials>,
) {
    commands
        .spawn(UiCameraBundle::default())
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
