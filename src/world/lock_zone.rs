use crate::{prelude::*, word::{SentenceStructure, ui::SentenceSection}};

use super::WorldObject;

#[derive(Default, Component)]
pub struct LockZone;

#[derive(Default, Bundle)]
pub struct LockZoneBundle {
    word_tag: LockZone,
    sprite: SpriteBundle,
    collider: Collider,
    colliding: CollidingEntities,
    rigidbody: RigidBody,
    events: ActiveEvents,
    sensor: Sensor,
    name: Name,
}

pub struct LockZoneInWorld {
    transform: Transform,
}

impl WorldObject for LockZone {
    type Bundle = LockZoneBundle;
    type InWorld = LockZoneInWorld;

    fn bundle(in_world: &Self::InWorld) -> Self::Bundle {
        LockZoneBundle {
            sprite: SpriteBundle { transform: in_world.transform, ..default() },
            ..default()
        }
    }
}

pub fn lock_zone_update(
    zones: Query<(&LockZone, &CollidingEntities)>,
    structures: Query<Entity, With<SentenceStructure>>,
    mut sentence_sections: Query<&mut SentenceSection>,
) {
    for zone in &zones {
        for colliding_entity in zone.1.iter() {
            let Ok(sentence) = structures.get(colliding_entity) else { continue };

            for mut section in &mut sentence_sections {
                if section.sentence_entity == sentence {
                    section.locked = true;
                }
            }
        }
    }
}
