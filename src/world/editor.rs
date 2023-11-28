use crate::{prelude::*, word::WordID};
use bevy::window::*;
use bevy_simple_tilemap::*;

use super::{LoadedWorld, DeWorld, TileIndex, WORLD_SIZE, dropdown::{DropdownBundle, Dropdown}, WordTagInWorld, WordTag};

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
    mut tilemaps: Query<(&Transform, &mut LoadedWorld)>,
    placement_dropdown: Query<&Dropdown, With<PlacementDropdown>>,
    mouse_button: Res<Input<MouseButton>>,
    mut world_assets: ResMut<Assets<DeWorld>>,
    mouse_world_coords: Res<MouseWorldCoords>,
    mut gizmos: Gizmos,
    mut asset_events: EventWriter<AssetEvent<DeWorld>>,
) {
    let Some(mouse_position) = mouse_world_coords.position else { return };
    let placement_dropdown = placement_dropdown.single();

    for tilemap in &mut tilemaps {
        let pos_on_map = mouse_position - tilemap.0.translation.xy();
        gizmos.circle_2d(pos_on_map, 10.0, Color::GREEN);

        if mouse_button.get_pressed().is_empty() { return };

        let world = world_assets.get_mut(tilemap.1.handle.id()).unwrap();

        match placement_dropdown.chosen {
            0 => { // world
                let tile_pos = (pos_on_map + 8.0) / 16.0;
                let tile_pos = (tile_pos.x as usize, tile_pos.y as usize);

                if !(0..world.tiles.len()).contains(&tile_pos.1) ||
                    !(0..world.tiles[0].len()).contains(&tile_pos.0) {
                    continue;
                }
                
                world.tiles[tile_pos.1][tile_pos.0] = 
                    if mouse_button.pressed(MouseButton::Left) {
                        TileIndex::Ground
                    } else {
                        TileIndex::Air
                    };
            },
            1 => {
                if !mouse_button.just_pressed(MouseButton::Left) { return }

                world.word_tags.push(WordTagInWorld {
                    word_id: WordID::Baby,
                    transform: Transform {
                        translation: pos_on_map.extend(0.0),
                        ..default()
                    },
                });
            }
            _ => unreachable!(),
        }
        
        asset_events.send(AssetEvent::Modified {
            id: tilemap.1.handle.id(),
        });
    }
}

pub fn update_positions_for_world(
    changed_word_tags: Query<(), (Changed<Transform>, With<WordTag>)>,
    all_word_tags: Query<(&WordTag, Ref<Transform>, &Parent)>,
    tilemaps: Query<&LoadedWorld>,
    mut world_assets: ResMut<Assets<DeWorld>>,
) {
    if !changed_word_tags.is_empty() {
        if all_word_tags.iter().any(|tag| tag.1.is_added()) { 
            // tags are added, not changed after adding
            return;
        }

        let world = **all_word_tags.iter().next().unwrap().2;
        let tilemap = tilemaps.get(world).unwrap();
        let world_asset = world_assets.get_mut(tilemap.handle.clone()).unwrap();

        world_asset.word_tags = all_word_tags
            .iter()
            .map(|(tag, transform, _)| WordTagInWorld {
                word_id: tag.word_id,
                transform: *transform,
            })
            .collect();
    }
}
