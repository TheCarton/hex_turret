use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    constants::ANTENNA_FIRE_RATE,
    game::AppState,
    hex::{Hex, HexMap, HexPosition, HexStatus, HexStructure},
    turrets::{
        AimVec, Antenna, AntennaAssets, AntennaBundle, FactoryAssets, FactoryBundle, ReloadTimer,
        TurretAssets, TurretBundle,
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
                    update_antenna_target,
                    select_spawn_structure,
                    select_structure,
                )
                    .run_if(in_state(AppState::InGame)),
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

fn update_antenna_target(
    q_hex_map: Query<&HexMap>,
    mut q_antenna: Query<&mut AimVec, With<Antenna>>,
    cursor_coords: Res<CursorWorldCoords>,
    selected_structure: Res<SelectedStructure>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    match (
        selected_structure.structure.entity,
        buttons.get_pressed().last(),
    ) {
        // TODO: Add a target component to the antenna bundle and use that to calculate the aim vector and hex aim point.
        (Some(structure_entity), Some(MouseButton::Right)) => {
            let hex_map = q_hex_map.single();
            if let Ok(mut antenna_aim_vec) = q_antenna.get_mut(structure_entity) {
                if hex_map
                    .map
                    .get(&HexPosition::from_pixel(cursor_coords.pos))
                    .is_some()
                {
                    *antenna_aim_vec = AimVec {
                        v: Some(cursor_coords.pos),
                    };
                }
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
    buttons: Res<ButtonInput<MouseButton>>,
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
    buttons: Res<ButtonInput<KeyCode>>,
) {
    match buttons.get_pressed().last() {
        Some(KeyCode::Digit1) => *spawn_structure = SpawnSelectedStructure::Turret,
        Some(KeyCode::Digit2) => *spawn_structure = SpawnSelectedStructure::Factory,
        Some(KeyCode::Digit3) => *spawn_structure = SpawnSelectedStructure::Antenna,
        _ => {}
    }
}

pub(crate) fn spawn_structure_on_click(
    mut commands: Commands,
    mut q_hex: Query<(&HexStatus, &mut HexStructure), With<Hex>>,
    q_hex_map: Query<&HexMap>,
    turret_texture_atlas: Res<TurretAssets>,
    antenna_texture_atlas: Res<AntennaAssets>,
    factory_texture_atlas: Res<FactoryAssets>,
    cursor_hex: Res<CursorHexPosition>,
    spawn_structure: Res<SpawnSelectedStructure>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let hex_map = q_hex_map.single();
    if buttons.just_pressed(MouseButton::Left) && hex_map.contains(cursor_hex.hex) {
        let hex_entity = hex_map.map.get(&cursor_hex.hex).expect("valid cursor hex");
        let (_hex_status, mut hex_structure) =
            q_hex.get_mut(*hex_entity).expect("valid hex entity");
        if hex_structure.entity.is_some() {
            return;
        }
        let turret_v = cursor_hex.hex.pixel_coords();
        let entity_id = match spawn_structure.into_inner() {
            SpawnSelectedStructure::Turret => commands
                .spawn(TurretBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteBundle {
                        texture: turret_texture_atlas.turret.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                })
                .id(),
            SpawnSelectedStructure::Factory => commands
                .spawn(FactoryBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteBundle {
                        texture: factory_texture_atlas.factory.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                })
                .id(),
            SpawnSelectedStructure::Antenna => commands
                .spawn(AntennaBundle {
                    hex_pos: cursor_hex.hex,
                    reload_timer: ReloadTimer::from(ANTENNA_FIRE_RATE),
                    spritebundle: SpriteBundle {
                        texture: antenna_texture_atlas.antenna.clone(),
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
