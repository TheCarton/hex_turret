use bevy::{ecs::system::SystemState, prelude::*};
use derive_more::Add;
use itertools::Itertools;
use std::{
    cmp::{max, min, Ordering},
    collections::HashMap,
    ops::{Index, IndexMut},
    slice::IterMut,
};

use rand::Rng;

use crate::{
    colors,
    constants::{CONTROL_DECAY, E, HEX_DIRECTIONS, HEX_SIZE, NE, NW, SE, SW, W},
};

pub struct HexPlugin;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_map, apply_deferred, populate_map).chain());
        app.add_systems(Update, (update_hexes, change_hex_color));
        app.add_systems(FixedUpdate, (diffuse_hex_control, decay_hex_control));
    }
}

#[derive(Component, Default)]
pub(crate) enum HexDirection {
    NE,
    #[default]
    E,
    SE,
    SW,
    W,
    NW,
}

impl HexDirection {
    pub(crate) fn to_hex(&self) -> HexPosition {
        match self {
            HexDirection::NE => NE,
            HexDirection::E => E,
            HexDirection::SE => SE,
            HexDirection::SW => SW,
            HexDirection::W => W,
            HexDirection::NW => NW,
        }
    }
}

pub(crate) fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let size = 4;
    let physical_map_size = f32::from(size) * HEX_SIZE;
    let map = HashMap::new();
    let hex_positions: Vec<HexPosition> = (-size..size).fold(Vec::new(), |mut acc, q| {
        (-size..size).for_each(|r| acc.push(HexPosition::from_qr(q, r)));
        acc
    });
    dbg!(&hex_positions);
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

fn decay_hex_control(mut hex_query: Query<&mut HexControl, With<Hex>>) {
    for mut hex_control in hex_query.iter_mut() {
        hex_control.red = (hex_control.red - CONTROL_DECAY).max(0f32);
        hex_control.blue = (hex_control.blue - CONTROL_DECAY).max(0f32);
    }
}

// life just streaks by you while you mumble moronic catchphrases.

// I wish I could tell them that there was no God, but they never believed in one to begin with.
// I have to reaquaint them with the entire illusion of modernity just to disillusion them.

fn diffuse_hex_control(
    mut q_hexes: Query<(&HexPosition, &mut HexControl, With<Hex>)>,
    q_hex_map: Query<&HexMap>,
) {
    let hex_map = q_hex_map.single();
    for (pos, entity) in hex_map.map.iter() {
        let neighbor_entities: Vec<&Entity> = pos
            .neighbors()
            .iter()
            .filter_map(|n| hex_map.map.get(n))
            .collect();
        let num_neighbors = neighbor_entities.len() as f32;
        for adj_entity in neighbor_entities {
            let hex_entities: [Entity; 2] = [*entity, *adj_entity];
            let [(_, mut hex_control, _), (_, mut adj_control, _)] = q_hexes.many_mut(hex_entities);
            let prev_control = hex_control.clone();
            for status_color in [HexStatus::Red, HexStatus::Blue] {
                if adj_control[status_color] < hex_control[status_color] {
                    let fraction_change = (prev_control[status_color] - adj_control[status_color])
                        / prev_control[status_color];
                    let max_share = 1f32 / (num_neighbors * 2f32);
                    let delta = prev_control[status_color] * max_share * fraction_change;
                    adj_control[status_color] += delta;
                    hex_control[status_color] -= delta;
                }
            }
        }
    }
}

pub(crate) fn populate_map(
    mut q_parent: Query<&mut HexMap>,
    q_child: Query<(Entity, &HexPosition, With<Hex>)>,
) {
    let mut hexmap = q_parent.single_mut();

    for (entity, &hex_pos, _) in q_child.iter() {
        dbg!(entity);
        dbg!(hex_pos);
        assert!(hexmap.map.insert(hex_pos, entity).is_none());
    }
}

fn update_hexes(mut hex_query: Query<(&HexControl, &mut HexStatus, With<Hex>)>) {
    for (control, mut hex_status, _) in hex_query.iter_mut() {
        *hex_status = control.max_status();
    }
}

fn change_hex_color(mut hex_query: Query<(&HexControl, &mut Sprite, With<Hex>)>) {
    for (control, mut sprite, _) in hex_query.iter_mut() {
        let base = control.red + control.blue + control.neutral;
        sprite.color.set_r(control.red / base);
        sprite.color.set_b(control.blue / base);
        sprite.color.set_g(control.neutral / base);
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

#[derive(Component, Eq, PartialEq, Default, Clone, Copy, Debug)]
pub(crate) enum HexStatus {
    Blue,
    #[default]
    Neutral,
    Red,
}

impl HexStatus {
    fn into_iter() -> std::array::IntoIter<HexStatus, 3> {
        [HexStatus::Red, HexStatus::Blue, HexStatus::Neutral].into_iter()
    }
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

impl PartialOrd for HexControl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.red + self.blue + self.neutral)
            .partial_cmp(&(&other.red + &other.blue + &other.neutral))
    }
}

impl PartialEq for HexControl {
    fn eq(&self, other: &Self) -> bool {
        (self.red + self.blue + self.neutral) == (other.red + other.blue + other.neutral)
    }
}

impl Ord for HexControl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("No NaNs in HexControl")
    }
}

impl Eq for HexControl {} // weird.

impl Index<HexStatus> for HexControl {
    type Output = f32;

    fn index(&self, status: HexStatus) -> &Self::Output {
        match status {
            HexStatus::Red => &self.red,
            HexStatus::Blue => &self.blue,
            HexStatus::Neutral => &self.neutral,
        }
    }
}

impl IndexMut<HexStatus> for HexControl {
    fn index_mut(&mut self, status: HexStatus) -> &mut Self::Output {
        match status {
            HexStatus::Red => &mut self.red,
            HexStatus::Blue => &mut self.blue,
            HexStatus::Neutral => &mut self.neutral,
        }
    }
}

impl HexControl {
    fn len() -> usize {
        3
    }

    fn to_array(&self) -> [(HexStatus, f32); 3] {
        [
            (HexStatus::Red, self.red),
            (HexStatus::Blue, self.blue),
            (HexStatus::Neutral, self.neutral),
        ]
    }

    fn max_status(&self) -> HexStatus {
        let (status, _val) = self
            .to_array()
            .into_iter()
            .max_by(|(_, x), (_, y)| x.total_cmp(y))
            .expect("control values not empty");
        status
    }
}

#[derive(Component)]
pub(crate) struct HexMap {
    //TODO: Better data structure for this. I'm iterating through these keys.
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
    pub(crate) fn neighbors(&self) -> [HexPosition; 6] {
        let mut neighbors = [*self; 6];
        for (n, d) in neighbors.iter_mut().zip(HEX_DIRECTIONS) {
            *n = *n + d;
        }
        assert!(neighbors.iter().all(|n| n != self));
        neighbors
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
