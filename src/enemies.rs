use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;

use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::game::AppState;
use crate::turrets::BuildTimer;
use crate::turrets::FactorySpawnConfig;
use crate::turrets::FireflyFactory;
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
                move_enemy,
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

#[derive(Bundle, Default)]
pub(crate) struct FireflyBundle {
    pub(crate) firefly: Firefly,
    pub(crate) animation_state: CurrentFireflyAnimationState,
    pub(crate) prev_animation_state: PrevFireflyAnimationState,
    pub(crate) hit: Hit,
    pub(crate) damaged_time: DamagedTime,
    pub(crate) enemy: Enemy,
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
pub(crate) struct Enemy;

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

fn despawn_dead_enemies(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Health, &Hit), With<Enemy>>,
) {
    for (enemy_entity, health, hit) in &q_enemies {
        if health.hp <= 0f32 || hit.has_hit {
            commands.entity(enemy_entity).despawn();
        }
    }
}

fn detect_enemy_player_collision(
    mut q_enemies: Query<(&Transform, &mut Hit), (With<Enemy>, Without<Player>)>,
    q_player: Query<&Transform, (With<Player>, Without<Enemy>)>,
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
