use bevy::prelude::*;
use fish_game_core::GamePhase;

use crate::core_adapter::{CoreControl, CoreState};
use crate::shared::game::GameRestarted;

#[derive(Debug, Clone, Resource)]
pub(super) struct PauseButtonMaterials {
    pub pause: Handle<Image>,
    pub play: Handle<Image>,
}

#[derive(Debug, Component)]
pub(super) struct PauseButton {
    is_paused: bool,
}

impl FromWorld for PauseButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();

        debug!("Loading pause button materials...");
        PauseButtonMaterials {
            pause: asset_server.load("sprites/ui/pause.png"),
            play: asset_server.load("sprites/ui/play.png"),
        }
    }
}

pub(super) fn setup_pause_button(
    mut commands: Commands,
    pause_button_materials: Res<PauseButtonMaterials>,
) {
    commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                margin: UiRect {
                    right: Val::Percent(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            image: UiImage::new(pause_button_materials.pause.clone()),
            ..Default::default()
        },
        PauseButton { is_paused: false },
    ));
}

/// The button routes clicks to `CoreControl.pause_toggle_pending`, which the
/// adapter consumes on the next tick. The adapter then emits the Bevy
/// `GamePaused` / `GameUnpaused` events — this system just asks; it doesn't
/// decide.
pub(super) fn pause_button_system(
    core: Res<CoreState>,
    mut control: ResMut<CoreControl>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut UiImage, &mut PauseButton),
        Changed<Interaction>,
    >,
) {
    if core.state.phase == GamePhase::GameOver {
        return;
    }

    for (interaction, mut ui_image, mut pause_button) in interaction_query.iter_mut() {
        if let Interaction::Pressed = *interaction {
            if pause_button.is_paused {
                ui_image.texture = pause_button_materials.pause.clone();
            } else {
                ui_image.texture = pause_button_materials.play.clone();
            }
            pause_button.is_paused = !pause_button.is_paused;
            control.pause_toggle_pending = true;
        }
    }
}

/// Keep the button sprite in sync with the adapter's pause state — covers the
/// keyboard-shortcut path (Escape) so the icon flips even though the button
/// wasn't clicked.
pub(super) fn sync_pause_button_to_control(
    control: Res<CoreControl>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut pause_button_query: Query<(&mut UiImage, &mut PauseButton)>,
) {
    for (mut ui_image, mut pause_button) in pause_button_query.iter_mut() {
        if pause_button.is_paused != control.paused {
            pause_button.is_paused = control.paused;
            ui_image.texture = if control.paused {
                pause_button_materials.play.clone()
            } else {
                pause_button_materials.pause.clone()
            };
        }
    }
}

pub(super) fn reset_pause_button_on_restart(
    mut restart_reader: EventReader<GameRestarted>,
    pause_button_materials: Res<PauseButtonMaterials>,
    mut pause_button_query: Query<(&mut UiImage, &mut PauseButton)>,
) {
    if restart_reader.read().next().is_some() {
        for (mut ui_image, mut pause_button) in pause_button_query.iter_mut() {
            ui_image.texture = pause_button_materials.pause.clone();
            pause_button.is_paused = false;
        }
    }
}

