use bevy::{prelude::*, window::PrimaryWindow};

use crate::{camera::MainCamera, hex::HexPosition};

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldCoords>()
            .init_resource::<CursorHexPosition>()
            .add_systems(Update, cursor_system);
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
