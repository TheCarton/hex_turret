use std::{cmp::max, collections::HashMap};

use bevy::{prelude::*, utils::dbg};
use itertools::Itertools;
mod colors;
const HEX_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 500.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, spawn_map, spawn_player))
        .add_systems(FixedUpdate, (move_player, render_hexes))
        .run()
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(),));
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
struct HexMap {
    size: i8,
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

#[derive(Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy)]
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

fn render_hexes(
    player_query: Query<&Player, Changed<Player>>,
    mut hex_query: Query<(&HexPosition, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
) {
    let player = player_query.single();
    for (hex_pos, mut image_handle) in hex_query.iter_mut() {
        if hex_pos == &player.hex {
            *image_handle = asset_server.load("red_hex.png");
        } else {
            *image_handle = asset_server.load("blue_hex.png");
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

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let size = 4;
    let physical_map_size = f32::from(size) * HEX_SIZE;
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
            HexMap { size },
        ))
        .with_children(|builder| {
            (-size..size)
                .cartesian_product(-size..size)
                .filter_map(|(q, r)| {
                    let s = -q - r;
                    if q + r + s == 0 {
                        Some(HexPosition::from_qr(q, r))
                    } else {
                        None
                    }
                })
                .for_each({
                    |hex| {
                        let pos = hex.pixel_coords();
                        builder.spawn((
                            SpriteBundle {
                                texture: asset_server.load("blue_hex.png"),
                                transform: Transform::from_xyz(pos.x, pos.y, 1.0),
                                ..default()
                            },
                            hex,
                        ));
                    }
                });
        });
}
