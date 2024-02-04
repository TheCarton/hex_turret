use bevy::{prelude::*, utils::dbg, window::PrimaryWindow};
use derive_more::Add;
use std::collections::HashMap;

mod constants;
use constants::{HEX_DIRECTIONS, HEX_SIZE, PLAYER_SPEED};
use itertools::Itertools;
mod colors;

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("triangle.png"),
            transform: Transform::from_xyz(0.0, 0.0, 2.0),

            ..default()
        },
        Player,
        HexPosition::from_pixel(Vec2::ZERO),
    ));
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Enemy;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct CursorWorldCoords {
    pos: Vec2,
}

#[derive(Resource, Default)]
struct CursorHexPosition {
    hex: HexPosition,
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

#[derive(Component)]
struct HexMap {
    size: i8,
    map: HashMap<HexPosition, Entity>,
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

#[derive(Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Add, Default)]
struct HexPosition {
    q: i8,
    r: i8,
}

impl HexPosition {
    fn s(&self) -> i8 {
        -self.q - self.r
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
enum HexStatus {
    Occupied,
    Unoccupied,
    Selected,
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

fn cube_round(frac: Vec3) -> Vec3 {
    let mut q = frac.x.round();
    let mut r = frac.y.round();
    let mut s = frac.z.round();

    let q_diff = (q - frac.x).abs();
    let r_diff = (r - frac.y).abs();
    let s_diff = (s - frac.z).abs();

    if q_diff > r_diff && q_diff > s_diff {
        q = -r - s;
    } else if r_diff > s_diff {
        r = -q - s;
    } else {
        s = -q - r;
    }
    Vec3::new(q, r, s)
}

impl HexPosition {
    fn pixel_coords(&self) -> Vec2 {
        let x =
            HEX_SIZE * (3f32.sqrt() * f32::from(self.q) + 3f32.sqrt() / 2f32 * f32::from(self.r));
        let y = HEX_SIZE * (3f32 / 2f32 * f32::from(self.r));
        Vec2::new(x, y)
    }

    fn from_qr(q: i8, r: i8) -> HexPosition {
        HexPosition { q, r }
    }

    fn from_pixel(pixel_pos: Vec2) -> HexPosition {
        let q = (3f32.sqrt() / 3f32 * pixel_pos.x - 1f32 / 3f32 * pixel_pos.y) / HEX_SIZE;
        let r = (2f32 / 3f32 * pixel_pos.y) / HEX_SIZE;
        let s = -q - r;
        let rounded = cube_round(Vec3::new(q, r, s));
        HexPosition {
            q: rounded.x as i8,
            r: rounded.y as i8,
        }
    }
}

#[derive(Bundle)]
struct HexBundle {
    pos: HexPosition,
    status: HexStatus,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
struct PlayerBundle {
    pos: HexPosition,
    status: HexStatus,
    sprite: SpriteBundle,
}
fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
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

fn populate_map(mut q_parent: Query<&mut HexMap>, q_child: Query<(Entity, &HexPosition)>) {
    let mut hexmap = q_parent.single_mut();

    for (entity, &hex_pos) in q_child.iter() {
        hexmap.map.insert(hex_pos, entity);
    }
}
