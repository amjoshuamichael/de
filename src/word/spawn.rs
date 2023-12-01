use bevy::ecs::system::EntityCommands;

use crate::prelude::*;
use super::*;

enum SentenceParseError {
    /// TODO: actually report errors
    Other,
}

#[derive(Bundle, Clone, Default)]
pub struct WordObjectBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

pub fn remake_player_character(
    mut structure_change_evt: EventReader<SentenceStructureChanged>,
    mut sentences: Query<(&mut SentenceStructure, Entity)>,
    mut all_sprites: Query<&mut Sprite>,
    mut commands: Commands,
    assets: Res<MiscAssets>,
) {
    for change in structure_change_evt.read() {
        let mut sentence = sentences.get_mut(change.on).unwrap();
        
        match spawn_with_noun(sentence.0.root, &sentence.0, &*assets, sentence.1) {
            Ok(commmand_closure) => {
                sentence.0.valid = true;
                commands.entity(sentence.1).despawn_descendants();
                commmand_closure(&mut commands);
                all_sprites.for_each_mut(|mut sprite| sprite.color = Color::WHITE);
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

fn spawn_with_noun(
    word: PhraseID,
    sentence: &SentenceStructure,
    assets: &MiscAssets,
    parent: Entity,
) -> Result<Box<dyn FnOnce(&mut Commands)>, SentenceParseError> {
    match &sentence.sentence[sentence.root] {
        PhraseData { word: None, .. } => return Err(SentenceParseError::Other),
        PhraseData { word: Some(word), kind: PhraseKind::Noun { adjective }} => {
            let mut bundle = WordObjectBundle::default();

            let modifier = modify_with_adjective(*adjective, &sentence, &*assets)?;

            match word {
                WordID::Baby => {
                    bundle.texture = assets.square_pale.clone();
                    let collider = (Collider::cuboid(8.0, 8.0), CollidingEntities::default());
                    Ok(Box::new(move |commands| {
                        let mut new_entity = commands
                            .spawn((bundle, collider, Name::new("Baby")));

                        new_entity.set_parent(parent);

                        modifier(&mut new_entity);
                    }))
                },
                _ => return Err(SentenceParseError::Other),
            }
        }
        _ => return Err(SentenceParseError::Other),
    }
}

fn modify_with_adjective(
    word: PhraseID,
    sentence: &SentenceStructure,
    _assets: &MiscAssets,
) -> Result<Box<dyn FnOnce(&mut EntityCommands)>, SentenceParseError> {
    match &sentence.sentence[word] {
        PhraseData { word: None, .. } => { Ok(Box::new(|_|{})) },
        PhraseData { word: Some(word), kind: PhraseKind::Adjective } => {
            match word {
                WordID::Wide => 
                    { Ok(Box::new(|entity| { entity.insert(WideMark); })) },
                WordID::Tall => 
                    { Ok(Box::new(|entity| { entity.insert(TallMark); })) },
                _ => todo!(),
            }
        }
        _ => return Err(SentenceParseError::Other),
    }
}

pub fn deactivate_inactive_sentence_structures(
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

