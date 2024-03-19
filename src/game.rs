use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    animation::HexTurretAnimationPlugin, camera::CameraPluginHexTurret, controls::ControlPlugin,
    enemies::EnemiesPlugin, hex::HexPlugin, player::PlayerPlugin, projectiles::ProjectilePlugin,
    turrets::TurretPlugin,
};

pub(crate) struct GamePlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    AssetLoading,
    InGame,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_loading_state(
                LoadingState::new(AppState::AssetLoading).continue_to_state(AppState::InGame),
            )
            .add_plugins(HexPlugin)
            .add_plugins(CameraPluginHexTurret)
            .add_plugins(PlayerPlugin)
            .add_plugins(EnemiesPlugin)
            .add_plugins(TurretPlugin)
            .add_plugins(HexTurretAnimationPlugin)
            .add_plugins(ProjectilePlugin)
            .add_plugins(ControlPlugin);
    }
}
