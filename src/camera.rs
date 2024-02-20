use bevy::prelude::*;

pub(crate) struct CameraPluginHexTurret;

impl Plugin for CameraPluginHexTurret {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup);
    }
}

#[derive(Component)]
pub(crate) struct MainCamera;

pub(crate) fn camera_setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}
