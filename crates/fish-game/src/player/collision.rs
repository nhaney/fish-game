use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};

use super::{
    attributes::Player,
    events::{PlayerAte, PlayerBonked, PlayerHooked},
};
use crate::{
    objects::boat::{Boat, Hook, Worm},
    shared::{
        arena::Arena,
        collision::Collider,
        game::{GameState, GameStates},
    },
};

/// Keeps player in bounds of arena
pub(super) fn player_bounds_system(
    game_state: Res<GameState>,
    arena: Res<Arena>,
    mut query: Query<(&Player, &Collider, &mut Transform)>,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (_, collider, mut transform) in query.iter_mut() {
        let new_pos = &mut transform.translation;

        let arena_half_width = arena.width / 2.0;
        let arena_half_height = arena.height / 2.0;

        let player_half_width = collider.width / 2.0;
        let player_half_height = collider.height / 2.0;

        if new_pos.x - player_half_width < -arena_half_width {
            new_pos.x = -arena_half_width + player_half_width;
        }

        if new_pos.x + player_half_width > arena_half_width {
            new_pos.x = arena_half_width - player_half_width;
        }

        if new_pos.y - player_half_height < -arena_half_height {
            new_pos.y = -arena_half_height + player_half_height;
        }

        // allow floating on the top
        if new_pos.y > (arena_half_height + arena.offset) {
            new_pos.y = arena_half_height + arena.offset;
        }
    }
}

pub(super) fn player_hook_collision_system(
    game_state: Res<GameState>,
    mut player_hooked_events: ResMut<Events<PlayerHooked>>,
    player_query: Query<(&Player, &Collider, &Transform, Entity)>,
    hook_query: Query<(&Hook, &Collider, &GlobalTransform, Entity)>,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (_, player_collider, player_transform, player_entity) in player_query.iter() {
        let player_pos = player_transform.translation;
        let player_half_size = player_collider.as_vec2_half_size();
        let player_aabb = Aabb2d::new(player_pos.xy(), player_half_size);
        for (_, hook_collider, hook_transform, hook_entity) in hook_query.iter() {
            let hook_pos = hook_transform.translation();
            let hook_half_size = hook_collider.as_vec2_half_size();
            let hook_aabb = Aabb2d::new(hook_pos.xy(), hook_half_size);

            if check_aabb_collision(&player_aabb, &hook_aabb) {
                debug!("Player collided with a hook!");

                player_hooked_events.send(PlayerHooked {
                    player_entity,
                    hook_entity,
                });
            }
        }
    }
}

pub(super) fn player_worm_collision_system(
    game_state: Res<GameState>,
    mut player_ate_events: ResMut<Events<PlayerAte>>,
    player_query: Query<(&Player, &Collider, &Transform, Entity)>,
    worm_query: Query<(&Worm, &Collider, &GlobalTransform, Entity)>,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (_, player_collider, player_transform, player_entity) in player_query.iter() {
        let player_aabb2d =
            get_aabb_from_transform_and_collider(&player_transform.translation, &player_collider);

        for (_, worm_collider, worm_transform, worm_entity) in worm_query.iter() {
            let worm_aabb2d =
                get_aabb_from_transform_and_collider(&worm_transform.translation(), worm_collider);

            if check_aabb_collision(&player_aabb2d, &worm_aabb2d) {
                debug!("Player collided with a worm!");
                player_ate_events.send(PlayerAte {
                    player_entity,
                    worm_entity,
                });
            }
        }
    }
}

pub(super) fn player_boat_collision_system(
    game_state: Res<GameState>,
    mut player_bonk_events: ResMut<Events<PlayerBonked>>,
    player_query: Query<(&Player, &Collider, &Transform, Entity)>,
    worm_query: Query<(&Boat, &Collider, &Transform, Entity)>,
) {
    if let GameStates::GameOver = game_state.cur_state {
        return;
    }

    for (_, player_collider, player_transform, player_entity) in player_query.iter() {
        let player_aabb2d =
            get_aabb_from_transform_and_collider(&player_transform.translation, player_collider);

        for (_, boat_collider, boat_transform, boat_entity) in worm_query.iter() {
            let boat_aabb2d =
                get_aabb_from_transform_and_collider(&boat_transform.translation, boat_collider);

            if check_aabb_collision(&player_aabb2d, &boat_aabb2d) {
                debug!("Player collided with a boat!");
                player_bonk_events.send(PlayerBonked {
                    player_entity,
                    boat_entity,
                });
            }
        }
    }
}

fn get_aabb_from_transform_and_collider(&pos: &Vec3, collider: &Collider) -> Aabb2d {
    Aabb2d::new(pos.xy(), collider.as_vec2_half_size())
}

fn check_aabb_collision(box1: &Aabb2d, box2: &Aabb2d) -> bool {
    return box1.intersects(box2.into());
}
