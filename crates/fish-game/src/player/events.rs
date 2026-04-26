use bevy::prelude::*;

#[derive(Debug, Message)]
pub struct PlayerHooked {
    pub player_entity: Entity,
    pub hook_entity: Entity,
}

#[derive(Debug, Message)]
pub struct PlayerStarved {
    pub player_entity: Entity,
}

#[derive(Debug, Message)]
pub struct PlayerBonked {
    pub player_entity: Entity,
    pub boat_entity: Entity,
}

#[derive(Debug, Message)]
pub struct PlayerAte {
    pub player_entity: Entity,
    pub worm_entity: Entity,
}

#[derive(Debug, Message)]
pub struct PlayerBoosted {
    pub player: Entity,
}
