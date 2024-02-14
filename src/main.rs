use bevy::sprite::collide_aabb::collide;
use bevy::time::Stopwatch;
use bevy::{prelude::*, window::PrimaryWindow};
use std::{cmp::Ordering, collections::HashMap};

mod colors;
mod constants;
mod entities;
mod init;

use constants::*;
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
        .init_resource::<FireflyTextureAtlas>()
        .init_resource::<TurretTextureAtlas>()
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
                animate_sprite,
                spawn_fireflies,
                update_firefly_animation_state,
                update_firefly_animation,
                spawn_turret_on_click,
                fire_projectiles,
                aim_turrets,
                turret_status_from_hex,
                move_projectiles,
                despawn_projectiles,
                despawn_dead_enemies,
                despawn_hit_enemies,
                update_hexes,
                render_hexes,
                cursor_system,
                detect_proj_enemy_collision,
                detect_enemy_player_collision,
            ),
        )
        .run()
}

fn despawn_dead_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Health, With<Enemy>)>) {
    for (enemy_entity, health, _) in &q_enemies {
        if health.hp <= 0f32 {
            commands.entity(enemy_entity).despawn();
        }
    }
}

fn despawn_hit_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Hit, With<Enemy>)>) {
    for (enemy_entity, hit, _) in &q_enemies {
        if hit.has_hit {
            commands.entity(enemy_entity).despawn();
        }
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

fn detect_enemy_player_collision(
    mut q_enemies: Query<(&Transform, &mut Hit, With<Enemy>, Without<Player>)>,
    q_player: Query<(&Transform, With<Player>, Without<Enemy>)>,
) {
    let (player, _, _) = q_player.single();
    for (enemy, mut damaged_time, _, _) in &mut q_enemies {
        damaged_time.has_hit = collide(
            enemy.translation,
            ENEMY_SIZE,
            player.translation,
            PLAYER_SIZE,
        )
        .is_some();
    }
}

fn update_firefly_animation_state(
    mut q_fireflies: Query<(
        &mut CurrentFireflyAnimationState,
        &mut PrevFireflyAnimationState,
        &mut DamagedTime,
        With<Firefly>,
    )>,
    time: Res<Time>,
) {
    //TODO: Fix logic for transition from normal animation cycle to hit animation cycle. We're going to
    // incorrect animation indices right now.
    for (mut animation_state, mut prev_animation_state, mut hit_timer, _) in q_fireflies.iter_mut()
    {
        if let Some(timer) = &mut hit_timer.time {
            timer.tick(time.delta());
            if timer.finished() {
                *animation_state = CurrentFireflyAnimationState {
                    state: FireflyAnimationState::Normal,
                };
                *prev_animation_state = PrevFireflyAnimationState {
                    state: FireflyAnimationState::Damaged,
                };
                hit_timer.time = None;
            } else {
                *animation_state = CurrentFireflyAnimationState {
                    state: FireflyAnimationState::Damaged,
                };
            }
        }
    }
}

fn update_firefly_animation(
    mut q_fireflies: Query<(
        &CurrentFireflyAnimationState,
        &mut PrevFireflyAnimationState,
        &mut AnimationIndices,
        Changed<CurrentFireflyAnimationState>,
    )>,
) {
    for (curr_anim, mut prev_anim, mut indices, _) in q_fireflies.iter_mut() {
        if curr_anim.state != prev_anim.state {
            *indices = match curr_anim.state {
                FireflyAnimationState::Normal => AnimationIndices::new(0, 3),
                FireflyAnimationState::Damaged => AnimationIndices::new(16, 19),
            };
            prev_anim.state = curr_anim.state;
        }
    }
}

fn spawn_turret_on_click(
    mut commands: Commands,
    q_hex: Query<&HexStatus>,
    q_hex_map: Query<&HexMap>,
    turret_sprite_sheet: Res<TurretTextureAtlas>,
    cursor_hex: Res<CursorHexPosition>,
    buttons: Res<Input<MouseButton>>,
) {
    let hex_map = q_hex_map.single();
    if buttons.just_pressed(MouseButton::Left) && hex_map.contains(cursor_hex.hex) {
        let hex_entity = hex_map.map.get(&cursor_hex.hex);
        if q_hex
            .get(*hex_entity.unwrap())
            .is_ok_and(|hex_status| hex_status != &HexStatus::Unoccupied)
        {
            return;
        }
        let turret_v = cursor_hex.hex.pixel_coords();
        commands.spawn(TurretBundle {
            turret: Turret,
            sprite: SpriteSheetBundle {
                texture_atlas: turret_sprite_sheet.atlas.clone(),
                transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                ..default()
            },
            ..default()
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
        self.map.contains_key(&hex)
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

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut indices, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = indices.next_index();
        }
    }
}

fn spawn_fireflies(
    mut commands: Commands,
    firefly_sprite_sheet: Res<FireflyTextureAtlas>,
    time: Res<Time>,
    mut config: ResMut<EnemySpawnConfig>,
) {
    config.timer.tick(time.delta());
    if config.timer.finished() {
        let mut anim_indices = AnimationIndices::new(0, 3);
        commands.spawn(FireflyBundle {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(anim_indices.next_index()),
                texture_atlas: firefly_sprite_sheet.atlas.clone(),
                transform: Transform::from_xyz(0f32, 0f32, 2f32),
                ..default()
            },
            animation_indices: anim_indices,
            ..default()
        });
    }
}

fn aim_turrets(
    mut q_turrets: Query<(
        &Transform,
        &mut AimVec,
        &TurretStatus,
        With<Turret>,
        Without<Enemy>,
    )>,
    q_enemies: Query<(&Transform, With<Enemy>)>,
) {
    for (turret, mut aim, status, _, _) in q_turrets.iter_mut() {
        let closest_enemy = q_enemies
            .iter()
            .map(|(enemy, _)| {
                (
                    enemy.translation,
                    enemy.translation.distance(turret.translation),
                )
            })
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).expect("no NaNs"));
        if closest_enemy.is_some_and(|(_, dist)| dist < TURRET_RANGE)
            && *status == TurretStatus::Friendly
        {
            let (target, _) = closest_enemy.unwrap();
            let aim_point = (target.truncate() - turret.translation.truncate()).try_normalize();
            *aim = AimVec { v: aim_point }
        } else {
            *aim = AimVec::default();
        }
    }
}

fn turret_status_from_hex(
    mut q_turrets: Query<(&Transform, &mut TurretStatus, With<Turret>)>,
    q_hex: Query<&HexStatus>,
    q_hex_map: Query<&HexMap>,
) {
    let hex_map = q_hex_map.single();
    for (turret, mut turret_status, _) in q_turrets.iter_mut() {
        let hex_entity = hex_map
            .map
            .get(&HexPosition::from_pixel(turret.translation.xy()))
            .unwrap();
        let hex_status = q_hex.get(*hex_entity).unwrap();
        *turret_status = match hex_status {
            HexStatus::Occupied => TurretStatus::Friendly,
            HexStatus::Unoccupied => TurretStatus::Neutral,
            HexStatus::Selected => TurretStatus::Friendly,
        };
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

fn move_enemy(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
    time: Res<Time>,
) {
    let player_transform = param_set.p0().single().clone();
    for mut enemy_transform in param_set.p1().iter_mut() {
        if let Some(n) =
            (player_transform.translation - enemy_transform.translation).try_normalize()
        {
            let mut v = n * ENEMY_SPEED * time.delta_seconds();
            v.z = 0f32;
            enemy_transform.translation += v;
        }
    }
}

fn update_hexes(
    player_hex_query: Query<&HexPosition, With<Player>>,
    mut hex_query: Query<(&HexPosition, &mut HexStatus)>,
) {
    let player_hex = player_hex_query.single();
    for (hex_pos, mut hex_status) in hex_query.iter_mut() {
        let is_player_hex = hex_pos == player_hex;
        let is_neighbor = HEX_DIRECTIONS
            .map(|delta| *hex_pos + delta)
            .iter()
            .any(|&n| n == *player_hex);
        match (is_player_hex, is_neighbor) {
            (true, _) => *hex_status = HexStatus::Occupied,
            (false, true) => *hex_status = HexStatus::Selected,
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
