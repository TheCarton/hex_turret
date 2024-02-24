use bevy::prelude::*;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::PROJECTILE_SPEED;
use crate::hex::HexDirection;
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
        app.init_resource::<FireflyFactoryTextureAtlas>();
        app.init_resource::<AntennaTextureAtlas>();
        app.add_systems(
            Update,
            (
                turret_status_from_hex,
                aim_turrets,
                fire_turrets,
                rotate_antennae,
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
pub(crate) struct Antenna;

#[derive(Bundle, Default)]
pub(crate) struct AntennaBundle {
    pub(crate) antenna: Antenna,
    pub(crate) hex_pos: HexPosition,
    pub(crate) sprite: SpriteSheetBundle,
    pub(crate) face_vec: HexDirection,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(Resource)]
pub(crate) struct AntennaTextureAtlas {
    pub(crate) atlas: Handle<TextureAtlas>,
}

impl FromWorld for AntennaTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture_handle = asset_server.load("antenna.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(64f32, 64f32), 1, 1, None, None);
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        AntennaTextureAtlas {
            atlas: texture_atlas_handle,
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct FireflyFactory;

#[derive(Resource)]
pub(crate) struct FireflyFactoryTextureAtlas {
    pub(crate) atlas: Handle<TextureAtlas>,
}

#[derive(Component, Default)]
pub(crate) struct PrevFactoryState {
    state: FactoryAnimationState,
}

#[derive(Component, Default)]
pub(crate) struct CurrentFactoryState {
    state: FactoryAnimationState,
}

#[derive(Default)]
pub(crate) enum FactoryAnimationState {
    #[default]
    Idle,
    Opening,
    Open,
    Malfunctioning,
}

#[derive(Bundle, Default)]
pub(crate) struct FireflyFactoryBundle {
    pub(crate) fireflyfactory: FireflyFactory,
    pub(crate) hex_pos: HexPosition,
    pub(crate) prev_animation_state: PrevFactoryState,
    pub(crate) current_animation_state: CurrentFactoryState,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
    pub(crate) sprite: SpriteSheetBundle,
    pub(crate) build_timer: BuildTimer,
}

#[derive(Resource)]
pub(crate) struct FactorySpawnConfig {
    pub(crate) timer: Timer,
}
#[derive(Component)]
pub(crate) struct BuildTimer {
    pub(crate) timer: Timer,
}

impl Default for BuildTimer {
    fn default() -> Self {
        BuildTimer {
            timer: Timer::from_seconds(5f32, TimerMode::Repeating),
        }
    }
}

impl FromWorld for FireflyFactoryTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture_handle = asset_server.load("firefly_factory_spritesheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(48f32, 48f32), 1, 1, None, None);
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        FireflyFactoryTextureAtlas {
            atlas: texture_atlas_handle,
        }
    }
}

#[derive(Component, Default, Eq, PartialEq)]
pub(crate) enum TurretStatus {
    #[default]
    Neutral,
    Friendly,
    Hostile,
}

#[derive(Component, Default)]
pub(crate) struct Turret;

#[derive(Bundle, Default)]
pub(crate) struct TurretBundle {
    pub(crate) turret: Turret,
    pub(crate) hex_pos: HexPosition,
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
        dbg!(&hex_map.map);
        let turret_hex = HexPosition::from_pixel(turret.translation.xy());
        dbg!(turret_hex);
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

fn rotate_antennae(
    mut q_antennae: Query<(
        &mut Transform,
        &HexPosition,
        &HexDirection,
        With<Antenna>,
        Changed<HexDirection>,
    )>,
) {
    for (mut trans, hex_pos, hex_dir, _, _) in q_antennae.iter_mut() {
        let aim_hex = *hex_pos + hex_dir.to_hex();
        let rotate_to_aim = Quat::from_rotation_arc(Vec3::Y, aim_hex.pixel_coords().extend(0f32));
        trans.rotation = rotate_to_aim;
    }
}
