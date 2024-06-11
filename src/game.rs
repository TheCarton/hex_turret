use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    animation::HexTurretAnimationPlugin, camera::CameraPluginHexTurret, controls::ControlPlugin,
    enemies::EnemiesPlugin, hex::HexPlugin, player::PlayerPlugin, projectiles::ProjectilePlugin,
    turrets::TurretPlugin,
};

pub(crate) struct GamePlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) enum AppState {
    #[default]
    AssetLoading,
    InGame,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[allow(dead_code)]
pub(crate) enum PauseState {
    Paused,
    #[default]
    Running,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AssetLoadingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EnterGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FixedUpdateInGameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct UpdateInGameSet;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_loading_state(
                LoadingState::new(AppState::AssetLoading).continue_to_state(AppState::InGame),
            )
            .init_state::<PauseState>()
            .configure_sets(OnEnter(AppState::InGame), EnterGameSet)
            .configure_sets(
                FixedUpdate,
                FixedUpdateInGameSet.run_if(in_state(AppState::InGame)),
            )
            .configure_sets(Update, UpdateInGameSet.run_if(in_state(AppState::InGame)))
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
