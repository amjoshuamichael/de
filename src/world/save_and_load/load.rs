use crate::world::objects::*;
use crate::world::*;

pub struct LevelLoader;
impl AssetLoader for LevelLoader {
    type Asset = DeLevel;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &self,
        reader: &'a mut Reader,
        _: &Self::Settings,
        _: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<DeLevel, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.expect("unable to read world file");

            let mut world = ron::de::from_bytes::<DeLevel>(&bytes)
                .expect("unable to read world file");

            if world.tiles.rows() == 0 || world.tiles.cols() == 0 {
                world.tiles = Grid::new(2, 2);
            }
            
            Ok(world)
        })
    }

    fn extensions(&self) -> &[&str] { &["level.ron"] }
}

pub struct WorldLoader;
impl AssetLoader for WorldLoader {
    type Asset = DeWorld;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &self, 
        reader: &'a mut Reader, 
        _: &Self::Settings, 
        ctx: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<DeWorld, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await.expect("unable to read world file");

            let world = ron::de::from_bytes::<DeWorld>(&bytes).unwrap_or_else(|err| {
                info!("error loading world: {err:?}");
                default()
            });

            Ok(world)
        })
    }

    fn extensions(&self) -> &[&str] { &["world.ron"] }
}

pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    array_texture_loader: Res<ArrayTextureLoader>,
) {
    let world = asset_server.load::<DeWorld>(LOAD_AT_START);
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };

    array_texture_loader.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(asset_server.load("tileset.bmp")),
        tile_size,
        ..Default::default()
    });

    commands.spawn((
        LoadedWorld { handle: world, ..default() },
        Name::new("World"),
    ));
}

pub fn spawn_world_on_load(
    mut world_assets: ResMut<Assets<DeWorld>>,
    mut worlds: Query<(&mut LoadedWorld, Entity)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_events: EventReader<AssetEvent<DeWorld>>,
) {
     for asset_event in asset_events.read() {
        let (AssetEvent::Modified { id } | AssetEvent::Added { id }) = *asset_event 
            else { continue };
        
        let mut world_object = worlds.iter_mut().find(|world| world.0.handle.id() == id).unwrap();

        let Some(state) = asset_server.get_load_state(world_object.0.handle.id()) 
            else { continue };

        if state == LoadState::Failed || state == LoadState::Loaded {
            let level_asset_id = world_object.0.handle.id(); 

            #[cfg(debug_assertions)]
            if state == LoadState::Failed && world_assets.get(level_asset_id).is_none() {
                world_assets.insert(level_asset_id, DeWorld::default());
            }

            commands.entity(world_object.1).despawn_descendants();

            let world = world_assets.get(level_asset_id).unwrap();

            world_object.0.levels = world.levels.clone();

            let tile_size = TilemapTileSize { x: 16., y: 16. };

            for (position, path) in &world.levels {
                commands.spawn((
                    TilemapBundle {
                        tile_size,
                        grid_size: tile_size.into(),
                        texture: TilemapTexture::Single(asset_server.load("tileset.bmp")),
                        transform: Transform::from_translation(*position),
                        ..Default::default()
                    },
                    LoadedLevel { 
                        handle: asset_server.load(path.clone()),
                        tiles: Grid::new(0, 0),
                    },
                    Name::new(format!("Level {path:?}")),
                ));
            }
        }
     }
}

pub fn spawn_level_on_load(
    mut tilemaps: Query<LevelQuery>,
    mut level_assets: ResMut<Assets<DeLevel>>,
    de_assets: Res<MiscAssets>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut asset_events: EventReader<AssetEvent<DeLevel>>,
    mut unspawned: Local<Vec<AssetId<DeLevel>>>,
) {
    let mut unspawned_consume = std::mem::replace(&mut *unspawned, Vec::new());
    let new_asset_ids = asset_events.read()
        .filter_map(|asset_event| {
            let (AssetEvent::Modified { id } | AssetEvent::Added { id }) = *asset_event 
                else { return None };
            Some(id)
        })
        .chain(unspawned_consume.drain(..));
        
    for asset_id in new_asset_ids {
        let Some(mut tilemap) = 
            tilemaps.iter_mut().find(|l| l.loaded_level.handle.id() == asset_id)
            else {
                info!("{:?}", &unspawned);
                unspawned.push(asset_id);
                continue;
            };


        let Some(state) = asset_server.get_load_state(tilemap.loaded_level.handle.id()) 
            else { continue };

        if state == LoadState::Failed || state == LoadState::Loaded {
            let level_asset_id = tilemap.loaded_level.handle.id(); 

            #[cfg(debug_assertions)]
            if state == LoadState::Failed && level_assets.get(level_asset_id).is_none() {
                level_assets.insert(level_asset_id, DeLevel::default());
            }

            commands.entity(tilemap.entity).despawn_descendants();

            let level = level_assets.get(level_asset_id).unwrap();
            spawn_level(level, &*de_assets, &mut tilemap, &mut commands);
        }
    }
}

fn spawn_level(
    world: &DeLevel,
    assets: &MiscAssets,
    tilemap: &mut LevelQueryItem,
    commands: &mut Commands,
) {
    *tilemap.size = TilemapSize { x: world.tiles.cols() as u32, y: world.tiles.rows() as u32};
    *tilemap.storage = TileStorage::empty(*tilemap.size);

    tilemap.loaded_level.tiles = world.tiles.clone();

    for word_tag in &world.word_tags {
        commands.spawn(WordTag::bundle(word_tag, &assets)).set_parent(tilemap.entity);
    }
    for lock_zone in &world.lock_zones {
        commands.spawn(LockZone::bundle(lock_zone, &assets)).set_parent(tilemap.entity);
    }
    for spawner in &world.player_spanwers {
        commands.spawn(PlayerSpawner::bundle(&spawner, &assets)).set_parent(tilemap.entity);
    }
    for fan in &world.fans {
        commands.spawn(Fan::bundle(fan, &assets)).set_parent(tilemap.entity);
    }
    for death_zone in &world.death_zones {
        commands.spawn(DeathZone::bundle(death_zone, &assets)).set_parent(tilemap.entity);
    }
}
