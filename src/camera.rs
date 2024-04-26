use bevy::prelude::*;

use crate::{game::UpdateInGameSet, player::Player};

pub(crate) struct CameraPluginHexTurret;

impl Plugin for CameraPluginHexTurret {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup);
        app.add_systems(
            Update,
            move_camera_to_player
                .in_set(UpdateInGameSet)
                .after(crate::player::move_player),
        );
    }
}

#[derive(Component)]
pub(crate) struct MainCamera;

pub(crate) fn camera_setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn move_camera_to_player(
    mut q_camera_transform: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    q_player_transform: Query<&Transform, (Without<MainCamera>, With<Player>)>,
) {
    let mut camera_transform = q_camera_transform.single_mut();
    let player_transform = q_player_transform.single();
    camera_transform.translation = player_transform.translation;
}
