use bevy::ecs::system::{EntityCommands, RunSystemOnce};

use crate::prelude::*;
use super::*;

enum SentenceParseError {
    /// TODO: actually report errors
    Other,
}

#[derive(Debug, Component)]
pub struct WordObject {
    pub sentence: Entity,
}

#[derive(Bundle, Clone, Default)]
pub struct WordObjectBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    texture: Handle<Image>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    impulse: ExternalImpulse,
    force: ExternalForce,
}

#[derive(Event)]
pub struct SentenceSpawn;

pub fn remake_player_character(
    mut structure_change_evt: EventReader<SentenceStructureChanged>,
    mut sentences: Query<(&mut SentenceStructure, Entity)>,
    mut all_sprites: Query<&mut Sprite>,
    mut commands: Commands,
    assets: Res<MiscAssets>,
    mut spawn_events: EventWriter<SentenceSpawn>,
) {
    for change in structure_change_evt.read() {
        let mut sentence = sentences.get_mut(change.on).unwrap();
        let sentence_ptr = (&*sentence.0, sentence.1);
        
        match spawn_with_noun(sentence.0.root, sentence_ptr, &*assets, sentence.1) {
            Ok(commmand_closure) => {
                sentence.0.valid = true;
                commands.entity(sentence.1).despawn_descendants();
                commmand_closure(&mut commands);
                all_sprites.for_each_mut(|mut sprite| sprite.color = Color::WHITE);
                spawn_events.send(SentenceSpawn);
            },
            Err(_) => {
                all_sprites
                    .for_each_mut(|mut sprite| sprite.color = Color::GRAY.with_a(0.2));
                sentence.0.valid = false;
            }
        }
    }
}

#[derive(Component)]
pub struct WideMark;

#[derive(Component)]
pub struct TallMark;

#[derive(Component)]
pub struct FlutteringMark {
    pub direction: FlutteringDirection,
}

pub enum FlutteringDirection { Up, Down, Left, Right }

fn spawn_with_noun(
    word: PhraseID,
    sentence: (&SentenceStructure, Entity),
    assets: &MiscAssets,
    parent: Entity,
) -> Result<Box<dyn FnOnce(&mut Commands)>, SentenceParseError> {
    match &sentence.0.sentence[word] {
        PhraseData { word: None, .. } => return Err(SentenceParseError::Other),
        PhraseData { word: Some(word), kind: PhraseKind::Noun { adjective }} => {
            let mut bundle = WordObjectBundle::default();
            let word_object = WordObject { sentence: sentence.1, };

            let modifier = modify_with_adjective(*adjective, sentence, &*assets)?;

            match word {
                WordID::Baby => {
                    bundle.texture = assets.square_pale.clone();
                    let collider = (
                        Collider::cuboid(8.0, 8.0), 
                        CollidingEntities::default(),
                        ColliderMassProperties::Density(1.2),
                    );
                    Ok(Box::new(move |commands| {
                        let mut new_entity = commands
                            .spawn((bundle, collider, word_object, Name::new("Baby")));

                        new_entity.set_parent(parent);

                        modifier(&mut new_entity);
                    }))
                },
                WordID::Horse => {
                    bundle.texture = assets.horse.clone();
                    let collider = (
                        Collider::cuboid(32.0, 8.0), 
                        CollidingEntities::default(),
                        ColliderMassProperties::Density(0.5),
                    );
                    Ok(Box::new(move |commands| {
                        let mut new_entity = commands
                            .spawn((bundle, collider, word_object, Name::new("Horse")));

                        new_entity.set_parent(parent);

                        modifier(&mut new_entity);
                    }))
                }
                _ => return Err(SentenceParseError::Other),
            }
        }
        _ => return Err(SentenceParseError::Other),
    }
}

fn modify_with_adjective(
    word: PhraseID,
    sentence: (&SentenceStructure, Entity),
    assets: &MiscAssets,
) -> Result<Box<dyn FnOnce(&mut EntityCommands)>, SentenceParseError> {
    match &sentence.0.sentence[word] {
        PhraseData { word: None, .. } => { Ok(Box::new(|_|{})) },
        PhraseData { word: Some(word), kind: PhraseKind::Adjective } => {
            match word {
                WordID::Wide => Ok(Box::new(|entity| { entity.insert(WideMark); })),
                WordID::Tall => Ok(Box::new(|entity| { entity.insert(TallMark); })),
                WordID::FlutteringUp => Ok(Box::new(|entity| { 
                    entity.insert(FlutteringMark { direction: FlutteringDirection::Up });
                })),
                WordID::FlutteringRight => Ok(Box::new(|entity| { 
                    entity.insert(FlutteringMark { direction: FlutteringDirection::Right });
                })),
                _ => return Err(SentenceParseError::Other),
            }
        }
        PhraseData { word: Some(WordID::And), kind: PhraseKind::CombineAdjectives { l, r } } => {
            let l_application = modify_with_adjective(*l, sentence, assets)?;
            let r_application = modify_with_adjective(*r, sentence, assets)?;

            Ok(Box::new(|entity| {
                l_application(entity);
                r_application(entity);
            }))
        }
        _ => return Err(SentenceParseError::Other),
    }
}

pub fn disable_physics_for_invalid_sentence_structures(
    mut sentences: Query<(&SentenceStructure, Entity), Changed<SentenceStructure>>,
    mut commands: Commands,
) {
    for sentence in &mut sentences {
        let mut sentence_object = commands.entity(sentence.1);

        if sentence.0.valid {
            sentence_object.remove::<RigidBodyDisabled>();
        } else {
            sentence_object.insert(RigidBodyDisabled::default());
        }
    }
}
