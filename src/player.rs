use bevy::prelude::*;

use crate::{
    constants::PLAYER_SPEED,
    game::{EnterGameSet, UpdateInGameSet},
    hex::{spawn_map, HexFaction, HexPosition},
};

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_player, apply_deferred)
                .chain()
                .after(spawn_map)
                .in_set(EnterGameSet),
        );
        app.add_systems(Update, move_player.in_set(UpdateInGameSet));
    }
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    pub(crate) player: Player,
    pub(crate) faction: HexFaction,
    pub(crate) pos: HexPosition,
    pub(crate) sprite: SpriteBundle,
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PlayerBundle {
        player: Player,
        faction: HexFaction::Friendly,
        pos: HexPosition::default(),
        sprite: SpriteBundle {
            texture: asset_server.load("triangle.png"),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),

            ..default()
        },
    });
}

pub(crate) fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_hex_query: Query<&mut HexPosition, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_transform_query.single_mut();
    let direction = match keyboard_input.get_pressed().last() {
        Some(KeyCode::ArrowLeft) | Some(KeyCode::KeyA) => Vec3::new(-1.0, 0.0, 0.0),
        Some(KeyCode::ArrowRight) | Some(KeyCode::KeyD) => Vec3::new(1.0, 0.0, 0.0),
        Some(KeyCode::ArrowUp) | Some(KeyCode::KeyW) => Vec3::new(0.0, 1.0, 0.0),
        Some(KeyCode::ArrowDown) | Some(KeyCode::KeyS) => Vec3::new(0.0, -1.0, 0.0),
        _ => Vec3::ZERO,
    };

    let new_player_pos =
        player_transform.translation + direction * PLAYER_SPEED * time.delta_seconds();

    let new_hex = HexPosition::from_pixel(Vec2::new(new_player_pos.x, new_player_pos.y));
    let mut player_hex = player_hex_query.single_mut();
    *player_hex = new_hex;
    player_transform.translation = new_player_pos;
}
