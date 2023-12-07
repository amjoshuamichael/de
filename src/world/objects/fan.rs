use crate::{prelude::*, word::{*, apply_words::QWordObject}};

use super::WorldObject;

#[derive(Default, Component)]
pub struct Fan {
    pub strength: f32,
}

#[derive(Default, Bundle)]
pub struct FanBundle {
    fan: Fan,
    sprite: SpriteBundle,
    collider: Collider,
    colliding: CollidingEntities,
    rigidbody: RigidBody,
    events: ActiveEvents,
    sensor: Sensor,
    name: Name,
}

#[derive(Debug, TypePath, Serialize, Deserialize)]
pub struct FanInWorld {
    pub strength: f32,
    pub transform: Transform,
}

impl WorldObject for Fan {
    type Bundle = FanBundle;
    type InWorld = FanInWorld;

    fn bundle(in_world: &FanInWorld, assets: &MiscAssets) -> Self::Bundle {
        FanBundle {
            fan: Fan { strength: in_world.strength },
            sprite: SpriteBundle { 
                transform: Transform {
                    translation: Vec3 {
                        z: -2.0,
                        ..in_world.transform.translation
                    },
                    ..in_world.transform
                },
                texture: assets.square_pink.clone(),
                ..default() 
            },
            collider: Collider::cuboid(8., 8.),
            rigidbody: RigidBody::Fixed,
            events: ActiveEvents::all(),
            name: Name::new("Fan"),
            ..default()
        }
    }
}

pub fn update(
    fans: Query<(&Fan, &CollidingEntities, &Transform)>,
    parents: Query<&Parent>,
    mut sentences: Query<(&mut SentenceStructure, Entity)>,
    word_objects: Query<QWordObject>,
    mut structure_changes: EventWriter<SentenceStructureChanged>,
) {
    let currently_fluttering: HashSet::<Entity> = word_objects.iter()
        .filter(|o| o.words.adjectives.fluttering.is_some())
        .map(|o| o.entity)
        .collect();
    let mut all_colliding = HashSet::<Entity>::new();

    for fan in &fans {
        for colliding in fan.1.iter() {
            all_colliding.insert(colliding);
            if currently_fluttering.contains(&colliding) { continue; }

            for ancestor in parents.iter_ancestors(colliding) {
                if let Ok(mut sentence) = sentences.get_mut(ancestor) {
                    if !sentence.0.valid { continue }

                    let root = sentence.0.root;
                    let adjective_id = id_of_adjective(&mut *sentence.0, root,
                        &split_adjectives).unwrap();

                    let dir = match fan.2.rotation.z {
                        z if z > -0.35 && z < 0.35 => WordID::FlutteringUp,
                        z if z < -0.35 && z > -1.05 => WordID::FlutteringRight,
                        _ => WordID::FlutteringUp,
                    };

                    sentence.0.sentence[adjective_id].word = Some(dir);
                    sentence.0.sentence[adjective_id].locked = true;

                    structure_changes.send(SentenceStructureChanged {
                        on: sentence.1,
                    });

                    break;
                }
            }
        }
    }

    for object in currently_fluttering {
        if !all_colliding.contains(&object) {
            for ancestor in parents.iter_ancestors(object) {
                if let Ok(mut sentence) = sentences.get_mut(ancestor) {
                    if !sentence.0.valid { continue }

                    let root = sentence.0.root;

                    id_of_adjective(
                        &mut *sentence.0, root,
                        &|id, sentence| {
                            if let PhraseKind::Combine { l, r } = 
                                sentence.sentence[id].kind {
                                let search = [
                                    Some(WordID::FlutteringUp), 
                                    Some(WordID::FlutteringRight)
                                ];
                                let word = if search.contains(&sentence.sentence[l].word) {
                                    sentence.sentence[r].word
                                } else if search.contains(&sentence.sentence[r].word) {
                                    sentence.sentence[l].word
                                } else {
                                    return None;
                                };

                                sentence.sentence[id] = PhraseData {
                                    word,
                                    kind: PhraseKind::Adjective,
                                    locked: false,
                                };

                                Some(id)
                            } else {
                                None
                            }
                        }
                    ).unwrap();

                    structure_changes.send(SentenceStructureChanged {
                        on: sentence.1,
                    });

                    break;
                }
            }
        }
    }
}

fn id_of_adjective(
    sentence: &mut SentenceStructure, 
    id: PhraseID, 
    filter: &impl Fn(PhraseID, &mut SentenceStructure) -> Option<PhraseID>,
) -> Option<PhraseID> {
    if let Some(id) = filter(id, sentence) {
        return Some(id);
    }

    match sentence.sentence[id] {
        PhraseData { kind: PhraseKind::Noun { adjective }, .. } => {
            id_of_adjective(sentence, adjective, filter)
        }
        PhraseData { kind: PhraseKind::Combine { l, r }, .. } => {
            id_of_adjective(sentence, l, filter)
                .or_else(|| id_of_adjective(sentence, r, filter))
        }
        _ => None
    }
}

fn split_adjectives(phrase_id: PhraseID, sentence: &mut SentenceStructure) -> Option<PhraseID> {
    let phrase_data = sentence.sentence[phrase_id];
    if phrase_data.kind == PhraseKind::Adjective {
        let l = sentence.sentence.insert(
            PhraseData { kind: PhraseKind::Adjective, ..default() });
        let r = sentence.sentence.insert(PhraseData {
            word: phrase_data.word,
            kind: PhraseKind::Adjective,
            ..default()
        });
        sentence.sentence[phrase_id] = PhraseData {
            word: Some(WordID::And),
            kind: PhraseKind::Combine { l, r },
            locked: true,
        };
        Some(l)
    } else {
        None
    }
}
