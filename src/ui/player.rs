use bevy::prelude::*;
use bevy::render::{camera::Camera, render_graph::base::camera::CAMERA_2D};

use super::FontHandles;
use crate::player::attributes::{HungerCountdown, Player};
use crate::shared::{
    collision::Collider,
    game::{GameOver, GameRestarted},
};

pub(super) struct PlayerCountdownText;

// TODO: When bevy allows text to be a child of a parent entity, change this to not use the UI system.
pub(super) fn add_countdown_text(commands: &mut Commands, fonts: Res<FontHandles>) {
    commands
        .spawn(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(100.0),
                    left: Val::Px(100.0),
                    right: Val::Px(100.0),
                    bottom: Val::Px(100.0),
                },
                ..Default::default()
            },
            text: Text {
                value: "30.0".to_string(),
                font: fonts.main_font.clone(),
                style: TextStyle {
                    font_size: 30.0,
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(PlayerCountdownText);
}

//updates countdown text
pub(super) fn update_coundown_text_system(
    mut text_query: Query<&mut Text, With<PlayerCountdownText>>,
    player_query: Query<&HungerCountdown, With<Player>>,
) {
    for mut text in text_query.iter_mut() {
        for hunger_countdown in player_query.iter() {
            text.value = format!("{:.1}", hunger_countdown.time_left);
            if hunger_countdown.time_left < 5.0 {
                text.style.color = Color::RED;
            } else {
                text.style.color = Color::PINK;
            }
        }
    }
}

// repositions the countdown text to line up above player - when it is available, switch to canvas drawing
pub(super) fn reposition_countdown_text_system(
    windows: Res<Windows>,
    mut text_query: Query<&mut Style, With<PlayerCountdownText>>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    camera_query: Query<(&Camera, &Transform)>,
) {
    let (_, camera_transform) = camera_query
        .iter()
        .find(|(camera, _)| camera.name == Some(CAMERA_2D.to_string()))
        .unwrap();

    let window = windows.get_primary().unwrap();
    let h = (window.height() / 2.0) as f32;
    let w = (window.width() / 2.0) as f32;

    for mut style in text_query.iter_mut() {
        for (player_transform, player_size) in player_query.iter() {
            let player_window_pos =
                (player_transform.translation / camera_transform.scale) + Vec3::new(w, h, 0.0);

            let scaled_sprite_size = Vec2::new(
                player_size.width / camera_transform.scale.x,
                player_size.height / camera_transform.scale.y,
            );

            style.position.left = Val::Px(player_window_pos.x - scaled_sprite_size.x / 2.0);
            style.position.right = Val::Px(player_window_pos.x + scaled_sprite_size.x / 2.0);
            style.position.bottom = Val::Px(player_window_pos.y + (scaled_sprite_size.y * 1.5));
        }
    }
}

pub(super) fn hide_countdown_on_game_over(
    game_over_events: Res<Events<GameOver>>,
    mut game_over_reader: Local<EventReader<GameOver>>,
    mut countdown_text_query: Query<&mut Visible, With<PlayerCountdownText>>,
) {
    if game_over_reader.earliest(&game_over_events).is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            countdown_text_visiblity.is_visible = false;
        }
    }
}

pub(super) fn show_countdown_on_restart(
    restart_events: Res<Events<GameRestarted>>,
    mut restart_reader: Local<EventReader<GameRestarted>>,
    mut countdown_text_query: Query<&mut Visible, With<PlayerCountdownText>>,
) {
    if restart_reader.earliest(&restart_events).is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            countdown_text_visiblity.is_visible = true;
        }
    }
}
