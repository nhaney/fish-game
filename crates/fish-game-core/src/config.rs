use serde::{Deserialize, Serialize};

/// All tunables for a simulation. A replay is `(FishGameConfig, Vec<FishGameInput>)`
/// — same config + same inputs MUST produce the same hash on every target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FishGameConfig {
    pub tick_rate: u32,
    pub seed: [u8; 32],
    pub arena: ArenaConfig,
    pub player: PlayerStatsConfig,
    pub score_interval_ticks: u32,
    pub difficulty_interval_ticks: u32,
    pub boat_spawn_interval_ticks: u32,
    pub max_difficulty: u8,
    pub score_per_worm: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArenaConfig {
    pub width: f32,
    pub height: f32,
    pub offset: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerStatsConfig {
    pub width: f32,
    pub height: f32,
    pub max_boosts: u8,
    pub boost_speed: f32,
    pub boost_duration_ticks: u32,
    pub boost_cooldown_ticks: u32,
    pub speed: f32,
    pub acceleration: f32,
    pub traction: f32,
    pub stop_threshold: f32,
    pub sink_weight: f32,
    pub hunger_ticks: u32,
    pub extra_hunger_ticks_per_worm: u32,
}

impl Default for FishGameConfig {
    fn default() -> Self {
        Self {
            tick_rate: 60,
            seed: [0; 32],
            arena: ArenaConfig {
                width: 640.0,
                height: 360.0,
                offset: -50.0,
            },
            player: PlayerStatsConfig {
                width: 32.0,
                height: 32.0,
                max_boosts: 3,
                boost_speed: 1500.0,
                // 0.1s at 60Hz.
                boost_duration_ticks: 6,
                // 0.2s at 60Hz.
                boost_cooldown_ticks: 12,
                speed: 400.0,
                acceleration: 0.8,
                traction: 0.8,
                stop_threshold: 0.1,
                sink_weight: 10.0,
                // 30s at 60Hz.
                hunger_ticks: 30 * 60,
                // 3s at 60Hz.
                extra_hunger_ticks_per_worm: 3 * 60,
            },
            // 1s at 60Hz.
            score_interval_ticks: 60,
            // 10s at 60Hz.
            difficulty_interval_ticks: 10 * 60,
            // 5s at 60Hz.
            boat_spawn_interval_ticks: 5 * 60,
            max_difficulty: 4,
            score_per_worm: 5,
        }
    }
}

impl FishGameConfig {
    pub fn with_seed(mut self, seed: [u8; 32]) -> Self {
        self.seed = seed;
        self
    }

    #[inline]
    pub fn dt(&self) -> f32 {
        1.0 / self.tick_rate as f32
    }
}
