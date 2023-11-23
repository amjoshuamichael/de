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
    mut commands: Commands,
    assets: Res<DeAssets>,
) {
    for change in structure_change_evt.read() {
        let mut sentence = sentences.get_mut(change.on).unwrap();
        commands.entity(sentence.1).despawn_descendants();
        
        let spawn_result = 
            spawn_with_noun(sentence.0.root, &sentence.0, &mut commands, &*assets, sentence.1);

        if spawn_result.is_ok() {
            sentence.0.active = true;
        }
    }
}

fn spawn_with_noun(
    word: PhraseID,
    sentence: &SentenceStructure,
    commands: &mut Commands,
    assets: &DeAssets,
    parent: Entity,
) -> Result<(), SentenceParseError> {
    match &sentence.sentence[sentence.root] {
        PhraseData { word: None, .. } => return Err(SentenceParseError::Other),
        PhraseData { word: Some(word), kind: PhraseKind::Noun { adjective }} => {
            let mut bundle = WordObjectBundle::default();

            modify_with_adjective(*adjective, &sentence, &mut bundle, &*assets)?;

            match word {
                WordID::Baby => {
                    bundle.texture = assets.square_pale.clone();
                    let collider = Collider::cuboid(8.0, 8.0);
                    commands.spawn((bundle, collider)).set_parent(parent);
                },
                _ => return Err(SentenceParseError::Other),
            }
        }
        _ => return Err(SentenceParseError::Other),
    }

    Ok(())
}

fn modify_with_adjective(
    word: PhraseID,
    sentence: &SentenceStructure,
    bundle: &mut WordObjectBundle,
    assets: &DeAssets,
) -> Result<(), SentenceParseError> {
    match &sentence.sentence[word] {
        PhraseData { word: None, .. } => {},
        PhraseData { word: Some(word), kind: PhraseKind::Adjective } => {
            match word {
                WordID::Wide => 
                    { bundle.transform.scale.x = 4.0; },
                WordID::Tall => 
                    { bundle.transform.scale.y = 4.0; },
                _ => todo!(),
            }
        }
        _ => return Err(SentenceParseError::Other),
    }

    Ok(())
}


pub fn deactivate_inactive_sentence_structures(
    mut sentences: Query<(&mut SentenceStructure, Option<&mut Velocity>)>,
) {
    for sentence in &mut sentences {
        if !sentence.0.active {
            if let Some(mut vel) = sentence.1 { *vel = Velocity::default() }
        }
    }
}

