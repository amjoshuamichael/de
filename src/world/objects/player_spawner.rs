use crate::{prelude::*, word::movement::Player};

use super::WorldObject;

#[derive(Component, Default)]
pub struct PlayerSpawner;

#[derive(Bundle, Default)]
pub struct PlayerSpawnerBundle {
    spawner: PlayerSpawner,
    transform: Transform,
    name: Name,
}

#[derive(Component, Default, Debug, Serialize, Deserialize)]
pub struct PlayerSpawnerInWorld {
    pub transform: Transform,
}

impl WorldObject for PlayerSpawner {
    type Bundle = PlayerSpawnerBundle;
    type InWorld = PlayerSpawnerInWorld;

    fn bundle(in_world: &PlayerSpawnerInWorld, _: &MiscAssets) -> Self::Bundle {
        PlayerSpawnerBundle {
            transform: in_world.transform,
            name: Name::new("spawner"),
            ..default()
        }
    }
}

pub fn update(
    spawners: Query<&Transform, (With<PlayerSpawner>, Changed<Transform>)>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlayerSpawner>)>,
    mut has_spawned: Local<bool>,
) {
    if spawners.is_empty() || *has_spawned { return }

    let spawner_transform = spawners.iter().next().unwrap();
    let mut player_transform = player.single_mut();

    *player_transform = *spawner_transform;
    *has_spawned = true;
}
