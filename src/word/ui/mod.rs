use crate::prelude::*;

use bevy::ecs::query::{WorldQuery, ReadOnlyWorldQuery};
use bevy::input::mouse::MouseMotion;
use bevy::ui::Interaction;

use crate::load_assets::MiscAssets;

use super::*;

mod drag_and_drop;
pub use drag_and_drop::*;

#[derive(Default, Component)]
pub struct DraggableWord {
    word_id: WordID,
    locked: bool,
}

#[derive(Component, Default)]
pub struct WordSnapParent;

#[derive(Debug, Component)]
pub struct SentenceSection {
    pub for_phrase: PhraseID,
    pub sentence_entity: Entity,
    pub locked: bool,
}

impl SentenceSection {
    fn new(for_phrase: PhraseID, sentence_entity: Entity) -> Self {
        Self { for_phrase, sentence_entity, locked: false }
    }
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
pub struct WordDock;

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
                    "",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 100.0,
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
                width: Val::Percent(25.),
                height: Val::Percent(100.),
                right: Val::Px(0.),
                ..default()
            },
            background_color: Color::YELLOW_GREEN.with_a(0.2).into(),
            ..default()
        },
        WordDock,
    )).id();

    commands.spawn((
        DraggingParent,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        }
    ));

    let _word_snap_parent = commands.spawn((
        WordSnapParent,
        NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        },
    )).id();

    //let _da = commands.spawn(TextBundle {
    //    text: Text::from_section(
    //        "Da",
    //        TextStyle {
    //            // This font is loaded and will be used instead of the default font.
    //            font: assets.font.clone(),
    //            font_size: 100.0,
    //            ..default()
    //        }
    //    ),
    //    style: Style {
    //        min_height: Val::Px(60.0),
    //        padding: UiRect::all(Val::Px(10.)),
    //        ..default()
    //    },
    //    ..default()
    //}).set_parent(word_snap_parent);
}

pub fn words_init(
    mut added_words: Query<(&DraggableWord, &mut Text), Added<DraggableWord>>,
    word_map: Res<Words>,
) {
    for (word, mut text) in &mut added_words {
        let word = &word_map.0[&word.word_id];
        text.sections[0].value = word.basic.to_string();
    }
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

pub fn update_sentence_ui(
    structures: Query<(&SentenceStructure, Option<&Children>, Entity)>,
    mut structure_changes: EventReader<SentenceStructureChanged>,
    new_structures: Query<Entity, Added<SentenceStructure>>,
    word_snap_parent: Query<Entity, With<WordSnapParent>>,
    mut commands: Commands,
    children: Query<&Children>,
    sentence_parts: Query<(&SentenceUIPart, Entity)>,
    docks: Query<(), &SentenceSection>,
    assets: Res<MiscAssets>,
) {
    let word_snap_parent = word_snap_parent.single();

    let structures_to_update: Vec<Entity> = 
        structure_changes.read().map(|change| change.on)
        .chain(new_structures.iter())
        .collect();

    for structure_entity in &structures_to_update {
        let sentence = structures.get(*structure_entity).unwrap();
        let mut spawned = Vec::new();

        spawn_snap_for(
            sentence.0.root,
            (sentence.2, &sentence.0),
            word_snap_parent,
            &mut commands,
            &children,
            &sentence_parts,
            &mut spawned,
            &*assets,
        );

        for unused in sentence_parts.iter().filter(|part| !spawned.contains(&part.1)) {
            if docks.get(unused.1).is_ok() {
                commands.entity(unused.1).despawn_recursive();
            } else {
                commands.entity(unused.1).remove_parent();
                commands.entity(unused.1).despawn();
            }
        }
    }
}

pub fn spawn_snap_for(
    phrase: PhraseID, 
    sentence: (Entity, &SentenceStructure),
    parent: Entity,
    commands: &mut Commands,
    children: &Query<&Children>,
    sentence_parts: &Query<(&SentenceUIPart, Entity)>,
    spawned: &mut Vec<Entity>,
    assets: &MiscAssets,
) -> Entity {
    // TODO: have this return EntityCommands??
    let mut find_or_spawn = |
        part_id: SentenceUIPart, 
        parent: Entity, 
        spawn_word: Option<WordID>,
    | -> Entity {
        let mut new_entity = || {
            let entity = commands.spawn((part_id, InheritedVisibility::default(), GlobalTransform::default()))
                .set_parent(parent)
                .id();
            if let Some(word_id) = spawn_word {
                commands
                    .spawn(DraggableWordBundle::for_word_snapped(word_id, assets))
                    .set_parent(entity);
            }
            spawned.push(entity);
            entity
        };

        let Ok(siblings) = children.get(parent) else { return new_entity(); };

        let Some(existing) = siblings.iter().find(|e| {
                let Ok(part) = sentence_parts.get(**e) else { return false };
                return *part.0 == part_id;
            }) else { return new_entity() };

        let first_child = children.get(*existing).map_or_else(|_| None, |c| c.get(0)).copied();
        if let Some(word_id) = spawn_word && first_child.is_none() {
            commands
                .spawn(DraggableWordBundle::for_word_snapped(word_id, assets))
                .set_parent(*existing);
        } else if spawn_word == None && part_id.is_slot() && let Some(child) = first_child {
            commands.add(move |world: &mut World| {
                if !world.entity(child).contains::<Dragging>() {
                    world.despawn(child);
                }
            })
        }

        spawned.push(*existing);
        *existing
    };

    match &sentence.1.sentence[phrase] {
        PhraseData { kind: PhraseKind::Noun { adjective }, word } => {
            let noun_joint = find_or_spawn(NounJoint, parent, None);
            let noun_slot = find_or_spawn(NounSlot, noun_joint, *word);

            commands.entity(noun_joint).insert((
                NodeBundle { 
                    style: Style { padding: UiRect::all(Val::Px(10.)), ..default() }, 
                    ..default() 
                },
                SentenceJoint,
            ));

            spawn_snap_for(*adjective, sentence, noun_joint, 
                commands, children, sentence_parts, spawned, assets);

            commands.entity(noun_slot).insert((
                NodeBundle {
                    background_color: BackgroundColor::from(Color::DARK_GREEN.with_a(0.5)),
                    style: Style {
                        min_width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    ..default()
                },
                SentenceSection::new(phrase, sentence.0),
                WordDock,
            ));

            noun_slot
        },
        PhraseData { kind: PhraseKind::Adjective, word } => {
            let adjective_slot = find_or_spawn(AdjectiveSlot, parent, *word);

            commands.entity(adjective_slot).insert((
                NodeBundle {
                    background_color: BackgroundColor::from(Color::YELLOW.with_a(0.5)),
                    style: Style {
                        min_width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    ..default()
                },
                SentenceSection::new(phrase, sentence.0),
                WordDock,
            ));

            adjective_slot
        },
        PhraseData { kind: PhraseKind::CombineAdjectives { l, r }, .. } => {
            let combine_joint = find_or_spawn(CombineJoint, parent, None);
            let combine_jointl = find_or_spawn(CombineJointL, combine_joint, None);
            let and_node = find_or_spawn(AndSlot, combine_joint, Some(WordID::And));
            let combine_jointr = find_or_spawn(CombineJointR, combine_joint, None);

            commands.entity(combine_joint).insert((
                NodeBundle {
                    style: Style { padding: UiRect::all(Val::Px(10.)), ..default() },
                    ..default()
                },
                SentenceJoint,
            ));

            commands.entity(combine_jointl).insert(NodeBundle::default());
            spawn_snap_for(*l, sentence, combine_jointl, 
                commands, children, sentence_parts, spawned, assets);

            commands.entity(and_node).insert((
                NodeBundle {
                    background_color: BackgroundColor::from(Color::ORANGE.with_a(0.5)),
                    style: Style {
                        min_width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        ..default()
                    },
                    ..default()
                },
                SentenceSection::new(phrase, sentence.0),
                WordDock,
            )).set_parent(combine_joint);

            commands.entity(combine_jointr).insert(NodeBundle::default());
            spawn_snap_for(*r, sentence, combine_jointr, 
                commands, children, sentence_parts, spawned, assets);

            combine_joint
        },
    }
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

pub fn sentence_section_docks(
    docks: Query<&SentenceSection>,
    words: Query<&DraggableWord>,
    mut sentences: Query<(Entity, &mut SentenceStructure)>,
    mut ui_changes: EventReader<SentenceUIChanged>,
    mut structure_changes: EventWriter<SentenceStructureChanged>,
) {
    let mut modified_sentences = HashSet::<Entity>::new();

    for ui_change in ui_changes.read() {
        // if this isn't a dock, it's the inventory, so we do nothing
        let Ok(dock) = docks.get(ui_change.for_dock) else { continue };
        let mut sentence_components = sentences.get_mut(dock.sentence_entity).unwrap();
        let sentence = &mut sentence_components.1.sentence;
        let set_phrase = dock.for_phrase;

        let word_id = words.get(ui_change.word).unwrap().word_id;
        if ui_change.added {
            match word_id {
                WordID::And => {
                    let l = sentence.insert(PhraseData::kind(PhraseKind::Adjective));
                    let r = sentence.insert(PhraseData::kind(PhraseKind::Adjective));

                    sentence[set_phrase] = PhraseData {
                        word: Some(WordID::And),
                        kind: PhraseKind::CombineAdjectives { l, r },
                    };
                }
                other_word_id => {
                    sentence[set_phrase].word = Some(other_word_id);
                }
            }
        } else {
            match word_id {
                WordID::And => {
                    sentence[set_phrase] = PhraseData::kind(PhraseKind::Adjective);
                },
                _ => sentence[set_phrase].word = None,
            }
        };

        modified_sentences.insert(dock.sentence_entity);
    }

    for sentence_entity in modified_sentences {
        structure_changes.send(SentenceStructureChanged {
            on: sentence_entity,
        });
    }
}
