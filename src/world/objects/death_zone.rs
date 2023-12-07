use crate::{prelude::*, word::{SentenceStructure, ui::SentenceSection, movement::Player, spawn::WordObject}};

use super::{WorldObject, player_spawner::PlayerSpawner};

#[derive(Default, Component)]
pub struct DeathZone;

#[derive(Default, Bundle)]
pub struct DeathZoneBundle {
    word_tag: DeathZone,
    spatial: SpatialBundle,
    collider: Collider,
    colliding: CollidingEntities,
    rigidbody: RigidBody,
    events: ActiveEvents,
    sensor: Sensor,
    name: Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeathZoneInWorld {
    pub transform: Transform,
}

impl WorldObject for DeathZone {
    type Bundle = DeathZoneBundle;
    type InWorld = DeathZoneInWorld;

    fn bundle(in_world: &DeathZoneInWorld, _: &MiscAssets) -> Self::Bundle {
        DeathZoneBundle {
            spatial: SpatialBundle::from_transform(in_world.transform),
            collider: Collider::cuboid(8., 8.),
            rigidbody: RigidBody::Fixed,
            events: ActiveEvents::all(),
            name: Name::new("Death Zone"),
            ..default()
        }
    }
}

pub fn update(
    zones: Query<&CollidingEntities, (Changed<CollidingEntities>, With<DeathZone>)>,
    mut word_objects_and_player: ParamSet<(
        Query<&mut Transform, With<WordObject>>,
        Query<&mut Transform, With<Player>>,
    )>,
    spawners: Query<&Transform, (With<PlayerSpawner>, Without<WordObject>, Without<Player>)>,
) {
    for zone in &zones {
        for colliding in zone.iter() {
            if word_objects_and_player.p0().get_mut(colliding).is_ok() {
                let spawner = spawners.iter().next().unwrap();
                *word_objects_and_player.p1().single_mut() = *spawner;
            }
        }
    }
}
