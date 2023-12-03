use crate::{prelude::*, word::{*, spawn::FlutteringMark}};

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
    type InWorld<'a> = (&'a FanInWorld, &'a MiscAssets);

    fn bundle<'a>(in_world: &Self::InWorld<'a>) -> Self::Bundle {
        FanBundle {
            fan: Fan { strength: in_world.0.strength },
            sprite: SpriteBundle { 
                transform: Transform {
                    translation: Vec3 {
                        z: -2.0,
                        ..in_world.0.transform.translation
                    },
                    ..in_world.0.transform
                },
                texture: in_world.1.square_pink.clone(),
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 0.2),
                    ..default()
                },
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

pub fn fans_update(
    fans: Query<(&Fan, &CollidingEntities, &Transform)>,
    parents: Query<&Parent>,
    mut sentences: Query<(&mut SentenceStructure, Entity)>,
    currently_fluttering: Query<Entity, With<FlutteringMark>>,
    mut structure_changes: EventWriter<SentenceStructureChanged>,
) {
    let mut all_colliding = HashSet::<Entity>::new();

    for fan in &fans {
        for colliding in fan.1.iter() {
            all_colliding.insert(colliding);
            if currently_fluttering.contains(colliding) { continue; }

            for ancestor in parents.iter_ancestors(colliding) {
                if let Ok(mut sentence) = sentences.get_mut(ancestor) {
                    if !sentence.0.valid { continue }

                    let root = sentence.0.root;
                    let adjective_id = id_of_adjective(&mut *sentence.0, root,
                        &split_adjectives).unwrap();

                    let dir = match fan.2.rotation.z {
                        z if z > -0.35 && z < 0.35 => WordID::FlutteringUp,
                        z if z < -0.35 && z > -1.05 => WordID::FlutteringRight,
                        _ => panic!(),
                    };

                    sentence.0.sentence[adjective_id].word = Some(dir);

                    structure_changes.send(SentenceStructureChanged {
                        on: sentence.1,
                    });

                    break;
                }
            }
        }
    }

    for object in &currently_fluttering {
        if !all_colliding.contains(&object) {
            for ancestor in parents.iter_ancestors(object) {
                if let Ok(mut sentence) = sentences.get_mut(ancestor) {
                    if !sentence.0.valid { continue }

                    let root = sentence.0.root;

                    id_of_adjective(
                        &mut *sentence.0, root,
                        &|id, sentence| {
                            if let PhraseKind::CombineAdjectives { l, r } = 
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
                                    kind: PhraseKind::Adjective
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
        PhraseData { kind: PhraseKind::CombineAdjectives { l, r }, .. } => {
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
            PhraseData::kind(PhraseKind::Adjective));
        let r = sentence.sentence.insert(PhraseData {
            word: phrase_data.word,
            kind: PhraseKind::Adjective,
        });
        sentence.sentence[phrase_id] = PhraseData {
            word: Some(WordID::And),
            kind: PhraseKind::CombineAdjectives { l, r },
        };
        Some(l)
    } else {
        None
    }
}
