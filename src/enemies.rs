use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;

use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::FIREFLY_BULLET_SCALE;
use crate::constants::FIREFLY_RANGE;
use crate::constants::PROJECTILE_SPEED;
use crate::game::AppState;
use crate::hex::Hex;
use crate::hex::HexFaction;
use crate::projectiles::spawn_projectile;
use crate::projectiles::FireflyProjectileAssets;
use crate::projectiles::ProjectileType;
use crate::turrets::BuildTimer;
use crate::turrets::FactoryEnergy;
use crate::turrets::FireflyFactory;
use crate::turrets::ReloadTimer;
use crate::{
    constants::{FIREFLY_HEALTH, FIREFLY_SIZE, FIREFLY_SPEED, PLAYER_SIZE},
    player::Player,
};

pub(crate) struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::AssetLoading).load_collection::<FireflyAssets>(),
        );
        app.add_systems(
            Update,
            (
                spawn_fireflies,
                firefly_targeting,
                fire_firefly_projectiles,
                update_firefly_hit_state,
                move_seeking_units,
                despawn_dead_mortals,
                detect_enemy_player_collision,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component, Default)]
pub(crate) struct Firefly;

#[derive(Component, Default)]
pub(crate) struct Hittable {
    pub(crate) hitbox: Vec2,
    pub(crate) hit: bool,
}

impl Hittable {
    pub(crate) fn from_hitbox(hitbox: Vec2) -> Hittable {
        Hittable { hitbox, hit: false }
    }
}

#[derive(Component, Default, Debug)]
pub(crate) struct Target {
    entity: Option<Entity>,
}

#[derive(Bundle, Default)]
pub(crate) struct FireflyBundle {
    pub(crate) firefly: Firefly,
    pub(crate) hittable: Hittable,
    pub(crate) seeking: Seeking,
    pub(crate) faction: HexFaction,
    pub(crate) animation_state: CurrentFireflyAnimationState,
    pub(crate) target: Target,
    pub(crate) reload_timer: ReloadTimer,
    pub(crate) prev_animation_state: PrevFireflyAnimationState,
    pub(crate) hit: Hit,
    pub(crate) damaged_time: DamagedTime,
    pub(crate) health: Health,
    pub(crate) sprite_bundle: SpriteBundle,
    pub(crate) texture_atlas: TextureAtlas,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub(crate) enum FireflyAnimationState {
    #[default]
    Normal,
    Damaged,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct FireflyAssets {
    #[asset(texture_atlas_layout(tile_size_x = 48., tile_size_y = 48., columns = 8, rows = 3))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(image(sampler = nearest))]
    #[asset(path = "firefly_spritesheet.png")]
    firefly: Handle<Image>,
}

#[derive(Component, Default)]
pub(crate) struct PrevFireflyAnimationState {
    pub(crate) state: FireflyAnimationState,
}

#[derive(Component, Default)]
pub(crate) struct CurrentFireflyAnimationState {
    pub(crate) state: FireflyAnimationState,
}

#[derive(Component)]
pub(crate) struct DamagedTime {
    pub(crate) time: Option<Timer>,
}

impl Default for DamagedTime {
    fn default() -> Self {
        DamagedTime { time: None }
    }
}

#[derive(Component, Default)]
pub(crate) struct Seeking;

#[derive(Component, Default)]
pub(crate) struct Hit {
    pub(crate) has_hit: bool,
}

#[derive(Component)]
pub(crate) struct Health {
    pub(crate) hp: f32,
}

impl From<f32> for Health {
    fn from(value: f32) -> Self {
        Health { hp: value }
    }
}

impl Default for Health {
    fn default() -> Self {
        Health { hp: FIREFLY_HEALTH }
    }
}

fn move_seeking_units(
    q_seeking: Query<(Entity, &Target), With<Seeking>>,
    mut param_set: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
    time: Res<Time>,
) {
    for (seeking_entity, target) in q_seeking.iter() {
        if let Some(target_entity) = target.entity {
            let unit_translation = param_set
                .p0()
                .get(seeking_entity)
                .expect("valid entity")
                .translation;
            if let Ok(target) = param_set.p0().get(target_entity) {
                if (unit_translation).distance(target.translation) > FIREFLY_RANGE {
                    let n = (target.translation - unit_translation).normalize();
                    let mut v = n * FIREFLY_SPEED * time.delta_seconds();
                    v.z = 0f32;
                    let new_unit_translation = unit_translation + v;
                    param_set
                        .p1()
                        .get_mut(seeking_entity)
                        .expect("valid entity")
                        .translation = new_unit_translation;
                }
            }
        }
    }
}

fn update_firefly_hit_state(mut q_fireflies: Query<(&mut DamagedTime, &mut Hittable)>) {
    for (mut damage_time, mut hittable) in q_fireflies.iter_mut() {
        if hittable.hit {
            *damage_time = DamagedTime {
                time: Some(Timer::from_seconds(0.5f32, TimerMode::Once)),
            };
            hittable.hit = false;
        }
    }
}

fn despawn_dead_mortals(mut commands: Commands, q_mortal: Query<(Entity, &Health)>) {
    for (mortal_entity, health) in &q_mortal {
        if health.hp <= 0f32 {
            commands.entity(mortal_entity).despawn();
        }
    }
}

fn detect_enemy_player_collision(
    mut q_enemies: Query<(&Transform, &mut Hit), (With<Seeking>, Without<Player>)>,
    q_player: Query<&Transform, (With<Player>, Without<Seeking>)>,
) {
    let player = q_player.single();
    for (enemy, mut enemy_collision) in &mut q_enemies {
        let collision = Aabb2d::new(enemy.translation.truncate(), FIREFLY_SIZE / 2f32).intersects(
            &Aabb2d::new(player.translation.truncate(), PLAYER_SIZE / 2f32),
        );
        enemy_collision.has_hit = collision;
    }
}

pub const FIREFLY_ENERGY_COST: f32 = 15f32;

fn spawn_fireflies(
    mut commands: Commands,
    firefly_assets: Res<FireflyAssets>,
    time: Res<Time>,
    mut q_factories: Query<(&Transform, &mut BuildTimer, &mut FactoryEnergy), With<FireflyFactory>>,
) {
    for (factory, mut build_timer, mut factory_energy) in q_factories.iter_mut() {
        build_timer.timer.tick(time.delta());
        if build_timer.timer.finished() {
            let p = Vec3::new(factory.translation.x, factory.translation.y, 2f32);
            let anim_indices = AnimationIndices::firefly_indices();
            let faction = factory_energy.energy.max_status();
            factory_energy.energy[faction] -= FIREFLY_ENERGY_COST;
            commands.spawn(FireflyBundle {
                faction,
                hittable: Hittable::from_hitbox(FIREFLY_SIZE),
                sprite_bundle: SpriteBundle {
                    texture: firefly_assets.firefly.clone(),
                    transform: Transform::from_translation(p),
                    ..default()
                },
                texture_atlas: TextureAtlas::from(firefly_assets.layout.clone()),
                animation_indices: anim_indices,
                ..default()
            });
        }
    }
}

fn firefly_targeting(
    q_firefly: Query<(Entity, &Transform, &HexFaction), With<Firefly>>,
    mut param_set: ParamSet<(
        Query<(Entity, &Transform, &HexFaction), Without<Hex>>,
        Query<&mut Target>,
    )>,
) {
    for (firefly_entity, firefly_transform, firefly_faction) in q_firefly.iter() {
        if let Some((closest_target, _target_dist)) = param_set
            .p0()
            .iter()
            .filter(|(_, _, faction)| firefly_faction != *faction)
            .map(|(entity, x, _)| {
                (
                    entity,
                    x.translation.distance(firefly_transform.translation),
                )
            })
            .min_by(|(_, x), (_, y)| x.total_cmp(y))
        {
            *param_set
                .p1()
                .get_mut(firefly_entity)
                .expect("valid entity") = Target {
                entity: Some(closest_target),
            };
        }
    }
}

fn fire_firefly_projectiles(
    mut commands: Commands,
    mut q_fireflies: Query<(&Transform, &Target, &mut ReloadTimer), With<Firefly>>,
    q_target: Query<&Transform>,
    projectile_assets: Res<FireflyProjectileAssets>,
    time: Res<Time>,
) {
    for (firefly_transform, target, mut reload_timer) in q_fireflies.iter_mut() {
        reload_timer.timer.tick(time.delta());
        if reload_timer.timer.finished() {
            let maybe_target_transform = target.entity.map(|e| q_target.get(e).ok()).flatten();
            if let Some(target_transform) = maybe_target_transform {
                if target_transform
                    .translation
                    .distance(firefly_transform.translation)
                    < FIREFLY_RANGE
                {
                    let aim_vector = (target_transform.translation.truncate()
                        - firefly_transform.translation.truncate())
                    .normalize();
                    let velocity = aim_vector * PROJECTILE_SPEED;
                    let rotate_to_target =
                        Quat::from_rotation_arc(Vec3::Y, aim_vector.extend(0f32));

                    let projectile_translation = firefly_transform.translation
                        + (aim_vector * FIREFLY_SIZE).extend(firefly_transform.translation.z);
                    let transform = Transform {
                        translation: projectile_translation,
                        rotation: rotate_to_target,
                        scale: FIREFLY_BULLET_SCALE,
                    };
                    spawn_projectile(
                        &mut commands,
                        ProjectileType::FireflyBullet,
                        velocity,
                        projectile_assets.projectile.clone(),
                        transform,
                    );
                }
            }
        }
    }
}
