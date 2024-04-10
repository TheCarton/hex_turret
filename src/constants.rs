use crate::hex::HexPosition;
use ::bevy::prelude::*;

pub const HEX_SIZE: f32 = 32.0;
pub const CONTROL_DECAY: f32 = 2.5f32;
pub const RED_CONTROL_TARGET: f32 = 0f32;
pub const BLUE_CONTROL_TARGET: f32 = 0f32;
pub const NEUTRAL_CONTROL_TARGET: f32 = 100f32;
pub const MAX_CONTROL_VALUE: f32 = 500f32;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(28f32, 16f32);

pub const PROJECTILE_SPEED: f32 = PLAYER_SPEED * 2f32;
pub const PROJECTILE_RANGE: f32 = HEX_SIZE * 8f32;
pub const TURRET_BULLET_SIZE: Vec2 = Vec2::new(6f32, 8f32);
pub const PROJECTILE_DAMAGE: f32 = 20f32;

pub const FACTORY_SIZE: Vec2 = Vec2::new(48f32, 48f32);

pub const FIREFLY_BULLET_SIZE: Vec2 = Vec2 {
    x: FIREFLY_BULLET_IMAGE_SIZE.x * FIREFLY_BULLET_SCALE.x,
    y: FIREFLY_BULLET_IMAGE_SIZE.y * FIREFLY_BULLET_SCALE.y,
};

pub const FIREFLY_BULLET_IMAGE_SIZE: Vec2 = Vec2::new(64f32, 64f32);
pub const FIREFLY_BULLET_SCALE: Vec3 = Vec3::new(0.25f32, 0.25f32, 0.25f32);

pub const TURRET_RANGE: f32 = HEX_SIZE * 6f32;
pub const TURRET_RELOAD_SECONDS: f32 = 0.75;
pub const TURRET_HEALTH: f32 = 125f32;
pub const TURRET_SIZE: Vec2 = Vec2::new(64f32, 64f32);

pub const ANTENNA_FIRE_RATE: f32 = 0.15;
pub const ANTENNA_RANGE: i8 = 4;
pub const ANTENNA_SIZE: Vec2 = Vec2::new(55f32, 57f32);

pub const TRIGGER_RANGE: f32 = HEX_SIZE;

pub const FIREFLY_SPEED: f32 = PLAYER_SPEED / 2f32;
pub const FIREFLY_HEALTH: f32 = 100f32;
pub const FIREFLY_SIZE: Vec2 = Vec2::new(42f32, 38f32);

pub const FIREFLY_HIT_ANIMATION_DURATION: f32 = 0.5f32;
pub const FIREFLY_RANGE: f32 = HEX_SIZE * 1.5;

pub const NE: HexPosition = HexPosition { q: 1, r: -1 };
pub const E: HexPosition = HexPosition { q: 1, r: 0 };
pub const SE: HexPosition = HexPosition { q: 0, r: 1 };
pub const SW: HexPosition = HexPosition { q: -1, r: 1 };
pub const W: HexPosition = HexPosition { q: -1, r: 0 };
pub const NW: HexPosition = HexPosition { q: 0, r: -1 };
pub const HEX_DIRECTIONS: [HexPosition; 6] = [NE, E, SE, SW, W, NW];
