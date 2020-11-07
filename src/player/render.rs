use bevy::prelude::*;

/// Represents one frame of animation
pub struct AnimationFrame {
    atlas_index: u32,
    time: f32,
}

/// Represents an entire animation
pub struct Animation {
    should_loop: bool,
    frames: Vec<AnimationFrame>,
}

/// Component that represents the current state of animation
pub struct AnimationState {
    animation: Animation,
    timer: Timer,
    frame_index: u32,
    speed_multiplier: f32,
}

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

#[derive(Default)]
pub struct PlayerSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

fn start_atlas_load(
    mut player_sprite_handles: ResMut<PlayerSpriteHandles>,
    asset_server: Res<AssetServer>,
) {
    player_sprite_handles.handles = asset_server.load_folder("assets/sprites/player").unwrap();
}

/**
Adds the player sprite to a player without a sprite as soon as the textures
load.
*/
fn load_player_atlas(
    mut commands: Commands,
    mut player_sprite_handles: ResMut<PlayerSpriteHandles>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<Without<SpriteSheetComponents, (&Player, Entity)>>,
) {
    if player_sprite_handles.atlas_loaded {
        return;
    }

    let mut texture_atlas_builder = TextureAtlasBuilder::default();

    if let LoadState::Loaded = asset_server
        .get_group_load_state(player_sprite_handles.handles.iter().map(|handle| handle.id))
    {
        for handle in player_sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }
        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();

        let player_animations = create_player_animations(texture_atlas);

        let vendor_handle =
            asset_server.get_handle("textures/rpg/chars/vendor/generic-rpg-vendor.png");
        let vendor_index = texture_atlas.get_texture_index(&vendor_handle).unwrap();
        let atlas_handle = texture_atlases.add(texture_atlas);

        // set up a scene to display our texture atlas
        commands
            .spawn(Camera2dComponents::default())
            // draw a sprite from the atlas
            .spawn(SpriteSheetComponents {
                transform: Transform {
                    translation: Vec3::new(150.0, 0.0, 0.0),
                    scale: Vec3::splat(4.0),
                    ..Default::default()
                },
                sprite: TextureAtlasSprite::new(vendor_index as u32),
                texture_atlas: atlas_handle,
                ..Default::default()
            })
            // draw the atlas itself
            .spawn(SpriteComponents {
                material: materials.add(texture_atlas_texture.into()),
                transform: Transform::from_translation(Vec3::new(-300.0, 0.0, 0.0)),
                ..Default::default()
            });

        rpg_sprite_handles.atlas_loaded = true;
    }
}
