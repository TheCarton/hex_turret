use bevy::prelude::*;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::PROJECTILE_SPEED;
use crate::controls::CursorHexPosition;
use crate::projectiles::Projectile;
use crate::projectiles::ProjectileBundle;
use crate::projectiles::Velocity;
use crate::{
    constants::{TURRET_RANGE, TURRET_RELOAD_SECONDS},
    enemies::Enemy,
    hex::{HexMap, HexPosition, HexStatus},
};

pub(crate) struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurretTextureAtlas>();
        app.add_systems(
            Update,
            (
                turret_status_from_hex,
                spawn_turret_on_click,
                aim_turrets,
                fire_turrets,
            ),
        );
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

#[derive(Component, Default)]
pub(crate) struct Turret;

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
            HexStatus::Blue => TurretStatus::Friendly,
            HexStatus::Neutral => TurretStatus::Neutral,
            HexStatus::Red => TurretStatus::Friendly,
        };
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
            .is_ok_and(|hex_status| hex_status != &HexStatus::Neutral)
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

fn fire_turrets(
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
