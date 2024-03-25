use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;

use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::constants::FIREFLY_RANGE;
use crate::constants::PROJECTILE_SPEED;
use crate::game::AppState;
use crate::projectiles::FireflyProjectileAssets;
use crate::projectiles::FireflyProjectileBundle;
use crate::projectiles::Velocity;
use crate::turrets::BuildTimer;
use crate::turrets::Faction;
use crate::turrets::FactorySpawnConfig;
use crate::turrets::FireflyFactory;
use crate::turrets::ReloadTimer;
use crate::{
    constants::{ENEMY_HEALTH, ENEMY_SIZE, ENEMY_SPEED, PLAYER_SIZE},
    player::Player,
};

pub(crate) struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_factory_spawning);
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::AssetLoading).load_collection::<FireflyAssets>(),
        );
        app.add_systems(
            Update,
            (
                spawn_fireflies,
                firefly_targeting,
                fire_firefly_projectiles,
                move_seeking_units,
                despawn_dead_enemies,
                detect_enemy_player_collision,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

#[derive(Component, Default)]
pub(crate) struct Firefly;

pub(crate) fn setup_factory_spawning(mut commands: Commands) {
    commands.insert_resource(FactorySpawnConfig {
        timer: Timer::from_seconds(3f32, TimerMode::Repeating),
    })
}

#[derive(Component, Default, Debug)]
pub(crate) struct Target {
    entity: Option<Entity>,
}

#[derive(Bundle, Default)]
pub(crate) struct FireflyBundle {
    pub(crate) firefly: Firefly,
    pub(crate) seeking: Seeking,
    pub(crate) faction: Faction,
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

impl Default for Health {
    fn default() -> Self {
        Health { hp: ENEMY_HEALTH }
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
            let target_translation = param_set
                .p0()
                .get(target_entity)
                .expect("valid entity")
                .translation;

            if (unit_translation).distance(target_translation) > FIREFLY_RANGE {
                let n = (target_translation - unit_translation).normalize();
                let mut v = n * ENEMY_SPEED * time.delta_seconds();
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

fn despawn_dead_enemies(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Health, &Hit), With<Seeking>>,
) {
    for (enemy_entity, health, hit) in &q_enemies {
        if health.hp <= 0f32 || hit.has_hit {
            commands.entity(enemy_entity).despawn();
        }
    }
}

fn detect_enemy_player_collision(
    mut q_enemies: Query<(&Transform, &mut Hit), (With<Seeking>, Without<Player>)>,
    q_player: Query<&Transform, (With<Player>, Without<Seeking>)>,
) {
    let player = q_player.single();
    for (enemy, mut enemy_collision) in &mut q_enemies {
        let collision = Aabb2d::new(enemy.translation.truncate(), ENEMY_SIZE / 2f32).intersects(
            &Aabb2d::new(player.translation.truncate(), PLAYER_SIZE / 2f32),
        );
        enemy_collision.has_hit = collision;
    }
}

fn spawn_fireflies(
    mut commands: Commands,
    firefly_assets: Res<FireflyAssets>,
    time: Res<Time>,
    mut q_factories: Query<(&Transform, &mut BuildTimer), With<FireflyFactory>>,
) {
    for (factory, mut build_timer) in q_factories.iter_mut() {
        build_timer.timer.tick(time.delta());
        if build_timer.timer.finished() {
            let p = Vec3::new(factory.translation.x, factory.translation.y, 2f32);
            let mut anim_indices = AnimationIndices::firefly_indices();
            dbg!(&firefly_assets.layout);
            commands.spawn(FireflyBundle {
                faction: Faction::Hostile,
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
    q_firefly: Query<(Entity, &Transform, &Faction), With<Firefly>>,
    mut param_set: ParamSet<(Query<(Entity, &Transform, &Faction)>, Query<&mut Target>)>,
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
            dbg!("reload finish");
            dbg!(target);
            if let Some(target_entity) = target.entity {
                let target_transform = q_target.get(target_entity).expect("valid entity");
                if target_transform
                    .translation
                    .distance(firefly_transform.translation)
                    < FIREFLY_RANGE
                {
                    let aim_point = (target_transform.translation.truncate()
                        - firefly_transform.translation.truncate())
                    .normalize();
                    let velocity = Velocity::from(aim_point * PROJECTILE_SPEED);
                    let rotate_to_target = Quat::from_rotation_arc(Vec3::Y, aim_point.extend(0f32));
                    let transform = Transform {
                        translation: firefly_transform.translation,
                        rotation: rotate_to_target,
                        scale: Vec3::new(0.5f32, 0.5f32, 0.5f32),
                    };
                    commands.spawn(FireflyProjectileBundle {
                        velocity,
                        sprite: SpriteBundle {
                            texture: projectile_assets.projectile.clone(),
                            transform,
                            ..default()
                        },
                        ..default()
                    });
                }
            }
        }
    }
}
