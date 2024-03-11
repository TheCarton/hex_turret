use crate::constants::{
    ENEMY_SIZE, FIREFLY_HIT_ANIMATION_DURATION, PROJECTILE_DAMAGE, PROJECTILE_RANGE,
    PROJECTILE_SIZE,
};
use crate::enemies::{DamagedTime, Enemy, Health, Hit};
use crate::hex::HexPosition;
use crate::turrets::{ControlRay, ControlVec, RayTimer};

use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use derive_more::Add;

pub(crate) struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
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

#[derive(Component, Default)]
pub(crate) struct Projectile;

#[derive(Bundle, Default)]
pub(crate) struct ProjectileBundle {
    pub(crate) projectile: Projectile,
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
    mut q_enemies: Query<(
        &Transform,
        &mut DamagedTime,
        &mut Health,
        With<Enemy>,
        Without<Projectile>,
    )>,
    mut q_projectiles: Query<(&Transform, &mut Hit, With<Projectile>, Without<Enemy>)>,
) {
    for (proj, mut proj_hit, _, _) in &mut q_projectiles {
        for (enemy, mut damage_dur, mut enemy_health, _, _) in &mut q_enemies {
            let proj_hit_enemy = collide(
                enemy.translation,
                ENEMY_SIZE,
                proj.translation,
                PROJECTILE_SIZE,
            )
            .is_some();

            if proj_hit_enemy {
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
    mut q_projectiles: Query<(&mut Transform, &mut Distance, &Velocity, With<Projectile>)>,
    time: Res<Time>,
) {
    for (mut trans, mut dist, vel, _) in &mut q_projectiles {
        let v = Vec3::from(vel) * time.delta_seconds();
        trans.translation += v;
        dist.d += v.length();
    }
}

fn update_control_rays(
    mut q_control_rays: Query<(
        Entity,
        &mut RayTimer,
        &mut ControlVec,
        &HexPosition,
        With<ControlRay>,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity_id, mut ray_time, mut control_vec, hex_vel, _) in q_control_rays.iter_mut() {
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
    q_projectiles: Query<(Entity, &Distance, &Hit, With<Projectile>)>,
) {
    for (entity, dist, hit, _) in &q_projectiles {
        if dist.d > PROJECTILE_RANGE || hit.has_hit {
            commands.entity(entity).despawn_recursive();
        }
    }
}
