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
