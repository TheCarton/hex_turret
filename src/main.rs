use animation::AnimationPlugin;
use bevy::core_pipeline::core_3d::Camera3dDepthLoadOp;
use bevy::sprite::collide_aabb::collide;
use bevy::{prelude::*, window::PrimaryWindow};
use camera::{CameraPluginHexTurret, MainCamera};
use enemies::{
    CurrentFireflyAnimationState, DamagedTime, EnemiesPlugin, Enemy, Firefly,
    FireflyAnimationState, FireflyFactoryTextureAtlas, FireflyTextureAtlas, Health, Hit,
    PrevFireflyAnimationState,
};
use hex::{random_hex, Hex, HexControl, HexMap, HexPlugin, HexPosition, HexStatus};
use player::{Player, PlayerPlugin};
use std::collections::hash_map::RandomState;
use std::ops::Add;
use std::{cmp::Ordering, collections::HashMap};
use turrets::{AimVec, ReloadTimer, Turret, TurretPlugin, TurretTextureAtlas};

mod animation;
mod camera;
mod colors;
mod constants;
mod enemies;
mod entities;
mod hex;
mod player;
mod turrets;

use constants::*;
use entities::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(HexPlugin)
        .add_plugins(CameraPluginHexTurret)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemiesPlugin)
        .add_plugins(TurretPlugin)
        .add_plugins(AnimationPlugin)
        .init_resource::<CursorWorldCoords>()
        .init_resource::<CursorHexPosition>()
        .init_resource::<FireflyTextureAtlas>()
        .init_resource::<TurretTextureAtlas>()
        .init_resource::<FireflyFactoryTextureAtlas>()
        .add_systems(
            Update,
            (
                fire_projectiles,
                move_projectiles,
                despawn_projectiles,
                cursor_system,
                detect_proj_enemy_collision,
            ),
        )
        .run()
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

fn cursor_system(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    mut cursor_hex: ResMut<CursorHexPosition>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_coords.pos = world_position;
        cursor_hex.hex = HexPosition::from_pixel(world_position);
    }
}

fn fire_projectiles(
    mut commands: Commands,
    mut q_turrets: Query<(&mut Transform, &mut ReloadTimer, &AimVec, With<Turret>)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    for (mut turret, mut reload_timer, aim_vec, _) in q_turrets.iter_mut() {
        reload_timer.timer.tick(time.delta());

        if let Some(aim_point) = aim_vec.v {
            let velocity = Velocity {
                v: aim_point * PROJECTILE_SPEED,
            };
            let rotate_to_enemy = Quat::from_rotation_arc(Vec3::Y, aim_point.extend(0f32));
            turret.rotation = rotate_to_enemy;
            if reload_timer.timer.finished() {
                commands.spawn(ProjectileBundle {
                    projectile: Projectile,
                    velocity,
                    sprite: SpriteBundle {
                        texture: asset_server.load("projectile.png"),
                        transform: *turret,
                        ..default()
                    },
                    ..default()
                });
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
