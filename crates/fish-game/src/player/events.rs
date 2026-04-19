use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct PlayerHooked {
    pub player_entity: Entity,
    pub hook_entity: Entity,
}

#[derive(Debug, Event)]
pub struct PlayerStarved {
    pub player_entity: Entity,
}

#[derive(Debug, Event)]
pub struct PlayerBonked {
    pub player_entity: Entity,
    pub boat_entity: Entity,
}

#[derive(Debug, Event)]
pub struct PlayerAte {
    pub player_entity: Entity,
    pub worm_entity: Entity,
}

#[derive(Debug, Event)]
pub struct PlayerBoosted {
    pub player: Entity,
}
