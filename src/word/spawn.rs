use bevy::ecs::system::{EntityCommands, RunSystemOnce, EntityCommand};

use crate::prelude::*;
use super::*;

enum SentenceParseError {
    /// TODO: actually report errors
    Other,
}

#[derive(Debug, Component, Clone)]
pub struct WordObject {
    pub sentence: Entity,
    pub noun_word: WordID,
    pub adjectives: AdjectiveStates,
}

#[derive(Default, Debug, Clone)]
pub struct AdjectiveStates {
    pub wide: bool,
    pub tall: bool,
    pub fast: bool,
    pub fluttering: Option<FlutteringDirection>,
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
    children: Query<&Children>,
    word_objects: Query<&WordObject>,
) {
    for change in structure_change_evt.read() {
        let mut sentence = sentences.get_mut(change.on).unwrap();
        let sentence_ptr = (&*sentence.0, sentence.1);
        
        match spawn_with_noun(sentence.0.root, sentence_ptr, &*assets, sentence.1,
          &children, &word_objects) {
            Ok(SpawnResult { command_closure, used_existing_entities }) => {
                sentence.0.valid = true;

                for child in children.iter_descendants(sentence.1) {
                    if !used_existing_entities.contains(&child) {
                        commands.entity(child).despawn_recursive();
                    }
                }

                command_closure(&mut commands);

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

#[derive(Copy, Debug, Clone)]
pub enum FlutteringDirection { Up, Down, Left, Right }

struct SpawnResult {
    command_closure: Box<dyn FnOnce(&mut Commands)>,
    used_existing_entities: HashSet<Entity>,
}

fn spawn_with_noun(
    word: PhraseID,
    sentence: (&SentenceStructure, Entity),
    assets: &MiscAssets,
    parent: Entity,
    children: &Query<&Children>,
    word_objects: &Query<&WordObject>,
) -> Result<SpawnResult, SentenceParseError> {
    match sentence.0.sentence[word] {
        PhraseData { word: None, .. } => return Err(SentenceParseError::Other),
        PhraseData { word: Some(noun_word), kind: PhraseKind::Noun { adjective }, .. } => {
            let existing_noun = find_preexisting_noun(parent, noun_word, 
                                    children, word_objects);
            
            let mut adjective_states = AdjectiveStates::default();
            modify_with_adjective(adjective, sentence, &*assets, &mut adjective_states)?;

            let word_object = WordObject { 
                sentence: sentence.1, 
                noun_word, 
                adjectives: adjective_states,
            };

            let mut used_existing_entities = HashSet::new();
            if let Some(existing_noun) = existing_noun { 
                used_existing_entities.insert(existing_noun);
            }

            let command_closure: Box<dyn FnOnce(&mut Commands)> = match noun_word {
                WordID::Baby => {
                    let square_pale = assets.square_pale.clone();
                    Box::new(move |commands| {
                        let mut new = if let Some(existing) = existing_noun {
                            commands.entity(existing)
                        } else {
                            commands.spawn((
                                WordObjectBundle { texture: square_pale, ..default() },
                                (
                                    Collider::cuboid(8.0, 8.0), 
                                    CollidingEntities::default(),
                                    ColliderMassProperties::Mass(180.),
                                    ActiveEvents::all(),
                                ),
                                Name::new("Baby"),
                            ))
                        };

                        new.insert(word_object).set_parent(parent);
                    })
                },
                WordID::Horse => {
                    let horse_asset = assets.horse.clone();
                    Box::new(move |commands| {
                        let mut new = if let Some(existing) = existing_noun {
                            commands.entity(existing)
                        } else {
                            commands.spawn((
                                WordObjectBundle { texture: horse_asset, ..default() },
                                (
                                    Collider::cuboid(32.0, 8.0), 
                                    CollidingEntities::default(),
                                    ColliderMassProperties::Density(0.5),
                                    ActiveEvents::all(),
                                ),
                                Name::new("Horse"),
                            ))
                        };

                        new.insert(word_object).set_parent(parent);
                    })
                }
                _ => return Err(SentenceParseError::Other),
            };

            Ok(SpawnResult { command_closure, used_existing_entities })
        }
        _ => return Err(SentenceParseError::Other),
    }
}

fn find_preexisting_noun(
    parent: Entity,
    word: WordID,
    children: &Query<&Children>,
    word_objects: &Query<&WordObject>,
) -> Option<Entity> {
    let Ok(children) = children.get(parent) else { return None };
    
    children.iter().find(|child| {
        let Ok(word_object) = word_objects.get(**child).cloned() else { return false };
        return word_object.noun_word == word
    }).copied()

}

fn modify_with_adjective(
    word: PhraseID,
    sentence: (&SentenceStructure, Entity),
    assets: &MiscAssets,
    adjective_states: &mut AdjectiveStates,
) -> Result<(), SentenceParseError> {
    match sentence.0.sentence[word] {
        PhraseData { word: None, .. } => { },
        PhraseData { word: Some(word), kind: PhraseKind::Adjective, .. } => {
            match word {
                WordID::Wide => adjective_states.wide = true,
                WordID::Tall => adjective_states.tall = true,
                WordID::Fast => adjective_states.fast = true,
                WordID::FlutteringUp => 
                    adjective_states.fluttering = Some(FlutteringDirection::Up),
                WordID::FlutteringRight =>
                    adjective_states.fluttering = Some(FlutteringDirection::Right),
                _ => return Err(SentenceParseError::Other),
            }
        }
        PhraseData { word: Some(WordID::And), kind: PhraseKind::Combine { l, r }, .. } => {
            modify_with_adjective(l, sentence, assets, adjective_states)?;
            modify_with_adjective(r, sentence, assets, adjective_states)?;
        }
        _ => return Err(SentenceParseError::Other),
    }

    Ok(())
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
