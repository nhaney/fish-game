use bevy::prelude::*;

#[derive(Debug, Clone)]
pub(super) struct PauseButtonMaterials {
    pause: Handle<ColorMaterial>,
    play: Handle<ColorMaterial>,
}

pub(super) struct PauseButton {
    is_paused: bool,
}

impl FromResources for PauseButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let asset_server = resources.get::<AssetServer>().unwrap();
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();

        println!("Loading pause button materials...");
        PauseButtonMaterials {
            pause: materials.add(asset_server.load("sprites/ui/pause.png").into()),
            play: materials.add(asset_server.load("sprites/ui/play.png").into()),
            // pause: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            // play: materials.add(Color::rgb(0.15, 0.55, 0.0).into()),
        }
    }
}

pub(super) fn pause_button_system(
    pause_button_materials: Res<PauseButtonMaterials>,
    mut interaction_query: Query<(
        Mutated<Interaction>,
        &mut Handle<ColorMaterial>,
        &mut PauseButton,
    )>,
) {
    for (interaction, mut material, mut pause_button) in interaction_query.iter_mut() {
        if let Interaction::Clicked = *interaction {
            if pause_button.is_paused {
                *material = pause_button_materials.pause.clone().into();
            } else {
                *material = pause_button_materials.play.clone().into();
            }

            pause_button.is_paused = !pause_button.is_paused;
        }
    }
}

pub(super) fn add_pause_button(
    container: &mut ChildBuilder,
    pause_button_materials: &PauseButtonMaterials,
    materials: &mut Assets<ColorMaterial>,
) {
    container
        .spawn(NodeComponents {
            style: Style {
                justify_content: JustifyContent::Center,
                padding: Rect {
                    left: Val::Px(50.0),
                    right: Val::Px(50.0),
                    top: Val::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonComponents {
                    style: Style {
                        size: Size::new(Val::Px(64.0), Val::Px(64.0)),
                        ..Default::default()
                    },
                    material: pause_button_materials.pause.clone(),
                    ..Default::default()
                })
                .with(PauseButton { is_paused: false });
        });
}
