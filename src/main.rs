use bevy::prelude::*;
use game::GamePlugin;
use gui::GuiPlugin;

mod animation;
mod camera;
mod colors;
mod constants;
mod controls;
mod enemies;
mod game;
mod gui;
mod hex;
mod player;
mod projectiles;
mod tools;
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
        .add_plugins(GuiPlugin)
        .run()
}
