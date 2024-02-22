use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    camera::MainCamera,
    hex::{HexMap, HexPosition, HexStatus},
    turrets::{
        AntennaBundle, AntennaTextureAtlas, FireflyFactoryBundle, FireflyFactoryTextureAtlas,
        Turret, TurretBundle, TurretTextureAtlas,
    },
};

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldCoords>()
            .init_resource::<CursorHexPosition>()
            .init_resource::<SelectedStructure>()
            .add_systems(
                Update,
                (cursor_system, spawn_structure_on_click, select_structure),
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
pub(crate) enum SelectedStructure {
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

fn select_structure(
    mut selected_structure: ResMut<SelectedStructure>,
    buttons: Res<Input<KeyCode>>,
) {
    match buttons.get_pressed().last() {
        Some(KeyCode::Key1) => *selected_structure = SelectedStructure::Turret,
        Some(KeyCode::Key2) => *selected_structure = SelectedStructure::Factory,
        Some(KeyCode::Key3) => *selected_structure = SelectedStructure::Antenna,
        _ => {}
    }
}

fn spawn_structure_on_click(
    mut commands: Commands,
    q_hex: Query<&HexStatus>,
    q_hex_map: Query<&HexMap>,
    turret_texture_atlas: Res<TurretTextureAtlas>,
    antenna_texture_atlas: Res<AntennaTextureAtlas>,
    factory_texture_atlas: Res<FireflyFactoryTextureAtlas>,
    cursor_hex: Res<CursorHexPosition>,
    selected_structure: Res<SelectedStructure>,
    buttons: Res<Input<MouseButton>>,
) {
    let hex_map = q_hex_map.single();
    if buttons.just_pressed(MouseButton::Left) && hex_map.contains(cursor_hex.hex) {
        let hex_entity = hex_map.map.get(&cursor_hex.hex);
        if q_hex
            .get(*hex_entity.unwrap())
            .is_ok_and(|hex_status| hex_status != &HexStatus::Neutral)
        {
            return;
        }
        let turret_v = cursor_hex.hex.pixel_coords();
        match selected_structure.into_inner() {
            SelectedStructure::Turret => {
                commands.spawn(TurretBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteSheetBundle {
                        texture_atlas: turret_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                });
            }
            SelectedStructure::Factory => {
                commands.spawn(FireflyFactoryBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteSheetBundle {
                        texture_atlas: factory_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                });
            }
            SelectedStructure::Antenna => {
                commands.spawn(AntennaBundle {
                    hex_pos: cursor_hex.hex,
                    sprite: SpriteSheetBundle {
                        texture_atlas: antenna_texture_atlas.atlas.clone(),
                        transform: Transform::from_xyz(turret_v.x, turret_v.y, 2f32),
                        ..default()
                    },
                    ..default()
                });
            }
        };
    }
}
