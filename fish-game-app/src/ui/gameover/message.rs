use bevy::prelude::*;

use crate::player::events::{PlayerBonked, PlayerHooked, PlayerStarved};
use crate::shared::game::GameRestarted;
use crate::shared::render::FontHandles;

#[derive(Component)]
pub(super) struct GameOverMessageRootNode;

#[derive(Component)]
pub(super) struct GameOverText;

#[derive(Component)]
struct RestartText;

pub(super) fn spawn_gameover_message_display(mut commands: Commands, fonts: Res<FontHandles>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_grow: 1.,
                    flex_shrink: 1.,
                    ..Default::default()
                },
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            GameOverMessageRootNode,
        ))
        .with_children(|builder| {
            // TODO: Can we use 1 text section here?
            builder.spawn((
                TextBundle {
                    text: Text::from_section(
                        "HOOKED!".to_string(),
                        TextStyle {
                            font_size: 100.0,
                            font: fonts.main_font.clone(),
                            color: Color::RED,
                        },
                    )
                    .with_justify(JustifyText::Center),
                    ..Default::default()
                },
                GameOverText,
            ));
            builder.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Press [R] to restart".to_string(),
                        TextStyle {
                            font_size: 50.0,
                            font: fonts.main_font.clone(),
                            color: Color::RED,
                        },
                    )
                    .with_justify(JustifyText::Center),
                    ..Default::default()
                },
                RestartText,
            ));
        });
}

pub(super) fn show_game_over_text(
    mut player_hooked_reader: EventReader<PlayerHooked>,
    mut player_starved_reader: EventReader<PlayerStarved>,
    mut player_bonked_reader: EventReader<PlayerBonked>,
    mut game_over_text_query: Query<&mut Text, With<GameOverText>>,
    mut game_over_root_node_query: Query<&mut Visibility, With<GameOverMessageRootNode>>,
) {
    // TODO: Refactor gameover event to be an enum and use match here instead.
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
        let mut game_over_text = game_over_text_query
            .get_single_mut()
            .expect("Could not find game over text node to set.");
        game_over_text.sections[0].value = game_over_message.clone();

        let mut game_over_root_node_vis = game_over_root_node_query
            .get_single_mut()
            .expect("Could not find game over message root node to change visibility.");

        *game_over_root_node_vis = Visibility::Visible;
    }
}

// TODO: Refactor to use types to reduce complexity
pub(super) fn clear_game_over_message_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    mut game_over_text_query: Query<&mut Visibility, With<GameOverMessageRootNode>>,
) {
    if restart_reader.read().next().is_some() {
        debug!("Clearing game over text after game was restarted.");
        for mut game_over_draw in game_over_text_query.iter_mut() {
            *game_over_draw = Visibility::Hidden;
        }
    }
}
