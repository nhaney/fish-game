use bevy::prelude::*;

use crate::shared::stages;

mod sfx;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<sfx::SfxFiles>()
            .add_system_to_stage(stages::HANDLE_EVENTS, sfx::play_sfx_system.system());
    }
}
