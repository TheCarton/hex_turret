use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_asset_loader::loading_state::LoadingStateAppExt;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::PROJECTILE_SPEED;
use crate::constants::TURRET_SIZE;
use crate::controls::spawn_structure_on_click;
use crate::controls::SelectedStructure;
use crate::enemies::Health;
use crate::enemies::Hittable;
use crate::game::AppState;
use crate::hex::cube_linedraw;
use crate::hex::Hex;
use crate::hex::HexControl;
use crate::projectiles::spawn_projectile;
use crate::projectiles::ProjectileType;
use crate::projectiles::TurretProjectileAssets;
use crate::projectiles::Velocity;
use crate::{
    constants::{TURRET_RANGE, TURRET_RELOAD_SECONDS},
    enemies::Seeking,
    hex::{HexFaction, HexMap, HexPosition},
};

pub(crate) struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::AssetLoading)
                .load_collection::<TurretAssets>()
                .load_collection::<AntennaAssets>()
                .load_collection::<FactoryAssets>(),
        )
        .add_systems(
            Update,
            (
                structure_faction_from_hex,
                aim_turrets,
                fire_turrets,
                fire_control_ray,
                rotate_antennae,
                update_factory_energy,
                change_selected_structure_color.after(spawn_structure_on_click),
            )
                .run_if(in_state(AppState::InGame)),
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

#[derive(Component, Default)]
pub(crate) struct Structure;

#[derive(Component, Default, Debug)]
pub(crate) struct ControlVec {
    pub(crate) hexes: Vec<HexPosition>,
    pub(crate) control: HexControl,
}

impl ControlVec {
    fn line(start: HexPosition, end: HexPosition, control: HexControl) -> ControlVec {
        ControlVec {
            hexes: cube_linedraw(start, end),
            control,
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct ControlRay;

#[derive(Component, Default)]
pub(crate) struct RayTimer {
    pub(crate) timer: Timer,
}

#[derive(Bundle, Default)]
pub(crate) struct ControlRayBundle {
    control_ray: ControlRay,
    control_vec: ControlVec,
    velocity: HexPosition,
    timer: RayTimer,
}

#[derive(Bundle, Default)]
pub(crate) struct AntennaBundle {
    pub(crate) antenna: Antenna,

    pub(crate) structure: Structure,
    pub(crate) health: Health,
    pub(crate) faction: HexFaction,
    pub(crate) hittable: Hittable,
    pub(crate) hex_pos: HexPosition,
    pub(crate) spritebundle: SpriteBundle,
    pub(crate) target_point: AimVec,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
    pub(crate) reload_timer: ReloadTimer,
}

fn fire_control_ray(
    mut q_antenna: Query<(&HexPosition, &mut ReloadTimer, &AimVec), (With<Antenna>, Without<Hex>)>,
    q_hex: Query<&HexControl, (With<Hex>, Without<Antenna>)>,
    q_hex_map: Query<&HexMap>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let hex_map = q_hex_map.single();
    for (start, mut reload, aim_vec) in q_antenna.iter_mut() {
        if let Some(aim_point) = aim_vec.v {
            reload.timer.tick(time.delta());
            let hex_entity = hex_map.map.get(start).expect("start is valid hex");
            let hex_control = q_hex.get(*hex_entity).expect("valid entity");
            if reload.timer.finished() {
                let end = HexPosition::from_pixel(aim_point);
                commands.spawn(ControlRayBundle {
                    control_vec: ControlVec {
                        hexes: cube_linedraw(*start, end),
                        control: *hex_control,
                    },
                    timer: RayTimer {
                        timer: Timer::from_seconds(0.5f32, TimerMode::Once),
                    },
                    ..default()
                });
            }
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct FireflyFactory;

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

#[derive(Component, Default)]
pub(crate) struct FactoryEnergy {
    pub(crate) energy: HexControl,
}

#[derive(Bundle, Default)]
pub(crate) struct FactoryBundle {
    pub(crate) fireflyfactory: FireflyFactory,
    pub(crate) structure: Structure,
    pub(crate) hittable: Hittable,
    pub(crate) faction: HexFaction,
    pub(crate) factory_energy: FactoryEnergy,
    pub(crate) health: Health,
    pub(crate) hex_pos: HexPosition,
    pub(crate) prev_animation_state: PrevFactoryState,
    pub(crate) current_animation_state: CurrentFactoryState,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
    pub(crate) sprite: SpriteBundle,
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

fn update_factory_energy(
    mut q_factory: Query<(&Transform, &mut FactoryEnergy)>,
    q_hex: Query<&HexControl>,
    q_hex_map: Query<&HexMap>,
) {
    let hex_map = q_hex_map.single();
    for (transform, mut factory_energy) in q_factory.iter_mut() {
        let hex_pos = HexPosition::from_pixel(transform.translation.truncate());
        let hex_entity = hex_map.map.get(&hex_pos).expect("valid hex pos");
        if let Ok(hex_control) = q_hex.get(*hex_entity) {
            factory_energy.energy = *hex_control;
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct Turret;

#[derive(Bundle, Default)]
pub(crate) struct TurretBundle {
    pub(crate) turret: Turret,
    pub(crate) structure: Structure,
    pub(crate) health: Health,
    pub(crate) hittable: Hittable,
    pub(crate) hex_pos: HexPosition,
    pub(crate) faction: HexFaction,
    pub(crate) sprite: SpriteBundle,
    pub(crate) reload_timer: ReloadTimer,
    pub(crate) aim: AimVec,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct FactoryAssets {
    #[asset(texture_atlas_layout(tile_size_x = 48., tile_size_y = 48., columns = 1, rows = 1))]
    pub(crate) layout: Handle<TextureAtlasLayout>,
    #[asset(path = "firefly_factory_spritesheet.png")]
    pub(crate) factory: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct AntennaAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    pub(crate) layout: Handle<TextureAtlasLayout>,
    #[asset(path = "antenna.png")]
    pub(crate) antenna: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct TurretAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    pub(crate) layout: Handle<TextureAtlasLayout>,
    #[asset(path = "turret.png")]
    pub(crate) turret: Handle<Image>,
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

impl From<f32> for ReloadTimer {
    fn from(value: f32) -> Self {
        ReloadTimer {
            timer: Timer::from_seconds(value, TimerMode::Repeating),
        }
    }
}

fn structure_faction_from_hex(
    mut q_turrets: Query<(&Transform, &mut HexFaction), (With<Structure>, Without<Hex>)>,
    q_hex: Query<&HexFaction, (Without<Structure>, With<Hex>)>,
    q_hex_map: Query<&HexMap>,
) {
    let hex_map = q_hex_map.single();
    for (transform, mut hex_faction) in q_turrets.iter_mut() {
        let hex_entity = hex_map
            .map
            .get(&HexPosition::from_pixel(transform.translation.xy()))
            .unwrap();
        let hex_status = q_hex.get(*hex_entity).unwrap();
        *hex_faction = *hex_status;
    }
}

fn aim_turrets(
    mut q_turrets: Query<(&Transform, &mut AimVec, &HexFaction), (With<Turret>, Without<Seeking>)>,
    q_enemies: Query<(Entity, &Transform), With<Seeking>>,
    q_target: Query<&HexFaction>,
) {
    for (transform, mut aim, turret_faction) in q_turrets.iter_mut() {
        let valid_target = q_enemies
            .iter()
            .filter(|(entity, _target)| {
                let target_faction = q_target.get(*entity).expect("valid entity");
                turret_faction != target_faction
            })
            .map(|(_entity, transform)| {
                (
                    transform.translation,
                    transform.translation.distance(transform.translation),
                )
            })
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).expect("no NaNs"));
        if let Some((target_coords, target_dist)) = valid_target {
            if target_dist < TURRET_RANGE {
                let aim_point =
                    (target_coords.truncate() - transform.translation.truncate()).try_normalize();
                *aim = AimVec { v: aim_point }
            }
        } else {
            *aim = AimVec::default();
        }
    }
}

fn fire_turrets(
    mut commands: Commands,
    mut q_turrets: Query<(&mut Transform, &mut ReloadTimer, &AimVec), With<Turret>>,
    projectile_assets: Res<TurretProjectileAssets>,
    time: Res<Time>,
) {
    for (mut turret, mut reload_timer, aim_vec) in q_turrets.iter_mut() {
        reload_timer.timer.tick(time.delta());

        if let Some(aim_vector) = aim_vec.v {
            let velocity = aim_vector * PROJECTILE_SPEED;
            let rotate_to_enemy = Quat::from_rotation_arc(Vec3::Y, aim_vector.extend(0f32));
            turret.rotation = rotate_to_enemy;

            let projectile_translation =
                turret.translation + (aim_vector * TURRET_SIZE).extend(turret.translation.z);
            if reload_timer.timer.finished() {
                let transform = Transform {
                    translation: projectile_translation,
                    rotation: rotate_to_enemy,
                    scale: Vec3::new(1f32, 1f32, 1f32),
                };
                spawn_projectile(
                    &mut commands,
                    ProjectileType::TurretBullet,
                    velocity,
                    projectile_assets.projectile.clone(),
                    transform,
                );
            }
        }
    }
}

fn rotate_antennae(mut q_antennae: Query<(&mut Transform, &AimVec), With<Antenna>>) {
    for (mut trans, maybe_aim_vec) in q_antennae.iter_mut() {
        if let Some(aim_vec) = maybe_aim_vec.v {
            if let Some(aim_point) = (aim_vec - trans.translation.truncate()).try_normalize() {
                let rotate_to_aim = Quat::from_rotation_arc(Vec3::Y, aim_point.extend(0f32));
                trans.rotation = rotate_to_aim;
            }
        }
    }
}

fn change_selected_structure_color(
    selected_structure: Res<SelectedStructure>,
    mut q_structure: Query<&mut Sprite>,
) {
    if let Some(structure_entity) = selected_structure.curr_structure {
        if let Ok(mut sprite) = q_structure.get_mut(structure_entity) {
            sprite.color.set_r(1f32);
            sprite.color.set_g(0f32);
            sprite.color.set_b(0f32);
        }
        if let Some(structure_entity) = selected_structure.prev_structure {
            if let Ok(mut sprite) = q_structure.get_mut(structure_entity) {
                sprite.color.set_r(1f32);
                sprite.color.set_g(1f32);
                sprite.color.set_b(1f32);
            }
        }
    }
}
