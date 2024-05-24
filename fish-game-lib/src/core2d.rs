// This module contains the core datatypes used in the 2d fish-game simulation.
// These data types need to be fully deterministic so they will use
// purely integers or fixed-point numbers.
//
// 32 bit signed integers will be used to represent everything in the fish-game
// and then will transformed into the appropriate floats for rendering by
// the caller of the library.
use bevy::prelude::*;

#[derive(Debug)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component)]
pub struct Position2d(pub Vec2);
#[derive(Debug, Component)]
pub struct Velocity2d(pub Vec2);

pub struct FacingDirection {
    is_right: bool,
}

impl FacingDirection {
    pub fn is_right(&self) -> bool {
        self.is_right
    }

    pub fn is_left(&self) -> bool {
        !self.is_right
    }
}
