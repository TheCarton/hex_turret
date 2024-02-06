use crate::HexPosition;
use ::bevy::prelude::*;
use derive_more::Add;

pub const HEX_SIZE: f32 = 32.0;
pub const PLAYER_SPEED: f32 = 500.0;

pub const ENEMY_SPEED: f32 = PLAYER_SPEED / 2f32;

pub const NE: HexPosition = HexPosition { q: 1, r: -1 };
pub const E: HexPosition = HexPosition { q: 1, r: 0 };
pub const SE: HexPosition = HexPosition { q: 0, r: 1 };
pub const SW: HexPosition = HexPosition { q: -1, r: 1 };
pub const W: HexPosition = HexPosition { q: -1, r: 0 };
pub const NW: HexPosition = HexPosition { q: 0, r: -1 };
pub const HEX_DIRECTIONS: [HexPosition; 6] = [NE, E, SE, SW, W, NW];
