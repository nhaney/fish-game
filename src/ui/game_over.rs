use bevy::prelude::*;

use crate::player::events::{PlayerBonked, PlayerHooked, PlayerStarved};
use crate::shared::game::GameRestarted;

pub(super) struct GameOverText;
pub(super) struct RestartText;

pub(super) fn add_game_over_text(
    commands: &mut Commands,
    asset_server: &AssetServer,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let root_game_over_node = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        ..Default::default()
                    },
                    text: Text {
                        value: "HOOKED".to_string(),
                        font: asset_server.load("fonts/Chonkly.ttf"),
                        style: TextStyle {
                            font_size: 100.0,
                            color: Color::RED,
                            alignment: TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                            ..Default::default()
                        },
                    },
                    visible: Visible {
                        is_visible: false,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(GameOverText)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
                            style: Style {
                                position: Rect {
                                    bottom: Val::Px(-100.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Press [R] to restart".to_string(),
                                font: asset_server.load("fonts/Chonkly.ttf"),
                                style: TextStyle {
                                    font_size: 50.0,
                                    color: Color::RED,
                                    alignment: TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    },
                                    ..Default::default()
                                },
                            },
                            visible: Visible {
                                is_visible: false,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with(RestartText);
                });
        })
        .current_entity()
        .unwrap();

    root_game_over_node
}

pub(super) fn show_game_over_text(
    mut player_hooked_reader: Local<EventReader<PlayerHooked>>,
    player_hooked_events: Res<Events<PlayerHooked>>,
    mut player_starved_reader: Local<EventReader<PlayerStarved>>,
    player_starved_events: Res<Events<PlayerStarved>>,
    mut player_bonked_reader: Local<EventReader<PlayerBonked>>,
    player_bonked_events: Res<Events<PlayerBonked>>,
    mut game_over_text_query: Query<
        (&mut Visible, &mut Text),
        (With<GameOverText>, With<Children>),
    >,
    mut restart_text_query: Query<
        &mut Visible,
        (Without<GameOverText>, With<Parent>, With<RestartText>),
    >,
) {
    let mut game_over_message = "".to_string();
    if let Some(_) = player_hooked_reader.earliest(&player_hooked_events) {
        game_over_message = "HOOKED!".to_string();
    }

    if let Some(_) = player_bonked_reader.earliest(&player_bonked_events) {
        game_over_message = "BONKED!".to_string();
    }

    if let Some(_) = player_starved_reader.earliest(&player_starved_events) {
        game_over_message = "STARVED".to_string();
    }

    if game_over_message != "".to_string() {
        for (mut game_over_draw, mut game_over_text) in game_over_text_query.iter_mut() {
            game_over_text.value = game_over_message.clone();
            game_over_draw.is_visible = true;
        }

        for mut restart_text_draw in restart_text_query.iter_mut() {
            restart_text_draw.is_visible = true;
        }
    }
}

pub(super) fn clear_game_over_message_on_restart(
    restart_events: Res<Events<GameRestarted>>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    mut game_over_text_query: Query<&mut Visible, (With<GameOverText>, With<Children>)>,
    mut restart_text_query: Query<
        &mut Visible,
        (Without<GameOverText>, With<Parent>, With<RestartText>),
    >,
) {
    if let Some(_) = restart_reader.earliest(&restart_events) {
        println!("Clearing game over text after game was restarted.");
        for mut game_over_draw in game_over_text_query.iter_mut() {
            game_over_draw.is_visible = false;
        }

        for mut restart_text_draw in restart_text_query.iter_mut() {
            restart_text_draw.is_visible = false;
        }
    }
}
