use bevy::{prelude::*, window::PrimaryWindow};
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
        .init_resource::<WorldCoords>()
        .add_systems(
            Startup,
            (setup, spawn_map, spawn_player, apply_deferred, populate_map).chain(),
        )
        .add_systems(
            FixedUpdate,
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
        Player {
            hex: HexPosition::from_pixel(Vec2::ZERO),
        },
    ));
}

#[derive(Component)]
struct MainCamera;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct WorldCoords(Vec2);

fn cursor_system(
    mut mycoords: ResMut<WorldCoords>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_hex_map: Query<&HexMap>,
    mut q_hex_status: Query<&mut HexStatus>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    let hex_map = q_hex_map.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(entity) = window.cursor_position().and_then(|cursor| {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor) {
            let pixel_xy = ray.origin.truncate();
            let hex_pos = HexPosition::from_pixel(pixel_xy);
            hex_map.map.get(&hex_pos)
        } else {
            None
        }
    }) {
        if let Ok(mut hex_status) = q_hex_status.get_mut(*entity) {
            *hex_status = HexStatus::Selected;
        }
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

#[derive(Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Add)]
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
struct Player {
    hex: HexPosition,
}

#[derive(Component)]
enum HexStatus {
    Occupied,
    Unoccupied,
    Selected,
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_query: Query<&mut Player>,
    time: Res<Time>,
) {
    let mut player_transform = player_transform_query.single_mut();
    let mut player = player_query.single_mut();
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
    player.hex = new_hex;
    player_transform.translation = new_player_pos;
}

fn update_hexes(
    player_query: Query<&Player, Changed<Player>>,
    mut hex_query: Query<(&HexPosition, &mut HexStatus)>,
) {
    let player = player_query.single();
    for (hex_pos, mut hex_status) in hex_query.iter_mut() {
        let is_player_hex = hex_pos == &player.hex;
        let is_neighbor = HEX_DIRECTIONS
            .map(|delta| *hex_pos + delta)
            .iter()
            .any(|&n| n == player.hex);
        match (is_player_hex, is_neighbor) {
            (true, _) => *hex_status = HexStatus::Occupied,
            (false, true) => *hex_status = HexStatus::Selected,
            (false, false) => *hex_status = HexStatus::Unoccupied,
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
    let mut map = HashMap::new();
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
