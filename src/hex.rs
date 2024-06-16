use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        config::{ConfigureLoadingState, LoadingStateConfig},
        LoadingStateAppExt,
    },
};
use derive_more::{Add, Sub};
use std::{
    cmp::Ordering,
    collections::HashMap,
    ops::{Add, AddAssign, Index, IndexMut, Mul, Sub},
};

use crate::{
    colors,
    constants::{E, HEX_DIRECTIONS, HEX_SIZE, MAX_CONTROL_VALUE, NE, NW, SE, SW, W},
    game::{AppState, EnterGameSet, FixedUpdateInGameSet, UpdateInGameSet},
    turrets::{
        ControlRay, ControlVec, EnergySource, EnergySourceAssets, EnergySourceBundle, ReloadTimer,
    },
};

pub struct HexPlugin;

const FIXED_UPDATE_INTERVAL: f64 = 1f64;

impl Plugin for HexPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::AssetLoading).load_collection::<HexAssets>(),
        );
        app.add_systems(
            OnEnter(AppState::InGame),
            (
                spawn_map,
                apply_deferred,
                populate_map,
                spawn_energy_sources,
            )
                .chain()
                .in_set(EnterGameSet),
        );
        app.add_systems(
            Update,
            (
                update_hexes,
                update_hex_control_from_control_rays,
                change_hex_color,
            )
                .in_set(UpdateInGameSet),
        );
        app.insert_resource(Time::<Fixed>::from_seconds(FIXED_UPDATE_INTERVAL));
        app.insert_resource(DecayTimer {
            timer: Timer::from_seconds(0.1f32, TimerMode::Repeating),
        });
        app.add_systems(
            FixedUpdate,
            (diffuse_hex_control, decay_hex_control).in_set(FixedUpdateInGameSet),
        );
    }
}

#[derive(Resource)]
struct DecayTimer {
    timer: Timer,
}

#[derive(Component, Default, Debug)]
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

const ENERGY_SOURCE_FLOW_RATE: f32 = 100f32;
const ENERGY_SOURCE_RELOAD_SECONDS: f32 = 5f32;

fn spawn_energy_sources(
    mut commands: Commands,
    energy_source_texture_atlas: Res<EnergySourceAssets>,
) {
    let starting_pos = vec![HexPosition { q: -3, r: 2 }, HexPosition { q: 2, r: -3 }];
    for pos in starting_pos {
        let mut transform = Transform::from_xyz(pos.pixel_coords().x, pos.pixel_coords().y, 2f32);
        transform.scale = Vec3::new(0.25f32, 0.25f32, 0.25f32);
        commands.spawn(EnergySourceBundle {
            energy_source: EnergySource {
                flow_rate: ENERGY_SOURCE_FLOW_RATE,
            },
            reload_timer: ReloadTimer {
                timer: Timer::from_seconds(ENERGY_SOURCE_RELOAD_SECONDS, TimerMode::Repeating),
            },
            hex_pos: pos,
            spritebundle: SpriteBundle {
                texture: energy_source_texture_atlas.energy_source.clone(),
                transform,
                ..default()
            },
        });
    }
}

pub(crate) fn spawn_map(mut commands: Commands, hex_texture_atlas: Res<HexAssets>) {
    let size = 4;
    let physical_map_size = f32::from(size) * HEX_SIZE;
    let map = HashMap::new();
    let hex_positions: Vec<HexPosition> = (-size..size).fold(Vec::new(), |mut acc, q| {
        (-size..size).for_each(|r| acc.push(HexPosition::from_qr(q, r)));
        acc
    });
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: colors::BOARD,
                custom_size: Some(Vec2::new(physical_map_size, physical_map_size)),
                ..default()
            },
            ..default()
        },
        HexMap { map },
    ));
    hex_positions.iter().for_each({
        |hex_pos| {
            commands.spawn(HexBundle {
                pos: *hex_pos,
                status: HexFaction::Neutral,
                sprite: SpriteBundle {
                    texture: hex_texture_atlas.hex.clone(),
                    transform: Transform::from_xyz(
                        hex_pos.pixel_coords().x,
                        hex_pos.pixel_coords().y,
                        0.0,
                    ),
                    ..default()
                },
                ..default()
            });
        }
    });
}

const MIN_CONTROL: f32 = 0.1f32;

fn decay_hex_control(
    mut hex_query: Query<&mut HexControl, With<Hex>>,
    mut decay_timer: ResMut<DecayTimer>,
    time: Res<Time>,
) {
    decay_timer.timer.tick(time.delta());
    if decay_timer.timer.finished() {
        for mut hex_control in hex_query.iter_mut() {
            for hex_faction in HexFaction::into_iter() {
                let new_control = lerp(hex_control[hex_faction], 0f32, 0.25f32);
                hex_control[hex_faction] = if new_control > MIN_CONTROL {
                    new_control
                } else {
                    0f32
                };
            }
        }
    }
}

// life just streaks by you while you mumble moronic catchphrases.

// I wish I could tell them that there was no God, but they never believed in one to begin with.
// I have to reaquaint them with the entire illusion of modernity just to disillusion them.

const DIFFUSION_EFFICIENCY: f32 = 0.01f32;
fn diffuse_hex_control(mut q_hexes: Query<&mut HexControl, With<Hex>>, q_hex_map: Query<&HexMap>) {
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
            let [mut hex_control, mut adj_control] = q_hexes.many_mut(hex_entities);
            let prev_control = hex_control.clone();
            for status_color in HexFaction::into_iter() {
                if adj_control[status_color] < hex_control[status_color] {
                    let fraction_change = (prev_control[status_color] - adj_control[status_color])
                        / prev_control[status_color];
                    let max_share = 1f32 / (num_neighbors * 2f32);
                    let delta = prev_control[status_color] * max_share * fraction_change;
                    adj_control[status_color] += delta * DIFFUSION_EFFICIENCY;
                    hex_control[status_color] -= delta;
                }
            }
        }
    }
}

pub(crate) fn populate_map(
    //wtf
    mut q_parent: Query<&mut HexMap>,
    q_child: Query<(Entity, &HexPosition), With<Hex>>,
) {
    let mut hexmap = q_parent.single_mut();

    for (entity, &hex_pos) in q_child.iter() {
        assert!(hexmap.map.insert(hex_pos, entity).is_none());
    }
}

pub(crate) fn update_hexes(
    mut hex_query: Query<(&HexControl, &mut HexFaction, &mut HexStructure), With<Hex>>,
    q_structure: Query<Entity>,
) {
    for (control, mut hex_status, mut structure) in hex_query.iter_mut() {
        *hex_status = control.max_status();
        if let Some(h_structure) = structure.entity {
            if q_structure.get(h_structure).is_err() {
                structure.entity = None;
            }
        }
    }
}

fn update_hex_control_from_control_rays(
    q_hex_map: Query<&HexMap>,
    mut hex_query: Query<&mut HexControl, (With<Hex>, Without<ControlRay>)>,
    q_control_ray: Query<&ControlVec, (With<ControlRay>, Without<Hex>)>,
) {
    let hex_map = q_hex_map.single();
    for control_vec in q_control_ray.iter() {
        for h in &control_vec.hexes {
            if let Some(hex_entity) = hex_map.map.get(&h) {
                let mut hc = hex_query.get_mut(*hex_entity).expect("valid hex entity");
                *hc = *hc + control_vec.control;
            }
        }
    }
}

fn change_hex_color(mut hex_query: Query<(&HexControl, &mut Sprite), With<Hex>>) {
    for (control, mut sprite) in hex_query.iter_mut() {
        let base = control.red + control.blue + control.neutral;
        if base > 0f32 {
            sprite.color.set_r(control.red / base);
            sprite.color.set_b(control.blue / base);
            sprite.color.set_g(control.neutral / base);
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct Hex;

#[derive(Component, Default, Copy, Clone)]
pub(crate) struct HexStructure {
    pub(crate) entity: Option<Entity>,
}

impl HexStructure {
    pub(crate) fn from_id(id: Entity) -> HexStructure {
        HexStructure { entity: Some(id) }
    }
}

#[derive(AssetCollection, Resource)]
pub(crate) struct HexAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 1, rows = 1))]
    #[asset(path = "blue_hex.png")]
    pub(crate) hex: Handle<Image>,
}

#[derive(Bundle, Default)]
pub(crate) struct HexBundle {
    pub(crate) hex: Hex,
    pub(crate) structure: HexStructure,
    pub(crate) pos: HexPosition,
    pub(crate) status: HexFaction,
    pub(crate) sprite: SpriteBundle,
    pub(crate) control: HexControl,
}

#[derive(Component, Eq, PartialEq, Default, Clone, Copy, Debug)]
pub(crate) enum HexFaction {
    Friendly,
    #[default]
    Neutral,
    Hostile,
}

impl HexFaction {
    fn into_iter() -> std::array::IntoIter<HexFaction, 3> {
        [
            HexFaction::Hostile,
            HexFaction::Friendly,
            HexFaction::Neutral,
        ]
        .into_iter()
    }
}

pub(crate) const MIN_HEX_CONTROL: HexControl = HexControl {
    red: 0f32,
    blue: 0f32,
    neutral: 0f32,
};

#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct HexControl {
    pub(crate) red: f32,
    pub(crate) blue: f32,
    pub(crate) neutral: f32,
}

impl HexControl {
    fn into_iter(&self) -> std::array::IntoIter<f32, 3> {
        [self.red, self.blue, self.neutral].into_iter()
    }
    #[allow(dead_code)]
    fn into_iter_mut(&mut self) -> std::array::IntoIter<&mut f32, 3> {
        [&mut self.red, &mut self.blue, &mut self.neutral].into_iter()
    }
}

impl Sub<f32> for HexControl {
    type Output = HexControl;

    fn sub(self, rhs: f32) -> Self::Output {
        HexControl {
            red: if (self.red - rhs) > MIN_HEX_CONTROL.red {
                self.red - rhs
            } else {
                MIN_HEX_CONTROL.red
            },
            blue: if (self.blue - rhs) > MIN_HEX_CONTROL.blue {
                self.blue - rhs
            } else {
                MIN_HEX_CONTROL.blue
            },
            neutral: if (self.neutral - rhs) > MIN_HEX_CONTROL.neutral {
                self.neutral - rhs
            } else {
                MIN_HEX_CONTROL.neutral
            },
        }
    }
}

impl AddAssign<HexControl> for HexControl {
    fn add_assign(&mut self, rhs: HexControl) {
        self.red += rhs.red;
        self.blue += rhs.blue;
        self.neutral += rhs.neutral;
    }
}

impl Default for HexControl {
    fn default() -> Self {
        HexControl {
            red: 0f32,
            blue: 0f32,
            neutral: 0f32,
        }
    }
}

impl Add for HexControl {
    type Output = HexControl;

    fn add(self, rhs: Self) -> Self::Output {
        HexControl {
            red: (self.red + rhs.red).min(MAX_CONTROL_VALUE),
            blue: (self.blue + rhs.blue).min(MAX_CONTROL_VALUE),
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
        self.into_iter().zip(other.into_iter()).all(|(x, y)| x == y)
    }
}

impl Ord for HexControl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("No NaNs in HexControl")
    }
}

impl Eq for HexControl {} // weird.

impl Index<HexFaction> for HexControl {
    type Output = f32;

    fn index(&self, status: HexFaction) -> &Self::Output {
        match status {
            HexFaction::Hostile => &self.red,
            HexFaction::Friendly => &self.blue,
            HexFaction::Neutral => &self.neutral,
        }
    }
}

impl IndexMut<HexFaction> for HexControl {
    fn index_mut(&mut self, status: HexFaction) -> &mut Self::Output {
        match status {
            HexFaction::Hostile => &mut self.red,
            HexFaction::Friendly => &mut self.blue,
            HexFaction::Neutral => &mut self.neutral,
        }
    }
}

impl HexControl {
    pub(crate) fn to_array(&self) -> [(HexFaction, f32); 3] {
        [
            (HexFaction::Hostile, self.red),
            (HexFaction::Friendly, self.blue),
            (HexFaction::Neutral, self.neutral),
        ]
    }

    pub(crate) fn max_status(&self) -> HexFaction {
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
    pub(crate) map: HashMap<HexPosition, Entity>,
}

impl HexMap {
    pub(crate) fn contains(&self, hex: HexPosition) -> bool {
        self.map.contains_key(&hex)
    }
}
#[derive(
    Component, PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy, Hash, Add, Sub, Default,
)]
pub(crate) struct HexPosition {
    pub(crate) q: i8,
    pub(crate) r: i8,
}

impl Mul<i8> for HexPosition {
    type Output = HexPosition;

    fn mul(self, rhs: i8) -> Self::Output {
        HexPosition {
            q: self.q * rhs,
            r: self.r * rhs,
        }
    }
}

impl HexPosition {
    pub(crate) fn s(&self) -> i8 {
        -self.q - self.r
    }
}
impl HexPosition {
    pub(crate) fn pixel_coords(&self) -> Vec2 {
        let x =
            HEX_SIZE * (3f32.sqrt() * f32::from(self.q) + (3f32.sqrt() / 2f32) * f32::from(self.r));
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

    pub(crate) fn from_vec3(vec3: Vec3) -> HexPosition {
        HexPosition {
            q: vec3.x as i8,
            r: vec3.y as i8,
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

    pub(crate) fn dist(&self, other: HexPosition) -> i8 {
        let diff = *self - other;
        (diff.q.abs() + (diff.r).abs() + (diff.s()).abs()) / 2
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

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start * (1f32 - t) + end * t
}

fn cube_lerp(a: HexPosition, b: HexPosition, t: f32) -> Vec3 {
    let x = lerp(f32::from(a.q), f32::from(b.q), t);
    let y = lerp(f32::from(a.r), f32::from(b.r), t);
    let z = lerp(f32::from(a.s()), f32::from(b.s()), t);
    Vec3::new(x, y, z)
}

#[allow(dead_code)]
pub(crate) fn cube_linedraw(a: HexPosition, b: HexPosition) -> Vec<HexPosition> {
    let n = a.dist(b);
    let mut line_vec = Vec::with_capacity(n as usize);
    for i in 0..=n {
        line_vec.push(HexPosition::from_vec3(cube_round(cube_lerp(
            a,
            b,
            1f32 / n as f32 * i as f32,
        ))))
    }

    line_vec
}

pub(crate) fn hex_direction(a: HexPosition, b: HexPosition) -> HexDirection {
    let n = a.dist(b) as f32;
    if n == 0f32 {
        return HexDirection::E;
    }
    let adj_pos = HexPosition::from_vec3(cube_round(cube_lerp(a, b, 1f32 / n)));
    let delta = adj_pos - a;
    let dir = match delta {
        NE => HexDirection::NE,
        E => HexDirection::E,
        SE => HexDirection::SE,
        SW => HexDirection::SW,
        W => HexDirection::W,
        NW => HexDirection::NW,
        _ => unreachable!(),
    };
    dir
}

#[test]
fn get_direction() {}

#[allow(dead_code)]
fn lerp_point(p0: Vec2, p1: Vec2, t: f32) -> Vec2 {
    Vec2 {
        x: lerp(p0.x, p1.x, t),
        y: lerp(p0.y, p1.y, t),
    }
}

#[cfg(test)]
#[test]
fn hex_position_to_pixel_math() {
    let (q, r) = (3, 2);
    let h = HexPosition::from_qr(q, r);
    let correct_x = HEX_SIZE * (3f32.sqrt() * q as f32 + (3f32.sqrt() / 2f32) * r as f32);
    let correct_y = HEX_SIZE * ((3f32 / 2f32) * r as f32);

    assert_eq!(h.pixel_coords().x, correct_x);
    assert_eq!(h.pixel_coords().y, correct_y);
}
