use crate::prelude::*;
use crate::world::*;

pub fn save_world(
    asset_server: Res<AssetServer>,
    keyboard: Res<Input<KeyCode>>,
    levels: Query<(&LoadedLevel, Entity)>,
    worlds: Query<&LoadedWorld>,
    children_query: Query<&Children>,
    word_tags: Query<(&WordTag, &Transform)>,
    lock_zones: Query<&Transform, With<LockZone>>,
    spawners: Query<&Transform, With<PlayerSpawner>>,
    fans: Query<(&Fan, &Transform)>,
    death_zones: Query<&Transform, With<DeathZone>>,
) {
    use std::path::*;
    use std::fs::*;

    if !(keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::S)) { return }

    for world in &worlds {
        let world_to_save = DeWorld {
            levels: world.levels.clone(),
        };

        let world_path = asset_server.get_path(world.handle.id()).unwrap();
        let file_path = Path::new("./assets").join(world_path.path());

        info!("saving world asset {file_path:?}");

        let config = PrettyConfig::new();
        let serialized_world = ron::ser::to_string_pretty(&world_to_save, config)
            .expect("unable to serialize world");
        write(file_path, &*serialized_world).expect("unable to write {file_path:?}");

    }

    for level in &levels {
        let mut level_to_save = DeLevel::default();
        level_to_save.tiles = level.0.tiles.clone();

        for child in children_query.iter_descendants(level.1) {
            if let Ok(word_tag) = word_tags.get(child) {
                level_to_save.word_tags.push(WordTagInWorld {
                    word_id: word_tag.0.word_id,
                    transform: *word_tag.1,
                });
            } else if let Ok(lock_zone) = lock_zones.get(child) {
                level_to_save.lock_zones.push(LockZoneInWorld {
                    transform: *lock_zone,
                });
            } else if let Ok(spawner) = spawners.get(child) {
                level_to_save.player_spanwers.push(PlayerSpawnerInWorld {
                    transform: *spawner,
                });
            } else if let Ok(fan) = fans.get(child) {
                level_to_save.fans.push(FanInWorld {
                    strength: fan.0.strength,
                    translation: fan.1.translation.xy(),
                    rotation: fan.1.rotation.to_euler(EulerRot::XYZ).2,
                    scale: fan.1.scale.xy(),
                });
            } else if let Ok(death_zone) = death_zones.get(child) {
                level_to_save.death_zones.push(DeathZoneInWorld {
                    transform: *death_zone,
                });
            }
        }

        let level_path = asset_server.get_path(level.0.handle.id()).unwrap();
        let file_path = Path::new("./assets").join(level_path.path());

        info!("saving level asset {file_path:?}");

        let config = PrettyConfig::new();
        let serialized_level = ron::ser::to_string_pretty(&level_to_save, config)
            .expect("unable to serialize level");
        write(file_path, &*serialized_level).expect("unable to write {file_path:?}");
    }
}
