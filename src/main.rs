use bevy::{prelude::*, window::PrimaryWindow};
use std::{cmp::Ordering, collections::HashMap};

mod colors;
mod constants;
mod entities;
mod init;

use constants::{
    ENEMY_SPEED, HEX_DIRECTIONS, HEX_SIZE, PLAYER_SPEED, PROJECTILE_RANGE, PROJECTILE_SPEED,
    TRIGGER_RANGE, TURRET_RANGE,
};
use entities::*;
use init::*;
use itertools::Itertools;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<CursorWorldCoords>()
        .init_resource::<CursorHexPosition>()
        .add_systems(
            Startup,
            (
                setup,
                spawn_map,
                spawn_player,
                setup_enemy_spawning,
                apply_deferred,
                populate_map,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                move_player,
                move_enemy,
                spawn_enemies,
                spawn_turret_on_click,
                fire_projectiles,
                move_projectiles,
                despawn_out_of_range_projectiles,
                explode_enemies,
                update_hexes,
                render_hexes,
                cursor_system,
            ),
        )
        .run()
}

fn spawn_turret_on_click(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cursor_hex: Res<CursorHexPosition>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let turret_v = cursor_hex.hex.pixel_coords();
        commands.spawn(TurretBundle {
            turret: Turret,
            pos: cursor_hex.hex,
            sprite: SpriteBundle {
                texture: asset_server.load("turret.png"),
                transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                ..default()
            },
        });
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

impl HexMap {
    fn contains(&self, hex: HexPosition) -> bool {
        let d = [hex.q, hex.r, hex.s()]
            .into_iter()
            .map(|v| v.abs())
            .max()
            .expect("hex has position.");
        d <= self.size
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_hex_query: Query<&mut HexPosition, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_transform_query.single_mut();
    let direction = match keyboard_input.get_pressed().last() {
        Some(KeyCode::Left) | Some(KeyCode::A) => Vec3::new(-1.0, 0.0, 0.0),
        Some(KeyCode::Right) | Some(KeyCode::D) => Vec3::new(1.0, 0.0, 0.0),
        Some(KeyCode::Up) | Some(KeyCode::W) => Vec3::new(0.0, 1.0, 0.0),
        Some(KeyCode::Down) | Some(KeyCode::S) => Vec3::new(0.0, -1.0, 0.0),
        _ => Vec3::ZERO,
    };

    let new_player_pos =
        player_transform.translation + direction * PLAYER_SPEED * time.delta_seconds();

    let new_hex = HexPosition::from_pixel(Vec2::new(new_player_pos.x, new_player_pos.y));
    let mut player_hex = player_hex_query.single_mut();
    *player_hex = new_hex;
    player_transform.translation = new_player_pos;
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut config: ResMut<EnemySpawnConfig>,
) {
    config.timer.tick(time.delta());
    if config.timer.finished() {
        commands.spawn(EnemyBundle {
            enemy: Enemy,
            pos: HexPosition::default(),
            sprite: SpriteBundle {
                texture: asset_server.load("enemy.png"),
                transform: Transform::from_xyz(0f32, 0f32, 2f32),
                ..default()
            },
        });
    }
}

fn fire_projectiles(
    mut commands: Commands,
    q_turrets: Query<&Transform, With<Turret>>,
    q_enemies: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    for turret in q_turrets.iter() {
        let closest_enemy = q_enemies
            .iter()
            .map(|enemy| {
                (
                    enemy.translation,
                    enemy.translation.distance(turret.translation),
                )
            })
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).expect("no NaNs"));
        if let Some((target, dist)) = closest_enemy {
            if dist < TURRET_RANGE {
                let x = (target - turret.translation)
                    .try_normalize()
                    .unwrap_or(Vec3 {
                        x: 1f32,
                        y: 0f32,
                        z: 0f32,
                    });
                let velocity = Velocity {
                    v: x.truncate() * PROJECTILE_SPEED,
                };
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

fn despawn_out_of_range_projectiles(
    mut commands: Commands,
    q_projectiles: Query<(Entity, &Distance, With<Projectile>)>,
) {
    for (entity, dist, _) in &q_projectiles {
        if dist.d > PROJECTILE_RANGE {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn move_enemy(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
    time: Res<Time>,
) {
    let player_transform = param_set.p0().single().clone();
    for mut enemy_transform in param_set.p1().iter_mut() {
        dbg!(&enemy_transform.translation);
        if let Some(n) =
            (player_transform.translation - enemy_transform.translation).try_normalize()
        {
            let mut v = n * ENEMY_SPEED * time.delta_seconds();
            v.z = 0f32;
            enemy_transform.translation += v;
        }
        dbg!(&enemy_transform.translation);
    }
}

fn explode_enemies(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(Entity, &Transform, With<Enemy>)>,
    )>,
) {
    let player_pos = param_set.p0().single().translation.clone();
    for (enemy_entity, enemy_transform, _) in param_set.p1().iter_mut() {
        if (player_pos - enemy_transform.translation).length() < TRIGGER_RANGE {
            commands.entity(enemy_entity).despawn();
        }
    }
}

fn update_hexes(
    player_hex_query: Query<&HexPosition, With<Player>>,
    mut hex_query: Query<(&HexPosition, &mut HexStatus)>,
    cursor_hex: Res<CursorHexPosition>,
) {
    let player_hex = player_hex_query.single();
    for (hex_pos, mut hex_status) in hex_query.iter_mut() {
        let is_player_hex = hex_pos == player_hex;
        let is_neighbor = HEX_DIRECTIONS
            .map(|delta| *hex_pos + delta)
            .iter()
            .any(|&n| n == *player_hex);
        let is_cursor = *hex_pos == cursor_hex.hex;
        match (is_player_hex, is_neighbor, is_cursor) {
            (true, _, _) => *hex_status = HexStatus::Occupied,
            (false, true, _) => *hex_status = HexStatus::Selected,
            (_, _, true) => *hex_status = HexStatus::Selected,
            _ => *hex_status = HexStatus::Unoccupied,
        }
    }
}

fn render_hexes(
    mut hex_query: Query<(&HexStatus, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
) {
    for (hex_status, mut image_handle) in hex_query.iter_mut() {
        match hex_status {
            HexStatus::Occupied => *image_handle = asset_server.load("red_hex.png"),
            HexStatus::Unoccupied => *image_handle = asset_server.load("blue_hex.png"),
            HexStatus::Selected => *image_handle = asset_server.load("orange_hex.png"),
        }
    }
}
