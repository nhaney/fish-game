use bevy::prelude::*;

#[derive(Debug)]
pub(super) struct PlayerStats {
    pub boost_speed: f32,
    pub boost_duration: f32,
    pub boost_cooldown: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub traction: f32,
    pub stop_threshold: f32,
}

#[derive(Debug)]
pub(super) struct Player {
    pub stats: PlayerStats,
}

// TODO: Could this component be shared? It might be cool for other things to sink in the future
pub(super) struct Sink {
    pub weight: f32,
}

pub(super) struct HungerCountdown {
    time_left: f32,
}

pub(super) fn hunger_countdown_system(
    time: Res<Time>,
    mut query: Query<(&mut HungerCountdown, Entity)>,
) {
    for (mut hunger_countdown, entity) in query.iter_mut() {
        hunger_countdown.time_left -= time.delta_seconds;

        if hunger_countdown.time_left < 0.0 {
            // emit starved event for entity
            ()
        }
    }
}
