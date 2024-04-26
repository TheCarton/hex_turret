use bevy::prelude::*;

use crate::{
    controls::{SelectedStructure, SpawnSelectedStructure},
    turrets::Structure,
};

pub(crate) struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, gui_setup);
        app.add_systems(Update, (show_selected_structure, show_to_spawn_structure));
    }
}

#[derive(Component)]
struct FooterSelectedStructureText;

#[derive(Bundle)]
struct FooterSelectedStructureTextBundle {
    selected_structure: FooterSelectedStructureText,
    text_bundle: TextBundle,
}

#[derive(Component)]
struct FooterSpawnStructureText;

#[derive(Bundle)]
struct FooterSpawnStructureTextBundle {
    spawn_structure: FooterSpawnStructureText,
    text_bundle: TextBundle,
}

fn gui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
                grid_template_rows: vec![
                    GridTrack::auto(),
                    GridTrack::flex(1.0),
                    GridTrack::percent(15f32),
                ],
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        align_items: AlignItems::Start,
                        justify_items: JustifyItems::Start,
                        grid_row: GridPlacement::start(3),
                        grid_column: GridPlacement::span(2),
                        grid_template_columns: RepeatedGridTrack::flex(2, 1f32),
                        grid_template_rows: RepeatedGridTrack::flex(1, 1f32),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::GRAY),
                    ..default()
                })
                .with_children(|builder| {
                    builder.spawn(FooterSelectedStructureTextBundle {
                        selected_structure: FooterSelectedStructureText,
                        text_bundle: TextBundle::from_section(
                            "Footer",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24f32,
                                ..default()
                            },
                        ),
                    });
                    builder.spawn(FooterSpawnStructureTextBundle {
                        spawn_structure: FooterSpawnStructureText,
                        text_bundle: TextBundle::from_section(
                            "idk",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24f32,
                                ..default()
                            },
                        ),
                    });
                });
        });
}

fn show_selected_structure(
    selected_structure: Res<SelectedStructure>,
    mut q_footer_text: Query<&mut Text, With<FooterSelectedStructureText>>,
    q_structure: Query<&Structure>,
) {
    let new_text = selected_structure
        .curr_structure
        .map(|e| q_structure.get(e))
        .map(|v| {
            if let Ok(structure) = v {
                structure.string()
            } else {
                "Nothing selected: Entity not found.".to_string()
            }
        })
        .unwrap_or("Nothing selected".to_string());
    let mut footer_text = q_footer_text.single_mut();
    footer_text.sections[0].value = new_text;
}

fn show_to_spawn_structure(
    selected_structure: Res<SpawnSelectedStructure>,
    mut q_footer_text: Query<&mut Text, With<FooterSpawnStructureText>>,
) {
    let new_text = selected_structure.string();
    let mut footer_text = q_footer_text.single_mut();
    footer_text.sections[0].value = new_text;
}
