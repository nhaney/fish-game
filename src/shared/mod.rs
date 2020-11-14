use bevy::prelude::*;

pub mod animation;
pub mod arena;
pub mod collision;
pub mod game;
pub mod movement;
pub mod render;
pub mod rng;

pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut AppBuilder) {
        println!("Building shared plugin...");
        app.add_startup_system(initialize_game.system())
            .add_startup_system(arena::initialize_arena.system())
            .add_system_to_stage(stage::UPDATE, movement::movement_system.system())
            .add_system_to_stage_front(stage::LAST, animation::animation_system.system())
            .add_system_to_stage_front(stage::LAST, animation::flip_sprite_system.system())
            .add_system_to_stage_front(
                stage::LAST,
                render::scale_game_transform_to_screen_size.system(),
            )
            .init_resource::<rng::GameRng>()
            .add_resource(game::Difficulty {
                multiplier: 1,
                timer: Timer::from_seconds(10.0, true),
            })
            .add_system_to_stage(stage::PRE_UPDATE, game::difficulty_scaling_system.system());
    }
}

fn initialize_game(mut commands: Commands) {
    commands.spawn(Camera2dComponents::default());
}
