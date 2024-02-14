use crate::constants::*;
use std::{collections::HashMap, iter::Cycle};

use bevy::{prelude::*, time::Stopwatch};
use derive_more::Add;

fn cube_round(frac: Vec3) -> Vec3 {
    let mut q = frac.x.round();
    let mut r = frac.y.round();
    let mut s = frac.z.round();

    let q_diff = (q - frac.x).abs();
    let r_diff = (r - frac.y).abs();
    let s_diff = (s - frac.z).abs();

    if q_diff > r_diff && q_diff > s_diff {
        q = -r - s;
    } else if r_diff > s_diff {
        r = -q - s;
    } else {
        s = -q - r;
    }
    Vec3::new(q, r, s)
}

impl HexPosition {
    pub(crate) fn pixel_coords(&self) -> Vec2 {
        let x =
            HEX_SIZE * (3f32.sqrt() * f32::from(self.q) + 3f32.sqrt() / 2f32 * f32::from(self.r));
        let y = HEX_SIZE * (3f32 / 2f32 * f32::from(self.r));
        Vec2::new(x, y)
    }

    pub(crate) fn from_qr(q: i8, r: i8) -> HexPosition {
        HexPosition { q, r }
    }

    pub(crate) fn from_pixel(pixel_pos: Vec2) -> HexPosition {
        let q = (3f32.sqrt() / 3f32 * pixel_pos.x - 1f32 / 3f32 * pixel_pos.y) / HEX_SIZE;
        let r = (2f32 / 3f32 * pixel_pos.y) / HEX_SIZE;
        let s = -q - r;
        let rounded = cube_round(Vec3::new(q, r, s));
        HexPosition {
            q: rounded.x as i8,
            r: rounded.y as i8,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct HexBundle {
    pub(crate) pos: HexPosition,
    pub(crate) status: HexStatus,
    pub(crate) sprite: SpriteBundle,
}

#[derive(Component)]
pub(crate) struct HexMap {
    pub(crate) size: i8,
    pub(crate) map: HashMap<HexPosition, Entity>,
}

#[derive(Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Add, Default)]
pub(crate) struct HexPosition {
    pub(crate) q: i8,
    pub(crate) r: i8,
}

impl HexPosition {
    pub(crate) fn s(&self) -> i8 {
        -self.q - self.r
    }
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Component, Eq, PartialEq)]
pub(crate) enum HexStatus {
    Occupied,
    Unoccupied,
    Selected,
}

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    pub(crate) player: Player,
    pub(crate) pos: HexPosition,
    pub(crate) sprite: SpriteBundle,
}

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

#[derive(Component, Default)]
pub(crate) struct Hit {
    pub(crate) has_hit: bool,
}

#[derive(Bundle, Default)]
pub(crate) struct ProjectileBundle {
    pub(crate) projectile: Projectile,
    pub(crate) velocity: Velocity,
    pub(crate) distance: Distance,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

#[derive(Component, Debug)]
pub(crate) struct AnimationIndices {
    pub(crate) cycle: Cycle<std::ops::Range<usize>>,
}

impl Default for AnimationIndices {
    fn default() -> Self {
        AnimationIndices {
            cycle: (0usize..1usize).into_iter().cycle(),
        }
    }
}

impl AnimationIndices {
    pub(crate) fn new(first: usize, last: usize) -> AnimationIndices {
        AnimationIndices {
            cycle: (first..last).cycle(),
        }
    }

    pub(crate) fn next_index(&mut self) -> usize {
        self.cycle.next().expect("cycle never empty")
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer {
    pub(crate) timer: Timer,
}

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

#[derive(Bundle, Default)]
pub(crate) struct FireflyBundle {
    pub(crate) firefly: Firefly,
    pub(crate) animation_state: CurrentFireflyAnimationState,
    pub(crate) prev_animation_state: PrevFireflyAnimationState,
    pub(crate) hit: Hit,
    pub(crate) damaged_time: DamagedTime,
    pub(crate) enemy: Enemy,
    pub(crate) health: Health,
    pub(crate) sprite: SpriteSheetBundle,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(Component, Default)]
pub(crate) struct Enemy;

impl Default for DamagedTime {
    fn default() -> Self {
        DamagedTime { time: None }
    }
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub(crate) enum FireflyAnimationState {
    #[default]
    Normal,
    Damaged,
}

#[derive(Component, Default)]
pub(crate) struct PrevFireflyAnimationState {
    pub(crate) state: FireflyAnimationState,
}

#[derive(Component, Default)]
pub(crate) struct CurrentFireflyAnimationState {
    pub(crate) state: FireflyAnimationState,
}

#[derive(Resource)]
pub(crate) struct EnemySpawnConfig {
    pub(crate) timer: Timer,
}

#[derive(Component, Default)]
pub(crate) struct Firefly;

#[derive(Component)]
pub(crate) struct DamagedTime {
    pub(crate) time: Option<Timer>,
}

#[derive(Component)]
pub(crate) enum HexDirections {
    North,
}

#[derive(Component)]
pub(crate) struct Antenna;

#[derive(Component, Default)]
pub(crate) struct Turret;

#[derive(Resource)]
pub(crate) struct FireflyTextureAtlas {
    pub(crate) atlas: Handle<TextureAtlas>,
}

impl FromWorld for FireflyTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture_handle = asset_server.load("firefly_spritesheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(48f32, 48f32), 8, 3, None, None);
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        FireflyTextureAtlas {
            atlas: texture_atlas_handle,
        }
    }
}

#[derive(Component)]
pub(crate) struct ReloadTimer {
    pub(crate) timer: Timer,
}

impl Default for ReloadTimer {
    fn default() -> Self {
        ReloadTimer {
            timer: Timer::from_seconds(TURRET_RELOAD_SECONDS, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub(crate) struct AimVec {
    pub(crate) v: Option<Vec2>,
}

impl Default for AimVec {
    fn default() -> Self {
        AimVec { v: None }
    }
}

#[derive(Component, Default, Eq, PartialEq)]
pub(crate) enum TurretStatus {
    #[default]
    Neutral,
    Friendly,
    Hostile,
}

#[derive(Bundle, Default)]
pub(crate) struct TurretBundle {
    pub(crate) turret: Turret,
    pub(crate) status: TurretStatus,
    pub(crate) sprite: SpriteSheetBundle,
    pub(crate) reload_timer: ReloadTimer,
    pub(crate) aim: AimVec,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(Resource)]
pub(crate) struct TurretTextureAtlas {
    pub(crate) atlas: Handle<TextureAtlas>,
}

impl FromWorld for TurretTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture_handle = asset_server.load("turret.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(64f32, 64f32), 1, 1, None, None);
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        TurretTextureAtlas {
            atlas: texture_atlas_handle,
        }
    }
}

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct Health {
    pub(crate) hp: f32,
}

impl Default for Health {
    fn default() -> Self {
        Health { hp: ENEMY_HEALTH }
    }
}

#[derive(Resource, Default)]
pub(crate) struct CursorWorldCoords {
    pub(crate) pos: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct CursorHexPosition {
    pub(crate) hex: HexPosition,
}
