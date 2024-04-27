use bevy::prelude::*;

use crate::shared::stages;

mod sfx;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<sfx::SfxHandles>().add_systems(
            Update,
            (sfx::play_sfx_system).in_set(stages::PrepareRenderSet),
        );
    }
}
