use bevy::prelude::*;

use crate::player::events::{PlayerAte, PlayerBonked, PlayerBoosted, PlayerHooked, PlayerStarved};

#[derive(Debug, Resource)]
pub(super) struct SfxHandles {
    bonked: Handle<AudioSource>,
    hooked: Handle<AudioSource>,
    starved: Handle<AudioSource>,
    eat: Handle<AudioSource>,
    boost: Handle<AudioSource>,
}

impl FromWorld for SfxHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            bonked: asset_server.load("audio/sfx/bonked.ogg"),
            hooked: asset_server.load("audio/sfx/hooked.ogg"),
            starved: asset_server.load("audio/sfx/starved.ogg"),
            eat: asset_server.load("audio/sfx/eat.ogg"),
            boost: asset_server.load("audio/sfx/boost.ogg"),
        }
    }
}

// TODO: See if this can be reduced
#[allow(clippy::too_many_arguments)]
pub(super) fn play_sfx_system(
    mut player_hooked_reader: EventReader<PlayerHooked>,
    mut player_starved_reader: EventReader<PlayerStarved>,
    mut player_bonked_reader: EventReader<PlayerBonked>,
    mut player_ate_reader: EventReader<PlayerAte>,
    mut player_boosted_reader: EventReader<PlayerBoosted>,
    sfx_handles: Res<SfxHandles>,
    mut commands: Commands,
) {
    for _ in player_hooked_reader.read() {
        debug!("Playing hooked sound effect");
        commands.spawn(AudioBundle {
            source: sfx_handles.hooked.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }

    for _ in player_starved_reader.read() {
        debug!("Playing starved sound effect");
        commands.spawn(AudioBundle {
            source: sfx_handles.starved.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }

    for _ in player_bonked_reader.read() {
        debug!("Playing bonked sound effect");
        commands.spawn(AudioBundle {
            source: sfx_handles.bonked.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }

    for _ in player_ate_reader.read() {
        debug!("Playing ate sound effect");
        commands.spawn(AudioBundle {
            source: sfx_handles.eat.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }

    for _ in player_boosted_reader.read() {
        debug!("Playing boosted sound effect");
        commands.spawn(AudioBundle {
            source: sfx_handles.boost.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}
