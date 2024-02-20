use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::animation::AnimationIndices;
use crate::animation::AnimationTimer;
use crate::{
    constants::{ENEMY_HEALTH, ENEMY_SIZE, ENEMY_SPEED, PLAYER_SIZE},
    hex::{random_hex, HexMap},
    player::Player,
};

pub(crate) struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_factory_spawning);
        app.add_systems(
            Update,
            (
                spawn_fireflies,
                move_enemy,
                despawn_dead_enemies,
                detect_enemy_player_collision,
                spawn_factories,
            ),
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
    pub(crate) sprite: SpriteSheetBundle,
    pub(crate) animation_indices: AnimationIndices,
    pub(crate) animation_timer: AnimationTimer,
}

#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub(crate) enum FireflyAnimationState {
    #[default]
    Normal,
    Damaged,
}

#[derive(Resource)]
pub(crate) struct FireflyTextureAtlas {
    pub(crate) atlas: Handle<TextureAtlas>,
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

impl FromWorld for FireflyTextureAtlas {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture_handle = asset_server.load("firefly_spritesheet.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(48f32, 48f32), 8, 3, None, None);
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        FireflyTextureAtlas {
            atlas: texture_atlas_handle,
        }
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

fn move_enemy(
    mut param_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
    time: Res<Time>,
) {
    let player_transform = param_set.p0().single().clone();
    for mut enemy_transform in param_set.p1().iter_mut() {
        dbg!(&enemy_transform);
        if let Some(n) =
            (player_transform.translation - enemy_transform.translation).try_normalize()
        {
            let mut v = n * ENEMY_SPEED * time.delta_seconds();
            v.z = 0f32;
            enemy_transform.translation += v;
        }
    }
}

fn despawn_dead_enemies(mut commands: Commands, q_enemies: Query<(Entity, &Health, With<Enemy>)>) {
    for (enemy_entity, health, _) in &q_enemies {
        if health.hp <= 0f32 {
            commands.entity(enemy_entity).despawn();
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

fn spawn_fireflies(
    mut commands: Commands,
    firefly_sprite_sheet: Res<FireflyTextureAtlas>,
    time: Res<Time>,
    mut q_factories: Query<(&Transform, &mut BuildTimer, With<FireflyFactory>)>,
) {
    for (factory, mut build_timer, _) in q_factories.iter_mut() {
        build_timer.timer.tick(time.delta());
        if build_timer.timer.finished() {
            let p = Vec3::new(factory.translation.x, factory.translation.y, 2f32);
            let mut anim_indices = AnimationIndices::firefly_indices();
            commands.spawn(FireflyBundle {
                sprite: SpriteSheetBundle {
                    sprite: TextureAtlasSprite::new(anim_indices.next_index()),
                    texture_atlas: firefly_sprite_sheet.atlas.clone(),
                    transform: Transform::from_translation(p),
                    ..default()
                },
                animation_indices: anim_indices,
                ..default()
            });
        }
    }
}

fn spawn_factories(
    mut commands: Commands,
    factory_sprite_sheet: Res<FireflyFactoryTextureAtlas>,
    time: Res<Time>,
    mut config: ResMut<FactorySpawnConfig>,
    q_hex_map: Query<&HexMap>,
) {
    let hex_map = q_hex_map.single();
    let random_hex = random_hex(hex_map);
    config.timer.tick(time.delta());
    if config.timer.finished() {
        let mut anim_indices = AnimationIndices::default();
        commands.spawn(FireflyFactoryBundle {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(anim_indices.next_index()),
                texture_atlas: factory_sprite_sheet.atlas.clone(),
                transform: Transform::from_translation(random_hex.pixel_coords().extend(1f32)),
                ..default()
            },
            animation_indices: anim_indices,
            ..default()
        });
    }
}
