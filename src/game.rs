use bevy::prelude::*;

use crate::{
    animation::HexTurretAnimationPlugin, camera::CameraPluginHexTurret, controls::ControlPlugin,
    enemies::EnemiesPlugin, hex::HexPlugin, player::PlayerPlugin, projectiles::ProjectilePlugin,
    turrets::TurretPlugin,
};

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HexPlugin)
            .add_plugins(CameraPluginHexTurret)
            .add_plugins(PlayerPlugin)
            .add_plugins(EnemiesPlugin)
            .add_plugins(TurretPlugin)
            .add_plugins(HexTurretAnimationPlugin)
            .add_plugins(ProjectilePlugin)
            .add_plugins(ControlPlugin);
    }
}
