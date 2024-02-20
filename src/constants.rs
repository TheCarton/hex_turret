use crate::hex::HexPosition;
use ::bevy::prelude::*;

pub const HEX_SIZE: f32 = 32.0;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(28f32, 16f32);

pub const PROJECTILE_SPEED: f32 = PLAYER_SPEED * 2f32;
pub const PROJECTILE_RANGE: f32 = HEX_SIZE * 8f32;
pub const PROJECTILE_SIZE: Vec2 = Vec2::new(6f32, 8f32);
pub const PROJECTILE_DAMAGE: f32 = 20f32;

pub const TURRET_RANGE: f32 = HEX_SIZE * 6f32;
pub const TURRET_RELOAD_SECONDS: f32 = 0.75;

pub const TRIGGER_RANGE: f32 = HEX_SIZE;

pub const ENEMY_SPEED: f32 = PLAYER_SPEED / 2f32;
pub const ENEMY_HEALTH: f32 = 100f32;
pub const ENEMY_SIZE: Vec2 = Vec2::new(42f32, 38f32);

pub const FIREFLY_HIT_ANIMATION_DURATION: f32 = 0.5f32;

pub const NE: HexPosition = HexPosition { q: 1, r: -1 };
pub const E: HexPosition = HexPosition { q: 1, r: 0 };
pub const SE: HexPosition = HexPosition { q: 0, r: 1 };
pub const SW: HexPosition = HexPosition { q: -1, r: 1 };
pub const W: HexPosition = HexPosition { q: -1, r: 0 };
pub const NW: HexPosition = HexPosition { q: 0, r: -1 };
pub const HEX_DIRECTIONS: [HexPosition; 6] = [NE, E, SE, SW, W, NW];
