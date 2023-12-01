use crate::{prelude::*, word::{movement::Player, WordID}};
use bevy_rapier2d::prelude::Collider;

use crate::word::{Words, ui::VocabChange};

use super::WorldObject;

#[derive(Default, Component)]
pub struct WordTag {
    pub(super) word_id: WordID,
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
    pub(super) word_id: WordID,
    pub(super) transform: Transform,
}

impl WorldObject for WordTag {
    type Bundle = WordTagBundle;
    type InWorld = WordTagInWorld;

    fn bundle(in_world: &WordTagInWorld) -> WordTagBundle {
        WordTagBundle {
            word_tag: WordTag { word_id: in_world.word_id },
                sprite: SpriteBundle {
                    transform: in_world.transform,
                    ..default()
                },
                rigidbody: RigidBody::Fixed,
                events: ActiveEvents::all(),
            ..default()
        }
    }
}

pub fn init_word_tags(
    mut new_tags: Query<
        (&WordTag, &Transform, &mut Handle<Image>, &mut Collider, &mut Name),
        Changed<WordTag>
    >, 
    words: Res<Words>
) {
    for mut tag in &mut new_tags {
        let word_data = &words.0[&tag.0.word_id];
        *tag.2 = word_data.tag_handle.clone();
        *tag.3 = Collider::cuboid(8.0, 8.0);
        tag.4.set(word_data.basic.to_string() + "tag");
    }
}

pub fn word_tags_update(
    word_tags: Query<(&WordTag, &CollidingEntities, Entity)>,
    parents: Query<&Parent>,
    players: Query<Entity, With<Player>>,
    mut vocab_changes: EventWriter<VocabChange>,
    mut commands: Commands,
) {
    let player = players.single();

    for tag in &word_tags {
        let word = tag.0.word_id;

        for colliding_obj in tag.1.iter() {
            for parent in parents.iter_ancestors(colliding_obj) {
                if parent == player {
                    vocab_changes.send(VocabChange::Added { word, to: player });
                    commands.entity(tag.2).despawn_recursive();
                }
            }
        }
    }
}
