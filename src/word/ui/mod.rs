use crate::prelude::*;

use bevy::ecs::query::{WorldQuery, ReadOnlyWorldQuery};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseMotion;
use bevy::ui::Interaction;
use itertools::PeekNth;

use crate::load_assets::MiscAssets;

use super::*;

mod drag_and_drop;
pub use drag_and_drop::*;

#[derive(Default, Component)]
pub struct DraggableWord {
    word_id: WordID,
    locked: bool,
}

#[derive(Component)]
pub struct SentenceUIParent {
    sentence_entity: Entity,
}

#[derive(Debug, Component)]
pub struct SentenceSection {
    pub for_phrase: PhraseID,
    pub sentence_entity: Entity,
    pub locked: bool,
}

#[derive(Debug, Component)]
pub struct SentenceJoint;

#[derive(Component)]
pub struct Inventory;

#[derive(Copy, Clone, PartialEq, Eq, Component, PartialOrd, Ord, Debug)]
pub enum SentenceUIPart {
    CombineJointL,
    CombineJoint,
    AndSlot,
    CombineJointR,
    AdjectiveSlot,
    NounJoint,
    NounSlot,
}

impl SentenceUIPart {
    fn is_slot(self) -> bool {
        match self {
            CombineJointL | CombineJoint | CombineJointR | NounJoint => false,
            NounSlot | AndSlot | AdjectiveSlot => true,
        }
    }
}

use SentenceUIPart::*;

#[derive(Component)]
pub struct DraggingParent;

const TEXT_OBJECTS_Z_INDEX: ZIndex = ZIndex::Global(30);

#[derive(Default, Bundle)]
pub struct DraggableWordBundle {
    text: TextBundle,
    draggable: DraggableWord,
    border_color: BorderColor,
    interaction: Interaction,
}

impl DraggableWordBundle {
    fn for_word_snapped(word_id: WordID, assets: &MiscAssets) -> Self {
        DraggableWordBundle {
            text: TextBundle {
                text: Text::from_section(
                    word_id.forms().basic,
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 60.0,
                        ..default()
                    }
                ),
                style: Style {
                    position_type: PositionType::Relative,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                background_color: BackgroundColor::from(Color::DARK_GREEN),
                z_index: TEXT_OBJECTS_Z_INDEX,
                ..default()
            },
            draggable: DraggableWord {
                word_id,
                locked: false,
            },
            interaction: Interaction::default(),
            ..default()
        }
    }
}

pub fn setup_word_ui(
    player: In<Entity>,
    mut commands: Commands,
) {
    let _inventory = commands.spawn((
        Inventory {},
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::Grid,
                grid_template_rows: {
                    let mut basic_rows = vec![GridTrack::auto(); 6];
                    basic_rows.push(GridTrack::flex(1.0));
                    basic_rows
                },
                width: Val::Percent(15.),
                height: Val::Percent(100.),
                right: Val::Px(0.),
                row_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        },
    )).id();

    commands.spawn((
        DraggingParent,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                ..default()
            },
            ..default()
        },
        Name::new("Dragging Parent"),
    ));

    let _word_snap_parent = commands.spawn((
        SentenceUIParent {
            sentence_entity: player.0,
        },
        NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Px(100.0),
                column_gap: Val::Px(20.0),
                ..default()
            },
            background_color: Color::RED.with_a(0.2).into(),
            ..default()
        },
    )).id();
}

#[derive(Event)]
pub enum VocabChange {
    Added {
        word: WordID,
        to: Entity,
    },
}

pub fn update_vocabulary(
    mut vocabularies: Query<&mut Vocabulary>,
    inventory: Query<Entity, With<Inventory>>,
    mut vocab_changes: EventReader<VocabChange>,
    mut commands: Commands,
    assets: Res<MiscAssets>,
) {
    let inventory = inventory.single();

    for vocab_change in vocab_changes.read() {
        match vocab_change {
            VocabChange::Added { word, to } => {
                let mut vocabulary = vocabularies.get_mut(*to).unwrap();
                vocabulary.words.insert(*word);
                commands
                    .spawn(DraggableWordBundle::for_word_snapped(*word, &*assets))
                    .set_parent(inventory);
            },
        }
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct QSentenceNode {
    style: &'static mut Style,
    background_color: &'static mut BackgroundColor,
    entity: Entity,
}

#[derive(SystemParam)]
pub struct SpawnSnapParams<'w, 's> {
    commands: Commands<'w, 's>,
    sentence_parts: Query<'w, 's, (&'static SentenceUIPart, Entity)>,
    children: Query<'w, 's, &'static Children>,
    assets: Res<'w, MiscAssets>,
}

pub fn reorder_sentence_ui(
    mut joint_children_sets: Query<&mut Children, (With<SentenceJoint>, Changed<Children>)>,
    parts: Query<&SentenceUIPart>,
) {
    for mut joint_children in &mut joint_children_sets {
        joint_children.sort_by(|a, b| parts.get(*a).unwrap().cmp(parts.get(*b).unwrap()));
    }
}

pub fn indicate_sentence_section_locks(
    sections: Query<(&mut SentenceSection, Option<&Children>), Changed<SentenceSection>>,
    mut words: Query<(&mut DraggableWord, &mut Text)>,
) {
    for (section, children) in &sections {
        let Some(children) = children else { continue };
        let Some(first_child) = children.get(0) else { continue };
        let Ok(mut word) = words.get_mut(*first_child) else { continue };

        word.0.locked = section.locked;
        if section.locked {
            word.1.sections[0].style.color = Color::GRAY;
        } else {
            word.1.sections[0].style.color = Color::WHITE;
        }
    }
}

pub fn regenerate_sentence_structure(
    words: Query<QDraggableWord>,
    mut sentence_ui_parents: Query<(&SentenceUIParent, Option<&mut Children>)>,
    mut sentences: Query<(Entity, &mut SentenceStructure)>,
    mut ui_changes: EventReader<SentenceUIChanged>,
    mut structure_changes: EventWriter<SentenceStructureChanged>,
) {
    for ui_change in ui_changes.read() {
        let Ok(ui_parent) = sentence_ui_parents.get_mut(ui_change.ui_parent)
            else { continue };

        let words: Vec<WordID> = if let Some(mut word_objects) = ui_parent.1 {
            word_objects.sort_by_key(|entity| {
                if *entity == ui_change.word_entity {
                    ui_change.word_pos.x as u32
                } else {
                    words.get(*entity).unwrap().global_transform.translation().x as u32
                }
            });

            word_objects
                .iter()
                .map(|entity| words.get(*entity).unwrap().draggable.word_id)
                .collect()
        } else {
            Vec::new()
        };

        let (sentence_entity, mut sentence) = 
            sentences.get_mut(ui_parent.0.sentence_entity).unwrap();

        sentence.sentence = PhraseMap::default();
        let root = sentence.sentence.insert(PhraseData::default());
        sentence.root = root;

        let mut word_iter = itertools::peek_nth(words.iter());
        parse_noun_phrase(&mut word_iter, &mut *sentence, root);

        if !word_iter.is_empty() {
            sentence.sentence = PhraseMap::default();
        }

        structure_changes.send(SentenceStructureChanged { on: sentence_entity });
    }
}

fn parse_noun_phrase(
    words: &mut PeekNth<std::slice::Iter<word_id::WordID>>,
    sentence: &mut SentenceStructure,
    insert_into: PhraseID,
) {
    use PartOfSpeech::*;

    if words.is_empty() { return }

    let adjective = parse_adjective_phrase(words, sentence);

    if part_of_speech(words.peek().copied()).contains(&Noun) {
        sentence.sentence[insert_into] = PhraseData {
            word: Some(*words.next().unwrap()),
            kind: PhraseKind::Noun { adjective },
            ..default()
        }
    }
}

fn parse_adjective_phrase(
    words: &mut PeekNth<std::slice::Iter<word_id::WordID>>,
    sentence: &mut SentenceStructure,
) -> PhraseID {
    use PartOfSpeech::*;

    let peek_speech = part_of_speech(words.peek().copied()); 

    if peek_speech.contains(&Adjective) {
        if words.peek_nth(1).is_none() && peek_speech.contains(&Noun) {
            // this is the last word in the sentence - it must be a noun, not an adjective
            return sentence.sentence.insert(PhraseData {
                word: None,
                kind: PhraseKind::Adjective,
                ..default()
            });
        }

        let adjective_word = words.next().unwrap();
        let adjective_one = sentence.sentence.insert(PhraseData {
            word: Some(*adjective_word),
            kind: PhraseKind::Adjective,
            ..default()
        });

        let conj_word = part_of_speech(words.peek().copied());

        if conj_word.contains(&Conjuction) {
            let conjunction_word = words.next().unwrap();
            let adjective_two = parse_adjective_phrase(words, sentence);
            sentence.sentence.insert(PhraseData {
                word: Some(*conjunction_word),
                kind: PhraseKind::Combine {
                    l: adjective_one,
                    r: adjective_two,
                },
                ..default()
            })
        } else {
            adjective_one
        }
    } else if peek_speech.contains(&Conjuction) {
        let null_adjective = sentence.sentence.insert(PhraseData {
            word: None,
            kind: PhraseKind::Adjective,
            ..default()
        });

        let conjunction_word = words.next().unwrap();
        let adjective_two = parse_adjective_phrase(words, sentence);

        sentence.sentence.insert(PhraseData {
            word: Some(*conjunction_word),
            kind: PhraseKind::Combine {
                l: null_adjective,
                r: adjective_two,
            },
            ..default()
        })
    } else {
        sentence.sentence.insert(PhraseData {
            word: None,
            kind: PhraseKind::Adjective,
            ..default()
        })
    }
}

fn part_of_speech(word: Option<&WordID>) -> &'static [PartOfSpeech] {
    word.copied().map(WordID::part_of_speech).unwrap_or_default()
}
