use bevy::prelude::*;
use bevy::render::camera::Camera;

use super::FontHandles;
use crate::player::attributes::{HungerCountdown, Player};
use crate::shared::{
    collision::Collider,
    game::{GameOver, GameRestarted},
};

#[derive(Component)]
pub(super) struct PlayerCountdownText;

// TODO: When bevy allows text to be a child of a parent entity, change this to not use the UI system.
pub(super) fn add_countdown_text(mut commands: Commands, fonts: Res<FontHandles>) {
    commands.spawn((
        TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(100.0),
                right: Val::Px(100.0),
                bottom: Val::Px(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            text: Text::from_section(
                "30.0".to_string(),
                TextStyle {
                    font: fonts.main_font.clone(),
                    font_size: 30.0,
                    ..Default::default()
                },
            )
            .with_justify(JustifyText::Center),
            ..default()
        },
        PlayerCountdownText,
    ));
}

//updates countdown text
pub(super) fn update_coundown_text_system(
    mut text_query: Query<&mut Text, With<PlayerCountdownText>>,
    player_query: Query<&HungerCountdown, With<Player>>,
) {
    for mut text in text_query.iter_mut() {
        for hunger_countdown in player_query.iter() {
            text.sections[0].value = format!("{:.1}", hunger_countdown.time_left);
            if hunger_countdown.time_left < 5.0 {
                text.sections[0].style.color = Color::RED;
            } else {
                text.sections[0].style.color = Color::PINK;
            }
        }
    }
}

// repositions the countdown text to line up above player
// TODO: switch to canvas drawing Text2d?
pub(super) fn reposition_countdown_text_system(
    windows: Query<&Window>,
    mut text_query: Query<&mut Style, With<PlayerCountdownText>>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    camera_query: Query<(&Camera, &Transform)>,
) {
    /*
    let (_, camera_transform) = camera_query
        .iter()
        .find(|(camera, _)| camera.name == Some(CAMERA_2D.to_string()))
        .unwrap();
    */

    let (_, camera_transform) = camera_query.single();
    let window = windows.single();

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

            style.left = Val::Px(player_window_pos.x - scaled_sprite_size.x / 2.0);
            style.right = Val::Px(player_window_pos.x + scaled_sprite_size.x / 2.0);
            style.bottom = Val::Px(player_window_pos.y + (scaled_sprite_size.y * 1.5));
        }
    }
}

pub(super) fn hide_countdown_on_game_over(
    mut game_over_reader: EventReader<GameOver>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if game_over_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Visible;
        }
    }
}

pub(super) fn show_countdown_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut countdown_text_query: Query<&mut Visibility, With<PlayerCountdownText>>,
) {
    if restart_reader.read().next().is_some() {
        for mut countdown_text_visiblity in countdown_text_query.iter_mut() {
            *countdown_text_visiblity = Visibility::Visible;
        }
    }
}
