use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::config::ConfigureLoadingState;
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_asset_loader::loading_state::LoadingStateAppExt;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::HEX_DIRECTIONS;
use crate::constants::PROJECTILE_SPEED;
use crate::constants::TURRET_SIZE;
use crate::controls::spawn_structure_on_click;
use crate::controls::SelectedStructure;
use crate::enemies::Health;
use crate::enemies::Hittable;
use crate::game::AppState;
use crate::game::UpdateInGameSet;
use crate::hex::Hex;
use crate::hex::HexControl;
use crate::hex::HexDirection;
use crate::hex::MIN_HEX_CONTROL;
use crate::projectiles::spawn_projectile;
use crate::projectiles::ProjectileType;
use crate::projectiles::TurretProjectileAssets;
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
                .load_collection::<EnergySourceAssets>()
                .load_collection::<FactoryAssets>(),
        )
        .add_systems(
            Update,
            (
                structure_faction_from_hex,
                aim_turrets,
                fire_turrets,
                spawn_control_ray,
                despawn_decayed_control_rays,
                rotate_antennae,
                update_factory_energy,
                generate_energy,
                change_selected_structure_color.after(spawn_structure_on_click),
            )
                .in_set(UpdateInGameSet),
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
pub(crate) enum Structure {
    #[default]
    Turret,
    Factory,
    Antenna,
}

impl Structure {
    pub(crate) fn string(&self) -> String {
        match self {
            Structure::Turret => "Turret",
            Structure::Factory => "Factory",
            Structure::Antenna => "Antenna",
        }
        .to_string()
    }
}

#[derive(Component, Default, Debug)]
pub(crate) struct ControlVec {
    pub(crate) hexes: Vec<HexPosition>,
    pub(crate) control: HexControl,
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

#[derive(Component, Default)]
pub(crate) struct EnergySource {
    pub(crate) flow_rate: f32,
}

impl EnergySource {
    fn to_hex_control(&self) -> HexControl {
        HexControl {
            red: 0f32,
            blue: 0f32,
            neutral: self.flow_rate,
        }
    }
}

#[derive(AssetCollection, Resource)]
pub(crate) struct EnergySourceAssets {
    #[asset(texture_atlas_layout(tile_size_x = 128., tile_size_y = 128., columns = 1, rows = 1))]
    #[asset(path = "coil_gun_path_1.png")]
    pub(crate) energy_source: Handle<Image>,
}

#[derive(Bundle)]
pub(crate) struct EnergySourceBundle {
    pub(crate) energy_source: EnergySource,
    pub(crate) hex_pos: HexPosition,
    pub(crate) spritebundle: SpriteBundle,
    pub(crate) reload_timer: ReloadTimer,
}

#[derive(Bundle)]
pub(crate) struct AntennaBundle {
    pub(crate) antenna: Antenna,
    pub(crate) face: HexDirection,
    pub(crate) icon: StructureIcon,
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

impl Default for AntennaBundle {
    fn default() -> Self {
        AntennaBundle {
            icon: StructureIcon::FactoryIcon,
            face: HexDirection::E,
            structure: Structure::Antenna,
            hittable: Hittable::default(),
            faction: HexFaction::Neutral,
            health: Health::default(),
            hex_pos: HexPosition::default(),
            animation_indices: AnimationIndices::default(),
            animation_timer: AnimationTimer::default(),
            antenna: Antenna,
            spritebundle: SpriteBundle::default(),
            target_point: AimVec::default(),
            reload_timer: ReloadTimer::default(),
        }
    }
}

fn spawn_control_ray(
    mut q_antenna: Query<
        (&HexPosition, &HexDirection, &mut ReloadTimer),
        (With<Antenna>, Without<Hex>),
    >,
    mut q_hex: Query<(Entity, &mut HexControl), (With<Hex>, Without<Antenna>)>,
    q_hex_map: Query<&HexMap>,
    time: Res<Time>,
) {
    let hex_map = q_hex_map.single();
    for (antenna_pos, antenna_face, mut reload_timer) in q_antenna.iter_mut() {
        let Ok((_, my_antenna_hc)) = q_hex.get(*hex_map.map.get(antenna_pos).expect("valid pos"))
        else {
            continue;
        };
        let antenna_hc = my_antenna_hc.clone();
        reload_timer.timer.tick(time.delta());
        if reload_timer.timer.finished() {
            for i in 1..4 {
                let hex_pos = antenna_face.to_hex() * i + *antenna_pos;
                let Some(hex_entity) = hex_map.map.get(&hex_pos) else {
                    break;
                };
                let (_, mut hc) = q_hex
                    .get_mut(*hex_entity)
                    .expect("valid entity from hex map");

                *hc += antenna_hc;
            }
        }
    }
}

fn generate_energy(
    mut q_sources: Query<
        (&EnergySource, &HexPosition, &mut ReloadTimer),
        (With<EnergySource>, Without<Hex>),
    >,
    mut q_hex: Query<(Entity, &mut HexControl), (With<Hex>, Without<EnergySource>)>,
    q_hex_map: Query<&HexMap>,
    time: Res<Time>,
) {
    let hex_map = q_hex_map.single();
    for (es, hex_pos, mut reload_timer) in q_sources.iter_mut() {
        reload_timer.timer.tick(time.delta());
        if reload_timer.timer.finished() {
            for delta in HEX_DIRECTIONS {
                let p = *hex_pos + delta;
                if let Some(entity) = hex_map.map.get(&p) {
                    let (_, mut hc) = q_hex.get_mut(*entity).expect("valid entity from hex map");
                    *hc += es.to_hex_control();
                }
            }
        }
    }
}

fn despawn_decayed_control_rays(
    q_rays: Query<(Entity, &ControlVec), With<ControlRay>>,
    mut commands: Commands,
) {
    for (control_entity, control_vec) in q_rays.iter() {
        if control_vec.control == MIN_HEX_CONTROL {
            commands.entity(control_entity).despawn();
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct FireflyFactory;

#[derive(Component, Default)]
pub(crate) struct FactoryEnergy {
    pub(crate) energy: HexControl,
}

#[derive(Bundle)]
pub(crate) struct FactoryBundle {
    pub(crate) fireflyfactory: FireflyFactory,
    pub(crate) icon: StructureIcon,
    pub(crate) structure: Structure,
    pub(crate) hittable: Hittable,
    pub(crate) faction: HexFaction,
    pub(crate) factory_energy: FactoryEnergy,
    pub(crate) health: Health,
    pub(crate) hex_pos: HexPosition,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
    pub(crate) sprite: SpriteBundle,
    pub(crate) build_timer: BuildTimer,
}

impl Default for FactoryBundle {
    fn default() -> Self {
        FactoryBundle {
            fireflyfactory: FireflyFactory::default(),
            icon: StructureIcon::FactoryIcon,
            structure: Structure::Factory,
            hittable: Hittable::default(),
            faction: HexFaction::Neutral,
            factory_energy: FactoryEnergy::default(),
            health: Health::default(),
            hex_pos: HexPosition::default(),
            animation_indices: AnimationIndices::default(),
            animation_timer: AnimationTimer::default(),
            sprite: SpriteBundle::default(),
            build_timer: BuildTimer::default(),
        }
    }
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

#[derive(Component, Default)]
#[allow(dead_code)]
pub(crate) enum StructureIcon {
    #[default]
    TurretIcon,
    AntennaIcon,
    FactoryIcon,
    NoStructureIcon,
}

#[derive(Bundle)]
pub(crate) struct TurretBundle {
    pub(crate) turret: Turret,
    pub(crate) icon: StructureIcon,
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

impl Default for TurretBundle {
    fn default() -> Self {
        TurretBundle {
            turret: Turret::default(),
            icon: StructureIcon::TurretIcon,
            structure: Structure::default(),
            hittable: Hittable::default(),
            faction: HexFaction::Neutral,
            health: Health::default(),
            hex_pos: HexPosition::default(),
            animation_indices: AnimationIndices::default(),
            animation_timer: AnimationTimer::default(),
            sprite: SpriteBundle::default(),
            reload_timer: ReloadTimer::default(),
            aim: AimVec::default(),
        }
    }
}
#[derive(AssetCollection, Resource)]
pub(crate) struct FactoryAssets {
    #[asset(texture_atlas_layout(tile_size_x = 48., tile_size_y = 48., columns = 1, rows = 1))]
    #[asset(path = "firefly_factory_spritesheet.png")]
    pub(crate) factory: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct AntennaAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    #[asset(path = "antenna.png")]
    pub(crate) antenna: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct TurretAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
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

fn rotate_antennae(
    mut q_antennae: Query<(&mut Transform, &HexDirection), (With<Antenna>, Changed<HexDirection>)>,
) {
    for (mut trans, hex_direction) in q_antennae.iter_mut() {
        let antenna_hex = HexPosition::from_pixel(trans.translation.truncate());
        let aim_hex = antenna_hex + hex_direction.to_hex();
        let maybe_aim_vec = (aim_hex.pixel_coords() - trans.translation.truncate()).try_normalize();
        if let Some(aim_vec) = maybe_aim_vec {
            let rotate_to_aim = Quat::from_rotation_arc(Vec3::Y, aim_vec.extend(0f32));
            trans.rotation = rotate_to_aim;
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
