use std::f32::consts::PI;

use bevy_ecs_tilemap::prelude::*;
use crate::{prelude::*, word::WordID, world::helpers::level_is_in_position};
use bevy::window::*;
use bevy_rapier2d::prelude::shape_views::CuboidView;

mod dropdown;
use dropdown::*;
use super::*;

pub struct WorldEditorPlugin;
impl Plugin for WorldEditorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(dropdown::DropdownPlugin)
            .add_systems(Update, (
                set_mouse_world_coords,
                open_world_editor,
                edit_world
                    .after(setup_world_editor_gui)
                    .run_if(in_state(WorldEditorState::On)),
                refresh_tilemap.after(save_and_load::spawn_level_on_load)
            ).chain())
            .add_systems(OnEnter(WorldEditorState::On), setup_world_editor_gui)
            .add_systems(OnExit(WorldEditorState::On), teardown_world_editor_gui);
    }
}

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

const EDITOR_ZOOM: f32 = 2.0;
pub fn setup_world_editor_gui(
    mut camera: Query<(&mut OrthographicProjection, &mut GameCamera), With<Camera>>,
    mut commands: Commands,
) {
    let mut camera = camera.single_mut();
    camera.1.move_mode = CameraMoveMode::Free;
    camera.0.scale *= EDITOR_ZOOM;
    
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
            choices: vec!["World", "Word Tags", "Lock Zones", "Player Spawner", "Fan", 
                "Multiselect", "Move Player", "Snap and Visualize Objects Movement", 
                "Death Zones", "Camera Zones"],
            chosen: 0,
        },
        marker: PlacementDropdown,
        ..default()
    }).set_parent(ui_parent);
}

pub fn teardown_world_editor_gui(
    mut camera: Query<(&mut OrthographicProjection, &mut GameCamera), With<Camera>>,
    mut commands: Commands,
    ui_parent: Query<Entity, With<WorldEditorUIParent>>,
) {
    let mut camera = camera.single_mut();
    camera.1.move_mode = CameraMoveMode::SnapToBounds;
    camera.0.scale /= EDITOR_ZOOM;

    let ui_parent = ui_parent.single();
    commands.entity(ui_parent).despawn_recursive();
}

#[derive(SystemParam)]
pub struct EditorState<'s> {
    multiselect_coords: Local<'s, (Vec2, Vec2)>,
}

pub fn edit_world(
    mut levels: Query<(&Transform, &mut LoadedLevel, Entity)>,
    placement_dropdown: Query<&Dropdown, With<PlacementDropdown>>,
    mut player: Query<&mut Transform, (With<Player>, Without<LoadedLevel>)>,
    mut other_objects: Query<(&mut Transform, Option<&Collider>, &GlobalTransform), 
                             (Without<Player>, Without<LoadedLevel>)>,
    children: Query<&Children>,
    mouse_button: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mouse_world_coords: Res<MouseWorldCoords>,
    assets: Res<MiscAssets>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut editor: EditorState,
) {
    let placement_dropdown = placement_dropdown.single();

    for mut level in &mut levels {
        use KeyCode as KC;
        use MouseButton as MB;

        let Some(mouse_position) = mouse_world_coords.position else { return };
        let pos_on_map = mouse_position;

        let Some(level_rect) = level_is_in_position((&*level.1, level.0), pos_on_map)
            else { continue };

        gizmos.circle_2d(pos_on_map, 10.0, Color::GREEN);

        for object in children.iter_descendants(level.2) {
           if let Ok((transformation, collider, global)) = other_objects.get_mut(object) {
               if let Some(collider) = collider {
                   match collider.as_typed_shape() {
                       ColliderView::Cuboid(CuboidView { raw }) => {
                           gizmos.rect_2d(
                               global.translation().xy(),
                               0.,
                               Vec2::from(raw.half_extents) * 2.,
                               Color::RED,
                           );
                       },
                       _ => error!("could not visualize: {collider:?}"),
                   }
               } else {
                   gizmos.circle_2d(
                       global.translation().xy(),
                       5.,
                       Color::RED,
                   );
               }
           }
        }

        gizmos.rect_2d(level_rect.center(), 0., level_rect.size(), Color::BLUE);

        // secondary match to handle indicators based on the dropdown
        match placement_dropdown.chosen {
            5 => {
                gizmos.rect_2d(
                    (editor.multiselect_coords.0 + editor.multiselect_coords.1) / 2.,
                    0., 
                    editor.multiselect_coords.1 - editor.multiselect_coords.0, 
                    Color::GREEN
                );
            },
            _ => {},
            }

        // primary match to handle interactions
        match placement_dropdown.chosen {
            0 if !mouse_button.get_pressed().is_empty() => { // world
                let tile_pos = (pos_on_map - level.0.translation.xy() + 8.0) / 16.0;
                let tile_pos = (tile_pos.x as usize, tile_pos.y as usize);

                let tiles = &mut level.1.tiles;

                if !(0..tiles.cols()).contains(&tile_pos.1) ||
                   !(0..tiles.rows()).contains(&tile_pos.0) {
                    continue;
                }

                let tile = if mouse_button.pressed(MouseButton::Left) {
                        TileIndex::Ground
                    } else {
                        TileIndex::Air
                    };

                tiles[(tile_pos.0, tile_pos.1)] = tile;
            },
            1 if mouse_button.just_pressed(MB::Left) => {
                commands.spawn(WordTag::bundle(
                    &WordTagInWorld {
                        word_id: WordID::Baby,
                        transform: Transform::from_translation(pos_on_map.extend(0.)),
                    },
                    &*assets,
                )).set_parent(level.2);
            },
            2 if mouse_button.just_pressed(MB::Left) => {
                commands.spawn(LockZone::bundle(
                    &LockZoneInWorld {
                        transform: Transform::from_translation(pos_on_map.extend(-2.)),
                    },
                    &*assets,
                )).set_parent(level.2);
            },
            3 if mouse_button.just_pressed(MB::Left) => {
                commands.spawn(PlayerSpawner::bundle(
                    &PlayerSpawnerInWorld {
                        transform: Transform::from_translation(pos_on_map.extend(0.)),
                    },
                    &*assets,
                )).set_parent(level.2);
            }
            4 if mouse_button.just_pressed(MB::Left) => {
                commands.spawn(Fan::bundle(
                    &FanInWorld {
                        strength: 1.8,
                        translation: pos_on_map,
                        rotation: 0.,
                        scale: default(),
                    },
                    &*assets,
                )).set_parent(level.2);
            }
            5 if mouse_button.just_pressed(MB::Left) => {
                editor.multiselect_coords.0 = pos_on_map;
            },
            5 if mouse_button.pressed(MB::Left) => {
                editor.multiselect_coords.1 = pos_on_map;
            },
            5 if mouse_button.just_released(MB::Left) => {
                editor.multiselect_coords.0 =
                     (editor.multiselect_coords.0 / 16.).floor() * 16. + 8.;
                editor.multiselect_coords.1 =
                     (editor.multiselect_coords.1 / 16.).floor() * 16. + 8.;
            },
            5 if keys.any_just_pressed([KC::Left, KC::Right, KC::Up, KC::Down]) => {
                let tmove_dir = if keys.just_pressed(KC::Left) { IVec2::new(-1, 0) } 
                else if keys.just_pressed(KC::Right) { IVec2::new(1, 0) } 
                else if keys.just_pressed(KC::Up) { IVec2::new(0, 1) } 
                else if keys.just_pressed(KC::Down) { IVec2::new(0, -1) } 
                else { unreachable!() };
                
                let tilemap_copy = level.1.tiles.clone();
                
                let tmultiselect_start: IVec2 = 
                    (editor.multiselect_coords.0 / 16. - level.0.translation.xy()).as_ivec2();
                let tmultiselect_end: IVec2 = 
                    (editor.multiselect_coords.1 / 16. - level.0.translation.xy()).as_ivec2();

                // not sure why we have to add one here, but otherwise the movement rect
                // is off.
                let tstart = tmultiselect_start.min(tmultiselect_end) + 1;
                let tend = tmultiselect_start.max(tmultiselect_end) + 1;

                for y in tstart.y..tend.y {
                    for x in tstart.x..tend.x {
                        if !(tstart.y..tend.y).contains(&(y - tmove_dir.y)) || 
                          !(tstart.x..tend.x).contains(&(x - tmove_dir.x)) {
                            level.1.tiles[(x as usize, y as usize)] = TileIndex::Air;
                        }

                        level.1.tiles[((x + tmove_dir.x) as usize, (y + tmove_dir.y) as usize)] = 
                            tilemap_copy[(x as usize, y as usize)];
                    }
                }

                let wmove_dir = tmove_dir.as_vec2() * 16.;
                editor.multiselect_coords.0 += wmove_dir;
                editor.multiselect_coords.1 += wmove_dir;

                let wstart = tstart.as_vec2() * 16.;
                let wend = tend.as_vec2() * 16.;

                for object in children.iter_descendants(level.2) {
                    if let Ok(mut object) = other_objects.get_mut(object) {
                        let pos = &mut object.2.translation();
                        if wstart.x <= pos.x && pos.x <= wend.x && 
                          wstart.y <= pos.y && pos.y <= wend.y {
                            *pos += wmove_dir.extend(0.);
                        }
                    }
                }
            },
            6 if mouse_button.just_pressed(MB::Left) =>  {
                player.single_mut().translation = pos_on_map.extend(0.);
            },
            7 if mouse_button.just_released(MB::Left) => {
                for object in children.iter_descendants(level.2) {
                   if let Ok((mut transformation, ..)) = other_objects.get_mut(object) {
                       transformation.translation = 
                           (transformation.translation / 8.).round() * 8.;
                       transformation.scale = transformation.scale.round();
                   }
                }
            },
            8 if mouse_button.just_pressed(MB::Left) => {
                commands.spawn(DeathZone::bundle(
                    &DeathZoneInWorld {
                        transform: Transform::from_translation(pos_on_map.extend(0.)),
                    },
                    &*assets,
                )).set_parent(level.2);
            }
            _ => {},
        }
    }
}

// this system triggers both on changes from the editor and from changes on asset loading.
// i don't really like how these aren't explicitly hooked up, and i think it would be
// a better idea to make this a callable function in the future.
fn refresh_tilemap(
    mut worlds: Query<
        (&LoadedLevel, &mut TileStorage, &mut TilemapSize, Option<&Children>, Entity), 
        Changed<LoadedLevel>
    >,
    asset_events: EventReader<AssetEvent<DeLevel>>,
    tiles: Query<(), Or<(With<TilePos>, With<Collider>)>>,
    mut commands: Commands,
) {
    for mut world in &mut worlds {
        *world.2 = TilemapSize { x: world.0.tiles.cols() as u32, y: world.0.tiles.rows() as u32};
        *world.1 = TileStorage::empty(*world.2);

        if asset_events.is_empty() && let Some(children) = world.3 {
            for child in children {
                if tiles.get(*child).is_ok() { 
                    commands.entity(*child).despawn_recursive();
                }
            }
        }

        for ((x, y), tile) in world.0.tiles.indexed_iter() {
            let tile_pos = TilePos { x: x as u32, y: y as u32 };

            if *tile == TileIndex::Air {
                // tile is already none, we don't have to set it
                world.1.remove(&tile_pos);
            } else {
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(world.4),
                        ..Default::default()
                    })
                    .set_parent(world.4)
                    .id();

                world.1.set(&tile_pos, tile_entity);
            }
        }

        let world_colliders = calculate_world_colliders(&world.0.tiles);
        for collider in world_colliders {
            commands.spawn(collider).set_parent(world.4);
        }
    }
}
