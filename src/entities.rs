use crate::enemies::Hit;
use crate::hex::*;

use bevy::prelude::*;
use derive_more::Add;

#[derive(Component, Default)]
pub(crate) struct Projectile;

#[derive(Component, Default)]
pub(crate) struct Velocity {
    pub(crate) v: Vec2,
}

impl From<Vec2> for Velocity {
    fn from(value: Vec2) -> Self {
        Velocity { v: value }
    }
}

impl From<Velocity> for Vec3 {
    fn from(value: Velocity) -> Self {
        Vec3::new(value.v.x, value.v.y, 0f32)
    }
}

impl From<&Velocity> for Vec3 {
    fn from(value: &Velocity) -> Self {
        Vec3::new(value.v.x, value.v.y, 0f32)
    }
}

impl From<Velocity> for Vec2 {
    fn from(value: Velocity) -> Self {
        Vec2::new(value.v.x, value.v.y)
    }
}

impl From<&Velocity> for Vec2 {
    fn from(value: &Velocity) -> Self {
        Vec2::new(value.v.x, value.v.y)
    }
}

#[derive(Component, Default, Add)]
pub(crate) struct Distance {
    pub(crate) d: f32,
}

impl From<f32> for Distance {
    fn from(value: f32) -> Self {
        Distance { d: value }
    }
}

#[derive(Bundle, Default)]
pub(crate) struct ProjectileBundle {
    pub(crate) projectile: Projectile,
    pub(crate) velocity: Velocity,
    pub(crate) distance: Distance,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

#[derive(Resource, Default)]
pub(crate) struct CursorWorldCoords {
    pub(crate) pos: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct CursorHexPosition {
    pub(crate) hex: HexPosition,
}
