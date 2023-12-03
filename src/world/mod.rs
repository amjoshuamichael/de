use crate::{prelude::*, word::WordID};
use bevy::{asset::{*, io::*}, app::AppExit, window::exit_on_all_closed};
use bevy_simple_tilemap::prelude::*;
use ron::ser::PrettyConfig;

use self::editor::WorldEditorState;

mod editor;
pub mod dropdown;
mod word_tag;
mod lock_zone;
mod player_spawner;
mod fan;

pub use word_tag::*;
pub use lock_zone::*;
pub use player_spawner::*;
pub use fan::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_tilemap)
            .add_systems(Update, (
                save_world,
                spawn_world_on_load,
            ))
            .add_state::<WorldEditorState>()
            .add_systems(Update, (
                editor::set_mouse_world_coords,
                editor::open_world_editor,
                editor::edit_world
                    .after(editor::setup_world_editor_gui)
                    .run_if(in_state(WorldEditorState::On)),
            ).chain())
            .add_systems(Update, (
                word_tag::init_word_tags,
                word_tag::word_tags_update,
                lock_zone::lock_zone_update,
                player_spawner::player_spawner_update,
                fan::fans_update.before(crate::word::SentenceModificationRoutine),
            ))
            .add_systems(OnEnter(WorldEditorState::On), editor::setup_world_editor_gui)
            .add_systems(OnExit(WorldEditorState::On), editor::teardown_world_editor_gui)
            .add_plugins(SimpleTileMapPlugin)
            .init_asset::<DeWorld>()
            .init_resource::<editor::MouseWorldCoords>()
            .register_asset_loader(WorldLoader);
    }
}

const WORLD_SIZE: UVec2 = UVec2::splat(100);

#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize)]
pub struct DeWorld {
    tiles: Vec<Vec<TileIndex>>,
    #[serde(default)] player_spanwers: Vec<PlayerSpawnerInWorld>,
    #[serde(default)] word_tags: Vec<WordTagInWorld>,
    #[serde(default)] lock_zones: Vec<LockZoneInWorld>,
    #[serde(default)] fans: Vec<FanInWorld>,
}

impl Default for DeWorld {
    fn default() -> Self {
        Self {
            tiles: vec![vec![TileIndex::Air; WORLD_SIZE.x as usize]; WORLD_SIZE.y as usize],
            player_spanwers: default(),
            word_tags: default(),
            lock_zones: default(),
            fans: default(),
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
        _load_context: &'a mut LoadContext,
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
    tiles: Vec<Vec<TileIndex>>,
}

fn setup_tilemap(
    assets: Res<MiscAssets>,
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

    commands.spawn((
        tilemap_bundle, 
        LoadedWorld { handle: world, tiles: default() },
        Name::new("World"),
    ));
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct TileMapQuery {
    tilemap: &'static mut TileMap,
    loaded_world: &'static mut LoadedWorld,
    entity: Entity,
}

fn spawn_world_on_load(
    mut tilemaps: Query<TileMapQuery>,
    mut world_assets: ResMut<Assets<DeWorld>>,
    de_assets: Res<MiscAssets>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut asset_events: EventReader<AssetEvent<DeWorld>>,
) {
    for asset_event in asset_events.read() {
        let (AssetEvent::Modified { id } | AssetEvent::Added { id }) = *asset_event 
            else { continue };

        let mut tilemap = tilemaps
            .iter_mut()
            .find(|tilemap| tilemap.loaded_world.handle.id() == id)
            .unwrap();

        let Some(state) = asset_server.get_load_state(tilemap.loaded_world.handle.id()) 
            else { continue };

        // in debug builds, create a new world file if one doesn't load.
        if state == LoadState::Failed || state == LoadState::Loaded {
            let world_asset_id = tilemap.loaded_world.handle.id(); 

            #[cfg(debug_assertions)]
            if state == LoadState::Failed && world_assets.get(world_asset_id).is_none() {
                world_assets.insert(world_asset_id, DeWorld::default());
            }

            commands.entity(tilemap.entity).despawn_descendants();

            let world = world_assets.get(world_asset_id).unwrap();
            load_world(world, &*de_assets, &mut tilemap, &mut commands);
        }
    }
}

fn load_world(
    world: &DeWorld,
    assets: &MiscAssets,
    tilemap: &mut TileMapQueryItem,
    commands: &mut Commands,
) {
    tilemap.tilemap.clear();
    commands.entity(tilemap.entity).despawn_descendants();

    tilemap.loaded_world.tiles = world.tiles.clone();

    for x in 0..(WORLD_SIZE.x as usize) {
        for y in 0..(WORLD_SIZE.y as usize) {
            let tile = world.tiles[y][x];
            let position = IVec3::new(x as i32, y as i32, 0);

            if tile == TileIndex::Air {  
                // tile is already none, we don't have to set it
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

    for word_tag in &world.word_tags {
        commands.spawn(WordTag::bundle(word_tag)).set_parent(tilemap.entity);
    }
    for lock_zone in &world.lock_zones {
        commands.spawn(LockZone::bundle(&(&lock_zone, assets))).set_parent(tilemap.entity);
    }
    for spawner in &world.player_spanwers {
        commands.spawn(PlayerSpawner::bundle(&spawner)).set_parent(tilemap.entity);
    }
    for fan in &world.fans {
        commands.spawn(Fan::bundle(&(&fan, assets))).set_parent(tilemap.entity);
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
    asset_server: Res<AssetServer>,
    keyboard: Res<Input<KeyCode>>,
    worlds: Query<(&LoadedWorld, Entity)>,
    children_query: Query<&Children>,
    word_tags: Query<(&WordTag, &Transform)>,
    lock_zones: Query<&Transform, With<LockZone>>,
    spawners: Query<&Transform, With<PlayerSpawner>>,
    fans: Query<(&Fan, &Transform)>,
) {
    use std::path::*;
    use std::fs::*;

    if !(keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::S)) { return }

    for world in &worlds {
        let mut world_to_save = DeWorld::default();
        world_to_save.tiles = world.0.tiles.clone();

        for child in children_query.iter_descendants(world.1) {
            if let Ok(word_tag) = word_tags.get(child) {
                world_to_save.word_tags.push(WordTagInWorld {
                    word_id: word_tag.0.word_id,
                    transform: *word_tag.1,
                });
            } else if let Ok(lock_zone) = lock_zones.get(child) {
                world_to_save.lock_zones.push(LockZoneInWorld {
                    transform: *lock_zone,
                });
            } else if let Ok(spawner) = spawners.get(child) {
                world_to_save.player_spanwers.push(PlayerSpawnerInWorld {
                    transform: *spawner,
                });
            } else if let Ok(fan) = fans.get(child) {
                world_to_save.fans.push(FanInWorld {
                    strength: fan.0.strength,
                    transform: *fan.1,
                });
            }

        }

        let world_path = asset_server.get_path(world.0.handle.id()).unwrap();
        let file_path = Path::new("./assets").join(world_path.path());

        info!("saving world asset {file_path:?}");

        let config = PrettyConfig::new();
        let serialized_world = ron::ser::to_string_pretty(&world_to_save, config)
            .expect("unable to serialize world");
        write(file_path, &*serialized_world).expect("unable to write {file_path:?}");
    }
}

pub trait WorldObject: Component {
    type Bundle: Bundle;
    type InWorld<'a>;

    fn bundle<'a>(in_world: &Self::InWorld<'a>) -> Self::Bundle;
}
