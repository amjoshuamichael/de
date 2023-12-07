use crate::{prelude::*, word::{SentenceStructure, ui::SentenceSection, movement::Player, spawn::WordObject}};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct LockZoneInWorld {
    pub transform: Transform,
}

impl WorldObject for LockZone {
    type Bundle = LockZoneBundle;
    type InWorld = LockZoneInWorld;

    fn bundle(in_world: &LockZoneInWorld, assets: &MiscAssets) -> Self::Bundle {
        LockZoneBundle {
            sprite: SpriteBundle { 
                transform: in_world.transform,
                texture: assets.square_yellow.clone(),
                ..default() 
            },
            collider: Collider::cuboid(8., 8.),
            rigidbody: RigidBody::Fixed,
            events: ActiveEvents::all(),
            name: Name::new("Lock Zone"),
            ..default()
        }
    }
}

pub fn update(
    zone_changes: Query<(), (Changed<CollidingEntities>, With<LockZone>)>,
    zones: Query<(&LockZone, &CollidingEntities)>,
    word_objects: Query<&WordObject>,
    mut sentence_sections: Query<&mut SentenceSection>,
) {
    if zone_changes.is_empty() { return };

    for mut section in &mut sentence_sections {
        section.locked = false;
    }

    for zone in &zones {
        for colliding_entity in zone.1.iter() {
            let Ok(word_object) = word_objects.get(colliding_entity) else { continue };

            for mut section in &mut sentence_sections {
                if section.sentence_entity == word_object.sentence {
                    section.locked = true;
                }
            }
        }
    }
}
