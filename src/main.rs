use bevy::prelude::*;
use itertools::Itertools;
mod colors;
const HEX_SIZE: f32 = 25.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Turret Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup, spawn_map))
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct Map {
    size: i8,
}

#[derive(Component)]
struct Hex {
    q: i8,
    r: i8,
}

impl Hex {
    fn pixel_coords(&self) -> Vec2 {
        let x =
            HEX_SIZE * (3f32.sqrt() * f32::from(self.q) + 3f32.sqrt() / 2f32 * f32::from(self.r));
        let y = HEX_SIZE * (3f32 / 2f32 * f32::from(self.r));
        Vec2::new(x, y)
    }

    fn from_qr(q: i8, r: i8) -> Hex {
        Hex { q, r }
    }
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map = Map { size: 4 };
    let physical_map_size = f32::from(map.size) * HEX_SIZE;
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: colors::BOARD,
                custom_size: Some(Vec2::new(physical_map_size, physical_map_size)),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            (-map.size..map.size)
                .cartesian_product(-map.size..map.size)
                .filter_map(|(q, r)| {
                    let s = -q - r;
                    if q + r + s == 0 {
                        Some(Hex::from_qr(q, r))
                    } else {
                        None
                    }
                })
                .for_each({
                    |hex| {
                        let pos = hex.pixel_coords();
                        builder.spawn(SpriteBundle {
                            texture: asset_server.load("bw-tile-hex-row.png"),
                            transform: Transform::from_xyz(pos.x, pos.y, 1.0),
                            ..default()
                        });
                    }
                });
        });
}
