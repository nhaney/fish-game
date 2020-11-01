use bevy::prelude::*;

/// creates the player texture atlas
fn init_player_animations(
    asset_server: Res<AssetServer>,
) -> HashMap<String, Vec<Handle<ColorMaterial>>> {
    let mut player_sprite_map = HashMap::new();

    // insert all sprites with animations here
    player_sprite_map.insert(
        "swim".to_string(),
        vec![
            asset_server.load("assets/player/fish1.png").unwrap(),
            asset_server.load("assets/player/fish2.png").unwrap(),
        ],
    );

    player_sprite_map
}
