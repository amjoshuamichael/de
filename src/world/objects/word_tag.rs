use crate::{prelude::*, word::{movement::Player, WordID}};
use bevy_rapier2d::prelude::Collider;

use crate::word::ui::VocabChange;

use super::WorldObject;

#[derive(Default, Component)]
pub struct WordTag {
    pub word_id: WordID,
}

#[derive(Default, Bundle)]
pub struct WordTagBundle {
    word_tag: WordTag,
    sprite: SpriteBundle,
    collider: Collider,
    colliding: CollidingEntities,
    rigidbody: RigidBody,
    events: ActiveEvents,
    sensor: Sensor,
    name: Name,
}

#[derive(Debug, TypePath, Serialize, Deserialize)]
pub struct WordTagInWorld {
    pub word_id: WordID,
    pub transform: Transform,
}

impl WorldObject for WordTag {
    type Bundle = WordTagBundle;
    type InWorld = WordTagInWorld;

    fn bundle(in_world: &WordTagInWorld, assets: &MiscAssets) -> WordTagBundle {
        WordTagBundle {
            word_tag: WordTag { word_id: in_world.word_id },
            sprite: SpriteBundle {
                transform: in_world.transform,
                texture: assets.word_tag_sprites[&in_world.word_id].clone(),
                ..default()
            },
            rigidbody: RigidBody::Fixed,
            events: ActiveEvents::all(),
            collider: Collider::cuboid(32.0, 8.0),
            name: Name::from(format!("{} Tag", in_world.word_id.forms().basic)),
            ..default()
        }
    }
}

pub fn word_tags_update(
    mut word_tags: Query<(&WordTag, &CollidingEntities, &mut Visibility, Entity)>,
    parents: Query<&Parent>,
    players: Query<Entity, With<Player>>,
    mut vocab_changes: EventWriter<VocabChange>,
) {
    let player = players.single();

    for mut tag in &mut word_tags {
        if *tag.2 == Visibility::Hidden { continue }

        let word = tag.0.word_id;

        for colliding_obj in tag.1.iter() {
            for parent in parents.iter_ancestors(colliding_obj) {
                if parent == player {
                    vocab_changes.send(VocabChange::Added { word, to: player });
                    *tag.2 = Visibility::Hidden;
                }
            }
        }
    }
}
