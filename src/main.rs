use bevy::prelude::*;
use game::GamePlugin;

mod animation;
mod camera;
mod colors;
mod constants;
mod controls;
mod enemies;
mod game;
mod hex;
mod player;
mod projectiles;
mod turrets;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run()
}
