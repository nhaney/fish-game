use bevy::{prelude::*, sprite::collide_aabb::Collision};

pub struct PlayerHooked {
    pub player_entity: Entity,
    pub hook_entity: Entity,
    pub collision: Collision,
}

pub struct PlayerStarved {
    pub player_entity: Entity,
}

pub struct PlayerBonked {
    pub player_entity: Entity,
    pub boat_entity: Entity,
    pub collision: Collision,
}

pub struct PlayerAte {
    pub player_entity: Entity,
    pub worm_entity: Entity,
    pub collision: Collision,
}
