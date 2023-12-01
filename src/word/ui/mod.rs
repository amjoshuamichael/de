use crate::prelude::*;

use bevy::ecs::query::WorldQuery;
use bevy::input::mouse::MouseMotion;
use bevy::ui::Interaction;

use crate::load_assets::MiscAssets;

use super::*;

mod drag_and_drop;
pub use drag_and_drop::*;

#[derive(Default, Component)]
pub struct DraggableWord {
    word_id: WordID,
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

#[derive(Component)]
pub struct Inventory {
    
}

#[derive(Component)]
pub struct WordDock;

#[derive(Component)]
pub struct DraggingParent;

const TEXT_OBJECTS_Z_INDEX: ZIndex = ZIndex::Global(30);

#[derive(Bundle)]
pub struct DraggableWordBundle {
    text: TextBundle,
    draggable: DraggableWord,
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
            },
            interaction: Interaction::default(),
        }
    }
}

pub fn setup_word_ui(
    mut commands: Commands,
    assets: Res<MiscAssets>,
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

    let word_snap_parent = commands.spawn((
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

    let _da = commands.spawn(TextBundle {
        text: Text::from_section(
            "Da",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: assets.font.clone(),
                font_size: 100.0,
                ..default()
            }
        ),
        style: Style {
            min_height: Val::Px(60.0),
            padding: UiRect::all(Val::Px(10.)),
            ..default()
        },
        ..default()
    }).set_parent(word_snap_parent);
}

pub fn update_sentence_ui(
    changed_structures: Query<
        (&SentenceStructure, Option<&Children>, Entity),
        Added<SentenceStructure>
    >,
    word_snap_parent: Query<Entity, With<WordSnapParent>>,
    mut commands: Commands,
) {
    let word_snap_parent = word_snap_parent.single();

    for sentence in &changed_structures {
        spawn_snap_for(
            sentence.0.root,
            (sentence.2, &sentence.0),
            &mut commands,
            word_snap_parent,
        );
    }
}

pub fn update_word_ui(
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

pub fn spawn_snap_for(
    phrase: PhraseID, 
    sentence: (Entity, &SentenceStructure),
    commands: &mut Commands,
    word_snap_parent: Entity,
) {
    match &sentence.1.sentence[phrase] {
        PhraseData { kind: PhraseKind::Noun { adjective }, .. } => {
            let noun_parent = commands.spawn((
                NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    ..default()
                },
            ))
                .set_parent(word_snap_parent)
                .id();

            spawn_snap_for(*adjective, sentence, commands, noun_parent);

            let _noun_slot = commands.spawn((
                NodeBundle {
                    background_color: BackgroundColor::from(Color::DARK_GREEN.with_a(0.5)),
                    style: Style {
                        display: Display::Flex,
                        min_width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    ..default()
                },
                SentenceSection::new(phrase, sentence.0),
                WordDock,
            )).set_parent(noun_parent);
        },
        PhraseData { kind: PhraseKind::Adjective, .. } => {
            commands.spawn((
                NodeBundle {
                    background_color: BackgroundColor::from(Color::YELLOW.with_a(0.5)),
                    style: Style {
                        display: Display::Flex,
                        min_width: Val::Px(100.0),
                        min_height: Val::Px(60.0),
                        padding: UiRect::all(Val::Px(10.)),
                        overflow: Overflow {
                            x: OverflowAxis::Clip,
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                },
                SentenceSection::new(phrase, sentence.0),
                WordDock,
                Name::new("Word Snap Point"),
            )).set_parent(word_snap_parent);
        },
    }
}

pub fn sentence_section_docks(
    In(changed_dock): In<Entity>,
    docks: Query<(Option<&Children>, &SentenceSection)>,
    words: Query<&DraggableWord>,
    mut sentences: Query<(Entity, &mut SentenceStructure)>,
    mut change_events: EventWriter<SentenceStructureChanged>,
) {
    let Ok(dock) = docks.get(changed_dock) else { return };

    let mut sentence = sentences.get_mut(dock.1.sentence_entity).unwrap();

    sentence.1.sentence[dock.1.for_phrase].word = 
        if let Some(docked) = dock.0 {
            let word = words.get(docked[0]).unwrap();
            Some(word.word_id)
        } else {
            None
        };

    change_events.send(SentenceStructureChanged {
        on: sentence.0,
    });
}
