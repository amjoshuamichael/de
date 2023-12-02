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
    type InWorld<'a> = (&'a LockZoneInWorld, &'a MiscAssets);

    fn bundle<'a>(in_world: &Self::InWorld<'a>) -> Self::Bundle {
        LockZoneBundle {
            sprite: SpriteBundle { 
                transform: Transform {
                    translation: Vec3 {
                        z: -2.0,
                        ..in_world.0.transform.translation
                    },
                    ..in_world.0.transform
                },
                texture: in_world.1.square_yellow.clone(),
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.2),
                    ..default()
                },
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

pub fn lock_zone_update(
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
