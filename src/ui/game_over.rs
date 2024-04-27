use bevy::prelude::*;

use super::FontHandles;
use crate::player::events::{PlayerBonked, PlayerHooked, PlayerStarved};
use crate::shared::game::GameRestarted;

#[derive(Component)]
pub(super) struct GameOverText;

#[derive(Component)]
pub(super) struct RestartText;

pub(super) fn add_game_over_text(
    commands: &mut Commands,
    materials: &mut Assets<ColorMaterial>,
    fonts: &FontHandles,
) -> Entity {
    let root_game_over_node = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    TextBundle {
                        style: Style {
                            ..Default::default()
                        },
                        text: Text::from_section(
                            "HOOKED!".to_string(),
                            TextStyle {
                                font_size: 100.0,
                                font: fonts.main_font.clone(),
                                color: Color::RED,
                            },
                        ),
                        visibility: Visibility::Inherited,
                        ..Default::default()
                    },
                    GameOverText,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                bottom: Val::Px(-100.0),
                                ..Default::default()
                            },
                            text: Text::from_section(
                                "Press [R] to restart".to_string(),
                                TextStyle {
                                    font_size: 50.0,
                                    font: fonts.main_font.clone(),
                                    color: Color::RED,
                                },
                            ),
                            visibility: Visibility::Inherited..Default::default(),
                            ..Default::default()
                        },
                        RestartText,
                    ));
                });
        })
        .id();

    root_game_over_node
}

// TODO: Refactor to use types to reduce complexity
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) fn show_game_over_text(
    mut player_hooked_reader: EventReader<PlayerHooked>,
    mut player_starved_reader: EventReader<PlayerStarved>,
    mut player_bonked_reader: EventReader<PlayerBonked>,
    mut game_over_text_query: Query<
        (&mut Visibility, &mut Text),
        (With<GameOverText>, With<Children>),
    >,
    /*
    mut restart_text_query: Query<
        &mut Visibility,
        (Without<GameOverText>, With<Parent>, With<RestartText>),
    >,*/
) {
    let mut game_over_message = "".to_string();
    if player_hooked_reader.read().next().is_some() {
        game_over_message = "HOOKED!".to_string();
    }

    if player_bonked_reader.read().next().is_some() {
        game_over_message = "BONKED!".to_string();
    }

    if player_starved_reader.read().next().is_some() {
        game_over_message = "STARVED!".to_string();
    }

    if game_over_message != *"" {
        for (mut game_over_draw, mut game_over_text) in game_over_text_query.iter_mut() {
            game_over_text.sections[0].value = game_over_message.clone();
            game_over_draw = Visibility::Visible;
        }

        /* TODO: Remove this if the restart text is drawn. I think it should because it inherits
         * visibility
        for mut restart_text_draw in restart_text_query.iter_mut() {
            restart_text_draw.is_visible = true;
        }
        */
    }
}

// TODO: Refactor to use types to reduce complexity
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub(super) fn clear_game_over_message_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut game_over_text_query: Query<&mut Visibility, (With<GameOverText>, With<Children>)>,
    /*
    mut restart_text_query: Query<
        &mut Visible,
        (Without<GameOverText>, With<Parent>, With<RestartText>),
    >,*/
) {
    if restart_reader.read().next().is_some() {
        debug!("Clearing game over text after game was restarted.");
        for mut game_over_draw in game_over_text_query.iter_mut() {
            game_over_draw = Visibility::Hidden;
        }

        /*
        for mut restart_text_draw in restart_text_query.iter_mut() {
            restart_text_draw.is_visible = false;
        }*/
    }
}
