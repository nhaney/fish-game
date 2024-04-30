use bevy::prelude::*;

#[derive(Debug, Clone, Resource)]
pub(crate) struct FontHandles {
    pub main_font: Handle<Font>,
}

impl FromWorld for FontHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        debug!("Loading fonts...");
        Self {
            main_font: asset_server.load("fonts/Chonkly.ttf"),
        }
    }
}
