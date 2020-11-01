#[derive(Debug)]
pub struct PlayerStats {
    pub boost_speed: f32,
    pub boost_duration: f32,
    pub boost_cooldown: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub traction: f32,
    pub stop_threshold: f32,
}

#[derive(Debug)]
pub struct Player {
    pub stats: PlayerStats,
}

pub struct Sink {
    pub weight: f32,
}
