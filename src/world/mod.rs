use crate::prelude::*;
use bevy::{asset::{*, io::*}, app::AppExit, window::exit_on_all_closed};
use bevy_simple_tilemap::prelude::*;

mod editor;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_tilemap)
            .add_systems(Update, (
                save_world,
                editor::set_mouse_world_coords,
                editor::edit_world,
            ))
            .add_plugins(SimpleTileMapPlugin)
            .init_asset::<DeWorld>()
            .init_resource::<editor::MouseWorldCoords>()
            .register_asset_loader(WorldLoader)
            .add_systems(Update, spawn_world_on_load);
    }
}

const WORLD_SIZE: UVec2 = UVec2::splat(100);

#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize)]
pub struct DeWorld {
    tiles: Vec<Vec<TileIndex>>,
}

impl Default for DeWorld {
    fn default() -> Self {
        Self {
            tiles: vec![vec![TileIndex::Air; WORLD_SIZE.x as usize]; WORLD_SIZE.y as usize],
        }
    }
}

struct WorldLoader;

impl AssetLoader for WorldLoader {
    type Asset = DeWorld;

    type Settings = ();

    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<DeWorld, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.expect("unable to read world file");
            Ok(ron::de::from_bytes(&bytes).expect("unable to read world file"))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["world.ron"]
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TileIndex {
    Ground = 0,
    #[default]
    Air = u32::MAX,
}

#[derive(Component)]
pub struct LoadedWorld {
    handle: Handle<DeWorld>,
    loaded: bool,
}

fn setup_tilemap(
    assets: Res<DeAssets>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let mut texture_atlas = 
        TextureAtlas::new_empty(assets.tileset.clone(), Vec2::splat(16.0));
    texture_atlas.add_texture(Rect::from_center_size(Vec2::splat(8.0), Vec2::splat(16.0)));
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let tilemap_bundle = TileMapBundle {
        texture_atlas: texture_atlas_handle.clone(),
        ..Default::default()
    };

    let world = asset_server.load("jungle.world.ron");

    commands.spawn((tilemap_bundle, LoadedWorld { handle: world, loaded: false }));
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct TileMapQuery {
    tilemap: &'static mut TileMap,
    asset: &'static mut LoadedWorld,
    entity: Entity,
}

fn spawn_world_on_load(
    mut tilemaps: Query<TileMapQuery>,
    mut world_assets: ResMut<Assets<DeWorld>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for mut tilemap in &mut tilemaps {
        if tilemap.asset.loaded { continue }
        let Some(state) = asset_server.get_load_state(tilemap.asset.handle.id()) 
            else { continue };

        // in debug builds, create a new world file if one doesn't load.
        if state == LoadState::Failed || state == LoadState::Loaded {
            let world_asset_id = tilemap.asset.handle.id(); 

            #[cfg(debug_assertions)]
            if state == LoadState::Failed && world_assets.get(tilemap.asset.handle.id()).is_none() {
                world_assets.insert(world_asset_id, DeWorld::default());
            }

            commands.entity(tilemap.entity).despawn_descendants();

            tilemap.asset.loaded = true;
            load_world(world_asset_id, &*world_assets, &mut tilemap, &mut commands);
        }
    }
}

fn load_world(
    world_asset_id: AssetId<DeWorld>,
    world_assets: &Assets<DeWorld>,
    tilemap: &mut TileMapQueryItem,
    commands: &mut Commands,
) {
    let world = world_assets.get(world_asset_id).unwrap();

    tilemap.tilemap.clear();
    commands.entity(tilemap.entity).despawn_descendants();

    for x in 0..(WORLD_SIZE.x as usize) {
        for y in 0..(WORLD_SIZE.y as usize) {
            let tile = world.tiles[y][x];
            let position = IVec3::new(x as i32, y as i32, 0);

            if tile == TileIndex::Air {  
                //tilemap.set_tile(position, None);
            } else {
                tilemap.tilemap.set_tile(position, Some(Tile {
                    sprite_index: tile as u32,
                    ..default()
                }));
            }
        }
    }

    let world_colliders = calculate_world_colliders(world);
    for collider in world_colliders {
        commands.spawn(collider).set_parent(tilemap.entity);
    }
}

#[derive(Default, Bundle)]
struct WorldCollderBundle {
    global: GlobalTransform,
    transform: Transform,
    rigidbody: RigidBody,
    collider: Collider,
}

fn calculate_world_colliders(world: &DeWorld) -> Vec<WorldCollderBundle> {
    let mut filled = [[false; WORLD_SIZE.x as usize]; WORLD_SIZE.y as usize];

    let mut output = Vec::<WorldCollderBundle>::new();

    for start_x in 0..(WORLD_SIZE.x as usize) {
        for start_y in 0..(WORLD_SIZE.y as usize) {
            let mut x = start_x;
            let mut y = start_y;

            if world.tiles[y][x] != TileIndex::Air && !filled[y][x] {
                while x < WORLD_SIZE.x as usize &&
                    world.tiles[y][x] != TileIndex::Air && 
                    !filled[y][x] {
                    x += 1;
                }

                while y < WORLD_SIZE.y as usize &&
                    world.tiles[y][start_x..x].iter().all(|t| *t != TileIndex::Air) &&
                    filled[y][start_x..x].iter().all(|f| !*f) {
                    y += 1;
                }

                for fill_x in start_x..x {
                    for fill_y in start_y..y {
                        filled[fill_y][fill_x] = true;
                    }
                }

                let rect = Rect {
                    min: Vec2::new(start_x as f32, start_y as f32) * 16.0,
                    max: Vec2::new(x as f32, y as f32) * 16.0,
                };

                output.push(WorldCollderBundle {
                    transform: Transform {
                        translation: (rect.center() - Vec2::splat(8.)).extend(0.),
                        ..default()
                    },
                    // we divide by 2 because cuboid takes half sizes
                        collider: Collider::cuboid(rect.size().x / 2., rect.size().y / 2.),
                    rigidbody: RigidBody::Fixed,
                    ..default()
                })
            }
        }
    }

    output
}

fn save_world(
    exit_events: EventReader<AppExit>,
    world_assets: Res<Assets<DeWorld>>,
    asset_server: Res<AssetServer>,
    keyboard: Res<Input<KeyCode>>,
) {
    use std::path::*;
    use std::fs::*;

    if keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::S) {
        for (world_asset_id, world) in world_assets.iter() {
            let world_path = asset_server.get_path(world_asset_id).unwrap();
            let file_path = Path::new("./assets").join(world_path.path());
            
            info!("saving world asset {file_path:?}");

            let serialized_world = ron::ser::to_string(world)
                .expect("unable to serialize world");
            write(file_path, &*serialized_world).expect("unable to write {file_path:?}");
        }
    }
}
