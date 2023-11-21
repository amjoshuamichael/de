use bevy::{prelude::*, window::*};
use bevy_simple_tilemap::TileMap;

use super::{LoadedWorld, DeWorld, TileIndex, WORLD_SIZE};

#[derive(Default, Resource)]
pub struct MouseWorldCoords {
    position: Option<Vec2>,
}

pub fn set_mouse_world_coords(
    mut mouse_world_coords: ResMut<MouseWorldCoords>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_transform) = cameras.single();
    let window = windows.single();

    mouse_world_coords.position = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());
}

pub fn edit_world(
    mut tilemaps: Query<(&Transform, &mut LoadedWorld)>,
    mouse_button: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut world_assets: ResMut<Assets<DeWorld>>,
    mouse_world_coords: Res<MouseWorldCoords>,
    mut gizmos: Gizmos,
    mut edit_mode: Local<bool>,
) {
    let edit_mode = &mut *edit_mode;

    if keys.just_pressed(KeyCode::T) {
        *edit_mode = !*edit_mode;
        info!("edit_mode: {edit_mode:?}");
    }

    if !*edit_mode { return };

    if !mouse_button.just_pressed(MouseButton::Left) { return };
    let Some(mouse_position) = mouse_world_coords.position else { return };

    for mut tilemap in &mut tilemaps {
        let pos_on_map = mouse_position - tilemap.0.translation.xy();
        gizmos.circle_2d(pos_on_map, 10.0, Color::GREEN);
        gizmos.circle_2d(mouse_position, 10.0, Color::BLUE);
        gizmos.circle_2d(tilemap.0.translation.xy(), 10.0, Color::RED);
        let tile_pos = (pos_on_map + 8.0) / 16.0;
        let tile_pos = (tile_pos.x as usize, tile_pos.y as usize);

        if !mouse_button.just_pressed(MouseButton::Left) { return };

        let world = world_assets.get_mut(tilemap.1.handle.id()).unwrap();

        if !(0..world.tiles.len()).contains(&tile_pos.1) ||
            !(0..world.tiles[0].len()).contains(&tile_pos.0) {
            continue;
        }
        
        world.tiles[tile_pos.1][tile_pos.0] = TileIndex::Ground;
        tilemap.1.loaded = false;
    }
}
