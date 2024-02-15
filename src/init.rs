use crate::entities::*;
use crate::*;
use bevy::prelude::*;

pub(crate) fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PlayerBundle {
        player: Player,
        pos: HexPosition::default(),
        sprite: SpriteBundle {
            texture: asset_server.load("triangle.png"),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),

            ..default()
        },
    });
}

pub(crate) fn setup_enemy_spawning(mut commands: Commands) {
    commands.insert_resource(EnemySpawnConfig {
        timer: Timer::from_seconds(5f32, TimerMode::Repeating),
    })
}

pub(crate) fn setup_factory_spawning(mut commands: Commands) {
    commands.insert_resource(FactorySpawnConfig {
        timer: Timer::from_seconds(3f32, TimerMode::Repeating),
    })
}
pub(crate) fn populate_map(
    mut q_parent: Query<&mut HexMap>,
    q_child: Query<(Entity, &HexPosition)>,
) {
    let mut hexmap = q_parent.single_mut();

    for (entity, &hex_pos) in q_child.iter() {
        hexmap.map.insert(hex_pos, entity);
    }
}

pub(crate) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub(crate) fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let size = 4;
    let physical_map_size = f32::from(size) * HEX_SIZE;
    let map = HashMap::new();
    let hex_positions: Vec<HexPosition> = (-size..size)
        .cartesian_product(-size..size)
        .filter_map(|(q, r)| {
            let s = -q - r;
            if q + r + s == 0 {
                Some(HexPosition::from_qr(q, r))
            } else {
                None
            }
        })
        .collect();
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: colors::BOARD,
                    custom_size: Some(Vec2::new(physical_map_size, physical_map_size)),
                    ..default()
                },
                ..default()
            },
            HexMap { size, map },
        ))
        .with_children(|builder| {
            hex_positions.iter().for_each({
                |hex_pos| {
                    builder.spawn(HexBundle {
                        pos: *hex_pos,
                        status: HexStatus::Unoccupied,
                        sprite: SpriteBundle {
                            texture: asset_server.load("blue_hex.png"),
                            transform: Transform::from_xyz(
                                hex_pos.pixel_coords().x,
                                hex_pos.pixel_coords().y,
                                1.0,
                            ),
                            ..default()
                        },
                    });
                }
            });
        });
}
