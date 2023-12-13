const LOAD_AT_START: &'static str = "jungle.world.ron";
//"jungle2.world.ron";
//"test.world.ron";

use std::path::PathBuf;

use crate::{prelude::*, word::WordID};
use bevy::{asset::{*, io::*}, app::AppExit, window::exit_on_all_closed};
use bevy_ecs_tilemap::prelude::*;
use ron::ser::PrettyConfig;

use self::editor::{WorldEditorState, WorldEditorPlugin};

mod editor;
mod objects;
mod save_and_load;
pub mod helpers;

use objects::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, save_and_load::setup_world)
            .add_systems(Update, (
                save_and_load::save_world,
                save_and_load::spawn_world_on_load.pipe(save_and_load::spawn_level_on_load),
            ))
            .add_state::<WorldEditorState>()
            .add_systems(Update, (
                word_tag::update,
                lock_zone::update,
                player_spawner::update,
                fan::update.before(SentenceModificationRoutine),
                death_zone::update,
            ))
            .add_plugins(TilemapPlugin)
            .add_plugins(WorldEditorPlugin)
            .init_resource::<editor::MouseWorldCoords>()
            .enable_functions::<LoadedLevel>()
            .enable_functions::<LoadedWorld>()
            .init_asset::<DeLevel>()
            .init_asset::<DeWorld>()
            .register_asset_loader(save_and_load::LevelLoader)
            .register_asset_loader(save_and_load::WorldLoader);
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TileIndex {
    #[serde(rename = "G")]
    Ground = 0,
    #[default]
    #[serde(rename = "A")]
    Air = u32::MAX,
}

#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize)]
pub struct DeLevel {
    #[serde(default = "empty_grid")]
    tiles: Grid<TileIndex>,
    #[serde(default)] player_spanwers: Vec<PlayerSpawnerInWorld>,
    #[serde(default)] word_tags: Vec<WordTagInWorld>,
    #[serde(default)] lock_zones: Vec<LockZoneInWorld>,
    #[serde(default)] fans: Vec<FanInWorld>,
    #[serde(default)] death_zones: Vec<DeathZoneInWorld>,
}

impl Default for DeLevel {
    fn default() -> Self {
        Self {
            tiles: Grid::new(100, 100),
            player_spanwers: default(),
            word_tags: default(),
            lock_zones: default(),
            fans: default(),
            death_zones: default(),
        }
    }
}


#[derive(Debug, Resource, Asset, TypePath, Serialize, Deserialize, Default)]
pub struct DeWorld {
    levels: Vec<(Vec3, PathBuf)>,
}

#[derive(Component)]
pub struct LoadedLevel {
    pub handle: Handle<DeLevel>,
    pub tiles: Grid<TileIndex>,
}

impl GrayboxFunctions for LoadedLevel {
    fn functions() -> Vec<(&'static str, fn(&mut Self))> {
        use TileIndex::Ground as G;
        fn atb(s: &mut LoadedLevel) { 
            s.tiles.push_col(vec![G; s.tiles.rows()]);
            info!("new count: {}", s.tiles.cols());
        }
        fn atr(s: &mut LoadedLevel) { 
            s.tiles.push_row(vec![G; s.tiles.cols()]);
            info!("new count: {}", s.tiles.rows());
        }
        fn rfb(s: &mut LoadedLevel) { 
            s.tiles.pop_col();
            info!("new count: {}", s.tiles.cols());
        }
        fn rfr(s: &mut LoadedLevel) { 
            s.tiles.pop_row();
            info!("new count: {}", s.tiles.rows());
        }

        vec![
            ("add row to bottom", atb),
            ("add col to right", atr),
            ("remove row from bottom", rfb),
            ("remove col add right", rfr),
        ]
    }
}

#[derive(Default, Component)]
pub struct LoadedWorld {
    handle: Handle<DeWorld>,
    levels: Vec<(Vec3, PathBuf)>,
}

impl GrayboxFunctions for LoadedWorld {
    fn functions() -> Vec<(&'static str, fn(&mut Self))> {
        fn add_level(world: &mut LoadedWorld) {
            world.levels.push((Vec3::ZERO, "test.level.ron".into()));
        }

        vec![
            ("add level", add_level)
        ]
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct LevelQuery {
    storage: &'static mut TileStorage,
    size: &'static mut TilemapSize,
    loaded_level: &'static mut LoadedLevel,
    entity: Entity,
}

#[derive(Default, Bundle)]
struct WorldCollderBundle {
    global: GlobalTransform,
    transform: Transform,
    rigidbody: RigidBody,
    collider: Collider,
}

fn calculate_world_colliders(tiles: &Grid<TileIndex>) -> Vec<WorldCollderBundle> {
    let mut filled = Grid::<bool>::new(tiles.rows() as usize, tiles.cols() as usize);

    let mut output = Vec::<WorldCollderBundle>::new();

    use TileIndex::Air as Air;

    for start_x in 0..tiles.rows() {
        for start_y in 0..tiles.cols() {
            let mut x = start_x;
            let mut y = start_y;

            if tiles[(x, y)] == Air || filled[(x, y)] { continue }

            while x < tiles.rows() as usize &&
                tiles[(x, y)] != Air && 
                !filled[(x, y)] {
                x += 1;
            }

            while y < tiles.cols() &&
              (start_x..x).all(|x| tiles[(x, y)] != Air) &&
              (start_x..x).all(|x| !filled[(x, y)]) {
                y += 1;
            }

            for fill_x in start_x..x {
                for fill_y in start_y..y {
                    filled[(fill_x, fill_y)] = true;
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

    output
}

