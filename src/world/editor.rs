use crate::{prelude::*, word::WordID};
use bevy::window::*;
use bevy_simple_tilemap::*;

use super::{LoadedWorld, DeWorld, TileIndex, WORLD_SIZE, dropdown::{DropdownBundle, Dropdown}, WordTagInWorld, WordTag, WordTagBundle, WorldObject};

#[derive(Default, States, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum WorldEditorState {
    On,
    #[default]
    Off,
}

#[derive(Default, Resource)]
pub struct MouseWorldCoords {
    position: Option<Vec2>,
}

pub fn set_mouse_world_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    interactions: Query<&Interaction, Changed<Interaction>>,
    mut interaction_lock: Local<bool>,
    mouse_button: Res<Input<MouseButton>>,
) {
    let (camera, camera_transform) = cameras.single();
    let window = windows.single();

    if mouse_button.just_pressed(MouseButton::Left) &&
        interactions.iter().any(|interaction| *interaction != Interaction::None) {
        // mouse click is interacting with something else.
        *interaction_lock = true;
    } else if mouse_button.just_released(MouseButton::Left) {
        *interaction_lock = false;
    }

    if *interaction_lock {
        mouse_world_coords.position = None;
        return;
    }

    mouse_world_coords.position = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());
}

#[derive(Component)]
pub struct WorldEditorUIParent;

pub fn open_world_editor(
    keys: Res<Input<KeyCode>>,
    edit_mode: Res<State<WorldEditorState>>,
    mut next_edit_mode: ResMut<NextState<WorldEditorState>>,
) {
    if keys.just_pressed(KeyCode::T) {
        if *edit_mode == WorldEditorState::Off {
            next_edit_mode.set(WorldEditorState::On);
            info!("enabling world edit mode.");
        } else {
            next_edit_mode.set(WorldEditorState::Off);
            info!("disabling world edit mode.");
        }
    }
}

#[derive(Component, Default)]
pub struct PlacementDropdown;

pub fn setup_world_editor_gui(
    mut commands: Commands,
) {
    let ui_parent = commands.spawn((
        WorldEditorUIParent, 
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        }
    )).id();

    commands.spawn(DropdownBundle {
        dropdown: Dropdown {
            options: vec!["World", "Word Tags"],
            chosen: 0,
        },
        marker: PlacementDropdown,
        ..default()
    }).set_parent(ui_parent);
}

pub fn teardown_world_editor_gui(
    mut commands: Commands,
    ui_parent: Query<Entity, With<WorldEditorUIParent>>,
) {
    let ui_parent = ui_parent.single();
    commands.entity(ui_parent).despawn_recursive();
}

pub fn edit_world(
    mut tilemaps: Query<(&Transform, &mut LoadedWorld, &mut TileMap, Entity)>,
    placement_dropdown: Query<&Dropdown, With<PlacementDropdown>>,
    mouse_button: Res<Input<MouseButton>>,
    mouse_world_coords: Res<MouseWorldCoords>,
    mut gizmos: Gizmos,
    mut commands: Commands,
) {
    let Some(mouse_position) = mouse_world_coords.position else { return };
    let placement_dropdown = placement_dropdown.single();

    for mut tilemap in &mut tilemaps {
        let pos_on_map = mouse_position - tilemap.0.translation.xy();
        gizmos.circle_2d(pos_on_map, 10.0, Color::GREEN);

        if mouse_button.get_pressed().is_empty() { return };

        let tiles = &mut tilemap.1.tiles;

        match placement_dropdown.chosen {
            0 => { // world
                let tile_pos = (pos_on_map + 8.0) / 16.0;
                let tile_pos = (tile_pos.x as usize, tile_pos.y as usize);

                if !(0..tiles.len()).contains(&tile_pos.1) ||
                    !(0..tiles[tile_pos.1].len()).contains(&tile_pos.0) {
                    continue;
                }
                
                let tile = if mouse_button.pressed(MouseButton::Left) {
                        TileIndex::Ground
                    } else {
                        TileIndex::Air
                    };

                tiles[tile_pos.1][tile_pos.0] = tile;

                let tile_opt = if tile == TileIndex::Air { 
                        None 
                    } else { 
                        Some(Tile { sprite_index: tile as u32, ..default() })
                    };
                tilemap.2.set_tile(IVec3::new(tile_pos.0 as i32, tile_pos.1 as i32, 0), tile_opt);
            },
            1 => {
                if !mouse_button.just_pressed(MouseButton::Left) { return }

                commands.spawn(WordTag::bundle(&WordTagInWorld {
                    word_id: WordID::Baby,
                    transform: Transform::from_translation(pos_on_map.extend(0.)),
                })).set_parent(tilemap.3);
            }
            _ => unreachable!(),
        }
    }
}
