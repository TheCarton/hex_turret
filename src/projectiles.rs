use crate::constants::{
    ENEMY_SIZE, FIREFLY_HIT_ANIMATION_DURATION, PROJECTILE_DAMAGE, PROJECTILE_RANGE,
    PROJECTILE_SIZE,
};
use crate::enemies::{DamagedTime, Health, Hit, Seeking};
use crate::game::AppState;
use crate::hex::HexPosition;
use crate::turrets::{ControlRay, ControlVec, RayTimer};

use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::{ConfigureLoadingState, LoadingStateConfig};
use bevy_asset_loader::loading_state::LoadingStateAppExt;
use derive_more::Add;

pub(crate) struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::AssetLoading)
                .load_collection::<TurretProjectileAssets>()
                .load_collection::<FireflyProjectileAssets>(),
        );
        app.add_systems(
            Update,
            (
                detect_proj_enemy_collision,
                despawn_projectiles,
                move_projectiles,
                update_control_rays,
            ),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub(crate) struct TurretProjectileAssets {
    #[asset(texture_atlas_layout(tile_size_x = 6., tile_size_y = 8., columns = 1, rows = 1))]
    pub(crate) layout: Handle<TextureAtlasLayout>,
    #[asset(path = "turret_projectile.png")]
    pub(crate) projectile: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct FireflyProjectileAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    pub(crate) layout: Handle<TextureAtlasLayout>,
    #[asset(path = "firefly_projectile.png")]
    pub(crate) projectile: Handle<Image>,
}

#[derive(Component, Default)]
pub(crate) struct Projectile;

#[derive(Component, Default)]
pub(crate) struct FireflyProjectile;

#[derive(Bundle, Default)]
pub(crate) struct FireflyProjectileBundle {
    pub(crate) firefly_projectile: FireflyProjectile,
    pub(crate) projectile: Projectile,
    pub(crate) velocity: Velocity,
    pub(crate) distance: Distance,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

#[derive(Component, Default)]
pub(crate) struct TurretProjectile;

#[derive(Bundle, Default)]
pub(crate) struct TurretProjectileBundle {
    pub(crate) projectile: Projectile,
    pub(crate) turret_projectile: TurretProjectile,
    pub(crate) velocity: Velocity,
    pub(crate) distance: Distance,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

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

fn detect_proj_enemy_collision(
    mut q_enemies: Query<
        (&Transform, &mut DamagedTime, &mut Health),
        (With<Seeking>, Without<TurretProjectile>),
    >,
    mut q_projectiles: Query<(&Transform, &mut Hit), (With<TurretProjectile>, Without<Seeking>)>,
) {
    for (proj, mut proj_hit) in &mut q_projectiles {
        for (enemy, mut damage_dur, mut enemy_health) in &mut q_enemies {
            let collision =
                Aabb2d::new(enemy.translation.truncate(), ENEMY_SIZE / 2f32).intersects(
                    &Aabb2d::new(proj.translation.truncate(), PROJECTILE_SIZE / 2f32),
                );
            if collision {
                proj_hit.has_hit = true;
                enemy_health.hp -= PROJECTILE_DAMAGE;
                damage_dur.time = Some(Timer::from_seconds(
                    FIREFLY_HIT_ANIMATION_DURATION,
                    TimerMode::Once,
                ));
                break;
            }
        }
    }
}

fn move_projectiles(
    mut q_projectiles: Query<(&mut Transform, &mut Distance, &Velocity), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut trans, mut dist, vel) in &mut q_projectiles {
        let v = Vec3::from(vel) * time.delta_seconds();
        trans.translation += v;
        dist.d += v.length();
    }
}

fn update_control_rays(
    mut q_control_rays: Query<
        (Entity, &mut RayTimer, &mut ControlVec, &HexPosition),
        With<ControlRay>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity_id, mut ray_time, mut control_vec, hex_vel) in q_control_rays.iter_mut() {
        ray_time.timer.tick(time.delta());
        if ray_time.timer.finished() {
            commands.entity(entity_id).despawn();
        }
        for pos in control_vec.hexes.iter_mut() {
            *pos = *pos + *hex_vel;
        }
    }
}

fn despawn_projectiles(
    mut commands: Commands,
    q_projectiles: Query<(Entity, &Distance, &Hit), With<TurretProjectile>>,
) {
    for (entity, dist, hit) in &q_projectiles {
        if dist.d > PROJECTILE_RANGE || hit.has_hit {
            commands.entity(entity).despawn_recursive();
        }
    }
}
