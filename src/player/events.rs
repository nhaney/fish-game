use bevy::{prelude::*, sprite::collide_aabb::Collision};

#[derive(Debug)]
pub struct PlayerHooked {
    pub player_entity: Entity,
    pub hook_entity: Entity,
    pub collision: Collision,
}

#[derive(Debug)]
pub struct PlayerStarved {
    pub player_entity: Entity,
}

#[derive(Debug)]
pub struct PlayerBonked {
    pub player_entity: Entity,
    pub boat_entity: Entity,
    pub collision: Collision,
}

#[derive(Debug)]
pub struct PlayerAte {
    pub player_entity: Entity,
    pub worm_entity: Entity,
    pub collision: Collision,
}

#[derive(Debug)]
pub struct PlayerBoosted {
    pub player: Entity,
}
