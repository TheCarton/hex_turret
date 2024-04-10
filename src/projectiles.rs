use crate::constants::{
    FIREFLY_BULLET_SIZE, FIREFLY_HIT_ANIMATION_DURATION, FIREFLY_SIZE, PROJECTILE_DAMAGE,
    PROJECTILE_RANGE, PROJECTILE_SPEED, TURRET_BULLET_SIZE,
};
use crate::enemies::{Health, Hit, Hittable};
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
                projectile_collisions,
                move_projectiles,
                update_control_rays,
                despawn_projectiles,
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

#[derive(Component)]
pub(crate) struct Projectile {
    kind: ProjectileType,
    origin: Vec2,
    velocity: Vec2,
}

#[derive(Default)]
pub(crate) enum ProjectileType {
    #[default]
    TurretBullet,
    FireflyBullet,
}

const TURRET_BULLET_DAMAGE: f32 = 25f32;
const FIREFLY_BULLET_DAMAGE: f32 = 15f32;

impl Projectile {
    pub(crate) fn new_turret_bullet(origin: Vec2, velocity: Vec2) -> Projectile {
        Projectile {
            kind: ProjectileType::TurretBullet,
            origin,
            velocity,
        }
    }

    pub(crate) fn new_firefly_bullet(origin: Vec2, velocity: Vec2) -> Projectile {
        Projectile {
            kind: ProjectileType::FireflyBullet,
            origin,
            velocity,
        }
    }

    fn damage(&self) -> f32 {
        match self.kind {
            ProjectileType::TurretBullet => TURRET_BULLET_DAMAGE,
            ProjectileType::FireflyBullet => FIREFLY_BULLET_DAMAGE,
        }
    }

    fn size(&self) -> Vec2 {
        match self.kind {
            ProjectileType::TurretBullet => TURRET_BULLET_SIZE,
            ProjectileType::FireflyBullet => FIREFLY_BULLET_SIZE,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct FireflyProjectileBundle {
    pub(crate) projectile: Projectile,
    pub(crate) velocity: Velocity,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

impl Default for FireflyProjectileBundle {
    fn default() -> Self {
        FireflyProjectileBundle {
            projectile: Projectile::new_firefly_bullet(Vec2::ZERO, Vec2::ZERO),
            velocity: Velocity::default(),
            sprite: SpriteBundle::default(),
            hit: Hit::default(),
        }
    }
}

#[derive(Bundle)]
pub(crate) struct TurretProjectileBundle {
    pub(crate) projectile: Projectile,
    pub(crate) velocity: Velocity,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hit: Hit,
}

impl Default for TurretProjectileBundle {
    fn default() -> Self {
        TurretProjectileBundle {
            projectile: Projectile::new_turret_bullet(Vec2::ZERO, Vec2::ZERO),
            velocity: Velocity::default(),
            sprite: SpriteBundle::default(),
            hit: Hit::default(),
        }
    }
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

fn projectile_collisions(
    mut q_hittables: Query<(&Transform, &mut Health, &mut Hittable), Without<Projectile>>,
    mut q_projectiles: Query<(&Transform, &mut Hit, &Projectile), With<Projectile>>,
) {
    for (proj_transform, mut proj_hit, projectile) in &mut q_projectiles {
        for (target_transform, mut target_health, mut hittable) in &mut q_hittables {
            let collision = Aabb2d::new(
                target_transform.translation.truncate(),
                hittable.hitbox / 2f32,
            )
            .intersects(&Aabb2d::new(
                proj_transform.translation.truncate(),
                projectile.size() / 2f32,
            ));
            if collision {
                proj_hit.has_hit = true;
                target_health.hp -= projectile.damage();
                hittable.hit = true;
                break;
            }
        }
    }
}

fn move_projectiles(mut q_projectiles: Query<(&mut Transform, &Projectile)>, time: Res<Time>) {
    for (mut trans, proj) in &mut q_projectiles {
        let new_translation =
            (proj.velocity * time.delta_seconds()).extend(0f32) + trans.translation;
        trans.translation = new_translation;
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
    q_projectiles: Query<(Entity, &Transform, &Projectile, &Hit)>,
) {
    for (entity, trans, proj, hit) in &q_projectiles {
        let distance_traveled = trans.translation.truncate().distance(proj.origin);
        if distance_traveled > PROJECTILE_RANGE || hit.has_hit {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn spawn_projectile(
    commands: &mut Commands,
    projectile_type: ProjectileType,
    velocity: Vec2,
    texture: Handle<Image>,
    transform: Transform,
) {
    match projectile_type {
        ProjectileType::TurretBullet => {
            commands.spawn(TurretProjectileBundle {
                projectile: Projectile::new_turret_bullet(
                    transform.translation.truncate(),
                    velocity,
                ),
                sprite: SpriteBundle {
                    texture: texture.clone(),
                    transform,
                    ..default()
                },
                ..default()
            });
        }
        ProjectileType::FireflyBullet => {
            commands.spawn(FireflyProjectileBundle {
                projectile: Projectile::new_firefly_bullet(
                    transform.translation.truncate(),
                    velocity,
                ),
                sprite: SpriteBundle {
                    texture,
                    transform,
                    ..default()
                },
                ..default()
            });
        }
    }
}
