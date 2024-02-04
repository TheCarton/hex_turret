use bevy::{prelude::*, window::PrimaryWindow};
use std::collections::HashMap;

mod colors;
mod constants;
mod entities;
mod init;

use constants::{HEX_DIRECTIONS, HEX_SIZE, PLAYER_SPEED};
use entities::*;
use init::*;
use itertools::Itertools;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<CursorWorldCoords>()
        .init_resource::<CursorHexPosition>()
        .add_systems(
            Startup,
            (setup, spawn_map, spawn_player, apply_deferred, populate_map).chain(),
        )
        .add_systems(
            Update,
            (move_player, update_hexes, render_hexes, cursor_system),
        )
        .run()
}

fn cursor_system(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    mut cursor_hex: ResMut<CursorHexPosition>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        cursor_coords.pos = world_position;
        cursor_hex.hex = HexPosition::from_pixel(world_position);
    }
}

impl HexMap {
    fn contains(&self, hex: HexPosition) -> bool {
        let d = [hex.q, hex.r, hex.s()]
            .into_iter()
            .map(|v| v.abs())
            .max()
            .expect("hex has position.");
        d <= self.size
    }
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_hex_query: Query<&mut HexPosition, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_transform_query.single_mut();
    let direction = match keyboard_input.get_pressed().last() {
        Some(KeyCode::Left) => Vec3::new(-1.0, 0.0, 0.0),
        Some(KeyCode::Right) => Vec3::new(1.0, 0.0, 0.0),
        Some(KeyCode::Up) => Vec3::new(0.0, 1.0, 0.0),
        Some(KeyCode::Down) => Vec3::new(0.0, -1.0, 0.0),
        _ => Vec3::ZERO,
    };

    let new_player_pos =
        player_transform.translation + direction * PLAYER_SPEED * time.delta_seconds();

    let new_hex = HexPosition::from_pixel(Vec2::new(new_player_pos.x, new_player_pos.y));
    let mut player_hex = player_hex_query.single_mut();
    *player_hex = new_hex;
    player_transform.translation = new_player_pos;
}

fn update_hexes(
    player_hex_query: Query<&HexPosition, With<Player>>,
    mut hex_query: Query<(&HexPosition, &mut HexStatus)>,
    cursor_hex: Res<CursorHexPosition>,
) {
    let player_hex = player_hex_query.single();
    for (hex_pos, mut hex_status) in hex_query.iter_mut() {
        let is_player_hex = hex_pos == player_hex;
        let is_neighbor = HEX_DIRECTIONS
            .map(|delta| *hex_pos + delta)
            .iter()
            .any(|&n| n == *player_hex);
        let is_cursor = *hex_pos == cursor_hex.hex;
        match (is_player_hex, is_neighbor, is_cursor) {
            (true, _, _) => *hex_status = HexStatus::Occupied,
            (false, true, _) => *hex_status = HexStatus::Selected,
            (_, _, true) => *hex_status = HexStatus::Selected,
            _ => *hex_status = HexStatus::Unoccupied,
        }
    }
}

fn render_hexes(
    mut hex_query: Query<(&HexStatus, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
) {
    for (hex_status, mut image_handle) in hex_query.iter_mut() {
        match hex_status {
            HexStatus::Occupied => *image_handle = asset_server.load("red_hex.png"),
            HexStatus::Unoccupied => *image_handle = asset_server.load("blue_hex.png"),
            HexStatus::Selected => *image_handle = asset_server.load("orange_hex.png"),
        }
    }
}
