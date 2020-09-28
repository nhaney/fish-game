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

pub mod state {
    use super::Player;
    use crate::shared::components::SideScrollDirection;
    use bevy::prelude::*;

    #[derive(Debug)]
    pub struct NormalState;

    #[derive(Debug)]
    pub struct BoostState {
        pub boost_velocity: Vec3,
        pub boost_timer: Timer,
    }

    impl BoostState {
        pub fn new(boost_velocity: Vec3, boost_duration: f32) -> Self {
            BoostState {
                boost_velocity,
                boost_timer: Timer::from_seconds(boost_duration, false),
            }
        }
    }

    #[derive(Debug)]
    pub struct BoostCooldown {
        pub timer: Timer,
        pub did_release: bool,
    }

    pub fn start_boost(
        commands: &mut Commands,
        entity: Entity,
        player: &Player,
        facing: &SideScrollDirection,
        target_speed: &Vec3,
    ) {
        // if not moving, boost in the direction that the player is facing
        let boost_direction = if *target_speed == Vec3::zero() {
            if facing.is_right() {
                Vec3::unit_x()
            } else {
                -Vec3::unit_x()
            }
        } else {
            target_speed.normalize()
        };

        // change state and specify velocity of the boost
        commands.insert_one(
            entity,
            BoostState::new(
                boost_direction * player.stats.boost_speed,
                player.stats.boost_duration,
            ),
        );
        commands.remove_one::<NormalState>(entity);
        println!("Started boost state");
    }

    // BoostState -> NormalState
    pub fn end_boost(commands: &mut Commands, player: &mut Player, entity: Entity) {
        println!("Finished boost, starting cooldown timer");
        commands.insert_one(
            entity,
            BoostCooldown {
                timer: Timer::from_seconds(player.stats.boost_cooldown, false),
                did_release: false,
            },
        );
        commands.insert_one(entity, NormalState);
        commands.remove_one::<BoostState>(entity);
    }
}
