use std::ops::Add;

use bevy::prelude::*;

use crate::{
    constants::PLAYER_SPEED,
    game::{EnterGameSet, FixedUpdateInGameSet, UpdateInGameSet},
    hex::{Hex, HexControl, HexMap, HexPosition},
    turrets::Faction,
};

pub(crate) struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_player, spawn_player_hex_control, apply_deferred)
                .chain()
                .in_set(EnterGameSet),
        );
        app.add_systems(Update, move_player.in_set(UpdateInGameSet));
        app.add_systems(FixedUpdate, player_control_hex.in_set(FixedUpdateInGameSet));
    }
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    pub(crate) player: Player,
    pub(crate) faction: Faction,
    pub(crate) pos: HexPosition,
    pub(crate) sprite: SpriteBundle,
    pub(crate) hex_control: HexControl,
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(PlayerBundle {
        player: Player,
        faction: Faction::Friendly,
        pos: HexPosition::default(),
        sprite: SpriteBundle {
            texture: asset_server.load("triangle.png"),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),

            ..default()
        },
        hex_control: HexControl {
            red: 100f32,
            blue: 0f32,
            neutral: 0f32,
        },
    });
}

fn player_control_hex(
    q_player_hex_control: Query<(&HexControl, &HexPosition), (With<Player>, Without<Hex>)>,
    mut q_player_hex: Query<&mut HexControl, (With<Hex>, Without<Player>)>,
    q_hex_map: Query<&HexMap>,
) {
    // panics no entities
    let (player_control, player_pos) = q_player_hex_control.single();
    let hex_map = q_hex_map.single();
    let hex_id = hex_map.map.get(player_pos);
    if let Some(h) = hex_id {
        if let Ok(mut hex_control) = q_player_hex.get_mut(*h) {
            *hex_control = hex_control.add(*player_control);
        }
    }
}

fn move_player(
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

pub(crate) fn spawn_player_hex_control(mut commands: Commands) {
    commands.spawn(HexControl {
        red: 0f32,
        blue: 500f32,
        neutral: 0f32,
    });
}
