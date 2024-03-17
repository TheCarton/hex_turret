use bevy::{
    core_pipeline::core_2d::graph::node::TONEMAPPING, prelude::*, utils::dbg, window::PrimaryWindow,
};

use crate::{
    camera::MainCamera,
    hex::{Hex, HexMap, HexPosition, HexStatus, HexStructure},
    turrets::{
        AimVec, Antenna, AntennaBundle, AntennaTextureAtlas, FireflyFactoryBundle,
        FireflyFactoryTextureAtlas, ReloadTimer, Turret, TurretBundle, TurretTextureAtlas,
    },
};

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldCoords>()
            .init_resource::<CursorHexPosition>()
            .init_resource::<SpawnSelectedStructure>()
            .init_resource::<PrevSelectedStructure>()
            .init_resource::<SelectedStructure>()
            .add_systems(
                Update,
                (
                    cursor_system,
                    spawn_structure_on_click,
                    update_antenna_aim_point,
                    select_spawn_structure,
                    select_structure,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub(crate) struct CursorWorldCoords {
    pub(crate) pos: Vec2,
}

#[derive(Resource, Default)]
pub(crate) struct CursorHexPosition {
    pub(crate) hex: HexPosition,
}

#[derive(Resource, Default)]
pub(crate) struct SelectedStructure {
    pub(crate) structure: HexStructure,
}

#[derive(Resource, Default)]
pub(crate) struct PrevSelectedStructure {
    pub(crate) structure: HexStructure,
}

#[derive(Resource, Default)]
pub(crate) enum SpawnSelectedStructure {
    #[default]
    Turret,
    Factory,
    Antenna,
}

fn cursor_system(
    mut cursor_coords: ResMut<CursorWorldCoords>,
    mut cursor_hex: ResMut<CursorHexPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
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

fn update_antenna_aim_point(
    mut q_antenna: Query<(&Transform, &mut AimVec, With<Antenna>)>,
    cursor_coords: Res<CursorWorldCoords>,
    selected_structure: Res<SelectedStructure>,
    buttons: Res<Input<MouseButton>>,
) {
    match (
        selected_structure.structure.entity,
        buttons.get_pressed().last(),
    ) {
        // TODO: Add a target component to the antenna bundle and use that to calculate the aim vector and hex aim point.
        (Some(structure_entity), Some(MouseButton::Right)) => {
            if let Ok((transform, mut antenna_aim_vec, _)) = q_antenna.get_mut(structure_entity) {
                let v = (cursor_coords.pos - transform.translation.truncate()).try_normalize();
                *antenna_aim_vec = AimVec { v };
            }
        }
        _ => {}
    }
}

fn select_structure(
    mut selected_structure: ResMut<SelectedStructure>,
    mut prev_selected_structure: ResMut<PrevSelectedStructure>,
    cursor_hex: Res<CursorHexPosition>,
    q_hex_map: Query<&HexMap>,
    q_hex: Query<&HexStructure>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let hex_map = q_hex_map.single();
        if let Some(hex_entity) = hex_map.map.get(&cursor_hex.hex) {
            let hex_structure = q_hex.get(*hex_entity).expect("valid entity from map");
            match (hex_structure.entity, selected_structure.structure.entity) {
                (Some(curr), Some(prev)) => {
                    if curr != prev {
                        dbg!("new structure");
                        *prev_selected_structure = PrevSelectedStructure {
                            structure: selected_structure.structure,
                        }
                    }
                }
                _ => {}
            }
            selected_structure.structure = *hex_structure;
        }
    }
}

fn select_spawn_structure(
    mut spawn_structure: ResMut<SpawnSelectedStructure>,
    buttons: Res<Input<KeyCode>>,
) {
    match buttons.get_pressed().last() {
        Some(KeyCode::Key1) => *spawn_structure = SpawnSelectedStructure::Turret,
        Some(KeyCode::Key2) => *spawn_structure = SpawnSelectedStructure::Factory,
        Some(KeyCode::Key3) => *spawn_structure = SpawnSelectedStructure::Antenna,
        _ => {}
    }
}

fn spawn_structure_on_click(
    mut commands: Commands,
    mut q_hex: Query<(&HexStatus, &mut HexStructure, With<Hex>)>,
    q_hex_map: Query<&HexMap>,
    turret_texture_atlas: Res<TurretTextureAtlas>,
    antenna_texture_atlas: Res<AntennaTextureAtlas>,
    factory_texture_atlas: Res<FireflyFactoryTextureAtlas>,
    cursor_hex: Res<CursorHexPosition>,
    spawn_structure: Res<SpawnSelectedStructure>,
    buttons: Res<Input<MouseButton>>,
) {
    let hex_map = q_hex_map.single();
    if buttons.just_pressed(MouseButton::Left) && hex_map.contains(cursor_hex.hex) {
        let hex_entity = hex_map.map.get(&cursor_hex.hex).expect("valid cursor hex");
        let (_hex_status, mut hex_structure, _) =
            q_hex.get_mut(*hex_entity).expect("valid hex entity");
        if hex_structure.entity.is_some() {
            return;
        }
        let turret_v = cursor_hex.hex.pixel_coords();
        let entity_id = match spawn_structure.into_inner() {
            SpawnSelectedStructure::Turret => commands
                .spawn(TurretBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteSheetBundle {
                        texture_atlas: turret_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                })
                .id(),
            SpawnSelectedStructure::Factory => commands
                .spawn(FireflyFactoryBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteSheetBundle {
                        texture_atlas: factory_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                })
                .id(),
            SpawnSelectedStructure::Antenna => commands
                .spawn(AntennaBundle {
                    hex_pos: cursor_hex.hex,
                    reload_timer: ReloadTimer::from(3f32),
                    spritebundle: SpriteSheetBundle {
                        texture_atlas: antenna_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                })
                .id(),
        };
        *hex_structure = HexStructure::from_id(entity_id);
    }
}
