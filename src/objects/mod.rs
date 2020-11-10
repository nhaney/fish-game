use bevy::prelude::*;

mod boat;

pub struct ObjectPlugins;

impl Plugin for ObjectPlugins {
    fn build(&self, app: &mut AppBuilder) {
        println!("test");
        app.add_startup_system(boat::spawn_boat.system());
    }
}
