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
    type InWorld<'a> = PlayerSpawnerInWorld;

    fn bundle(in_world: &PlayerSpawnerInWorld) -> Self::Bundle {
        PlayerSpawnerBundle {
            transform: in_world.transform,
            name: Name::new("spawner"),
            ..default()
        }
    }
}

pub fn player_spawner_update(
    spawners: Query<&Transform, (With<PlayerSpawner>, Changed<Transform>)>,
    mut player: Query<&mut Transform, (With<Player>, Without<PlayerSpawner>)>,
) {
    if spawners.is_empty() { return }

    let spawner_transform = spawners.iter().next().unwrap();
    let mut player_transform = player.single_mut();

    *player_transform = *spawner_transform;
}
