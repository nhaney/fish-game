use serde::{Deserialize, Serialize};

/// One input packet per tick. The presentation layer translates keyboard state
/// into this struct before calling `tick`.
///
/// `boost_pressed` + `boost_just_pressed` are kept separate because boost logic
/// cares about both the edge (pressing) and the level (holding — so we can gate
/// re-boost behind a release).
///
/// Lifecycle (pause / restart) lives in the presentation layer:
///   - Pause = stop calling `tick` until unpause.
///   - Restart = construct a new `FishGameState` with a fresh seed.
/// Keeping them out of core means the input log a replay records is purely
/// gameplay — there are no "user paused for 30s" frames to replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FishGameInput {
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub boost_pressed: bool,
    pub boost_just_pressed: bool,
}
