use bevy::prelude::*;
use derive_more::Add;
use itertools::Itertools;
use std::collections::HashMap;

use rand::Rng;

use crate::{colors, constants::HEX_SIZE};
pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_map, apply_deferred, populate_map).chain());
        app.add_systems(Update, (update_hexes, render_hexes));
    }
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
                        status: HexStatus::Neutral,
                        sprite: SpriteBundle {
                            texture: asset_server.load("blue_hex.png"),
                            transform: Transform::from_xyz(
                                hex_pos.pixel_coords().x,
                                hex_pos.pixel_coords().y,
                                1.0,
                            ),
                            ..default()
                        },
                        ..default()
                    });
                }
            });
        });
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

fn update_hexes(mut hex_query: Query<(&HexControl, &mut HexStatus)>) {
    //TODO: Switch to hashmap for updating hex status. Have hax status be based on control floats: red / blue.
    // component: hex control component. has a struct of hex position and an effect on control, and a time of effect.
    for (control, mut hex_status) in hex_query.iter_mut() {
        let (faction, _) = control
            .into_iter()
            .max_by(|(_, x), (_, y)| x.total_cmp(y))
            .expect("not empty");

        *hex_status = faction;
    }
}

fn render_hexes(
    mut hex_query: Query<(&HexStatus, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
) {
    for (hex_status, mut image_handle) in hex_query.iter_mut() {
        match hex_status {
            HexStatus::Blue => *image_handle = asset_server.load("blue_hex.png"),
            HexStatus::Neutral => *image_handle = asset_server.load("orange_hex.png"),
            HexStatus::Red => *image_handle = asset_server.load("red_hex.png"),
        }
    }
}
pub(crate) fn random_hex(hex_map: &HexMap) -> HexPosition {
    let mut rng = rand::thread_rng();
    let q = rng.gen_range(-hex_map.size..hex_map.size);
    let r = rng.gen_range(-hex_map.size..hex_map.size);
    let h = HexPosition::from_qr(q, r);
    assert!(hex_map.contains(h));
    h
}

#[derive(Component, Default)]
pub(crate) struct Hex;

#[derive(Bundle, Default)]
pub(crate) struct HexBundle {
    pub(crate) hex: Hex,
    pub(crate) pos: HexPosition,
    pub(crate) status: HexStatus,
    pub(crate) sprite: SpriteBundle,
    pub(crate) control: HexControl,
}

#[derive(Component, Eq, PartialEq, Default)]
pub(crate) enum HexStatus {
    Blue,
    #[default]
    Neutral,
    Red,
}

#[derive(Component, Debug, Copy, Clone, Add)]
pub(crate) struct HexControl {
    pub(crate) red: f32,
    pub(crate) blue: f32,
    pub(crate) neutral: f32,
}

impl Default for HexControl {
    fn default() -> Self {
        HexControl {
            red: 0f32,
            blue: 0f32,
            neutral: 100f32,
        }
    }
}

impl IntoIterator for HexControl {
    type Item = (HexStatus, f32);

    type IntoIter = std::array::IntoIter<Self::Item, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (HexStatus::Red, self.red),
            (HexStatus::Blue, self.blue),
            (HexStatus::Neutral, self.neutral),
        ]
        .into_iter()
    }
}

#[derive(Component)]
pub(crate) struct HexMap {
    pub(crate) size: i8,
    pub(crate) map: HashMap<HexPosition, Entity>,
}

impl HexMap {
    pub(crate) fn contains(&self, hex: HexPosition) -> bool {
        self.map.contains_key(&hex)
    }
}
#[derive(Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Add, Default)]
pub(crate) struct HexPosition {
    pub(crate) q: i8,
    pub(crate) r: i8,
}

impl HexPosition {
    pub(crate) fn s(&self) -> i8 {
        -self.q - self.r
    }
}
impl HexPosition {
    pub(crate) fn pixel_coords(&self) -> Vec2 {
        let x =
            HEX_SIZE * (3f32.sqrt() * f32::from(self.q) + 3f32.sqrt() / 2f32 * f32::from(self.r));
        let y = HEX_SIZE * (3f32 / 2f32 * f32::from(self.r));
        Vec2::new(x, y)
    }

    pub(crate) fn from_qr(q: i8, r: i8) -> HexPosition {
        HexPosition { q, r }
    }

    pub(crate) fn from_pixel(pixel_pos: Vec2) -> HexPosition {
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
