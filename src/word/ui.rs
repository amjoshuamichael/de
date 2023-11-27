use crate::prelude::*;
use bevy::ecs::query::WorldQuery;
use bevy::input::mouse::MouseMotion;
use bevy::ui::Interaction;

use crate::load_assets::DeAssets;

use super::{PhraseID, SentenceStructure, PhraseData, SentenceStructureChanged, WordID, Words, PhraseKind};

#[derive(Component)]
pub struct DraggableWord {
    is_being_dragged: bool,
    is_snapped: bool,
    word_id: WordID,
}

impl DraggableWord {
    fn for_word(word_id: WordID) -> Self {
        Self {
            is_being_dragged: false,
            is_snapped: false,
            word_id,
        }
    }
}

#[derive(Component, Default)]
pub struct DraggableWordParent;

#[derive(Component, Default)]
pub struct WordSnapParent;

#[derive(Component)]
pub struct WordSnapPoint {
    for_phrase: PhraseID,
    sentence: Entity,
}

const TEXT_OBJECTS_Z_INDEX: ZIndex = ZIndex::Global(30);

fn word_bundle_for(word_id: WordID, assets: &DeAssets) 
    -> (TextBundle, DraggableWord, Interaction) {
    (
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: assets.font.clone(),
                    font_size: 100.0,
                    ..default()
                }
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: BackgroundColor::from(Color::DARK_GREEN),
            z_index: TEXT_OBJECTS_Z_INDEX,
            ..default()
        },
        DraggableWord::for_word(word_id),
        Interaction::default(),
    )
}

pub fn setup_word_ui(
    mut commands: Commands,
    assets: Res<DeAssets>,
) {
    // Draggable word parent is the parent of all of the words that aren't currently 
    // snapped. When words get snapped, they are childfren of their snap point.
    let draggable_word_parent = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        },
        DraggableWordParent::default(),
    )).id();

    commands.entity(draggable_word_parent).with_children(|commands| {
        commands.spawn(word_bundle_for(WordID::Baby, &*assets));
        commands.spawn(word_bundle_for(WordID::Wide, &*assets));
        commands.spawn(word_bundle_for(WordID::Tall, &*assets));
    });

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
        text.sections[0].value = word.basic.clone();
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
                WordSnapPoint {
                    for_phrase: phrase,
                    sentence: sentence.0,
                },
            )).set_parent(noun_parent);
        },
        PhraseData { word, kind: PhraseKind::Adjective } => {
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
                WordSnapPoint {
                    for_phrase: phrase,
                    sentence: sentence.0,
                },
                Name::new("Word Snap Point"),
            )).set_parent(word_snap_parent);
        },
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct QDraggableWord {
    style: &'static mut Style,
    node: &'static Node,
    interaction: &'static Interaction,
    transform: &'static Transform, // we don't have to read transform, it is set 
                                   // automatically by the bevy UI library
    draggable: &'static mut DraggableWord,
    global_transform: &'static GlobalTransform,
    entity: Entity,
    parent: &'static Parent,
}

pub fn do_drag(
    mut draggables: Query<QDraggableWord>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for mut draggable in &mut draggables {
        draggable.draggable.is_being_dragged = 
            *draggable.interaction == Interaction::Pressed;

        if draggable.draggable.is_being_dragged {
            for motion in mouse_motion.read() {
                let Val::Px(left) = &mut draggable.style.left else { panic!() };
                *left += motion.delta.x;
                let Val::Px(top) = &mut draggable.style.top else { panic!() };
                *top += motion.delta.y;
            }
        }
    }
}

pub fn do_unsnap(
    mut draggables: Query<QDraggableWord>,
    snap_points: Query<&WordSnapPoint>,
    mut sentences: Query<&mut SentenceStructure>,
    draggable_parent: Query<Entity, With<DraggableWordParent>>,
    mut commands: Commands,
    mut change_events: EventWriter<SentenceStructureChanged>,
) {
    let draggable_parent = draggable_parent.single();

    for mut draggable in &mut draggables {
        if *draggable.interaction == Interaction::Pressed { 
            if draggable.draggable.is_snapped {
                let draggable_center = 
                    draggable.node.logical_rect(draggable.global_transform);

                draggable.style.left = Val::Px(draggable_center.min.x);
                draggable.style.top = Val::Px(draggable_center.min.y);
                draggable.style.position_type = PositionType::Absolute;
                draggable.draggable.is_snapped = false;

                let snap_point = snap_points.get(**draggable.parent).unwrap();
                let mut sentence = sentences.get_mut(snap_point.sentence).unwrap();
                sentence.sentence[snap_point.for_phrase].word = None;

                change_events.send(SentenceStructureChanged {
                    on: snap_point.sentence,
                });

                commands.entity(draggable.entity).set_parent(draggable_parent);
            }
        }
    }
}

pub fn do_snap(
    mut draggables: Query<QDraggableWord>,
    snap_points: Query<(&GlobalTransform, Entity, &Node, &WordSnapPoint)>,
    mut sentences: Query<&mut SentenceStructure>,
    mut commands: Commands,
    mut change_events: EventWriter<SentenceStructureChanged>,
) {
    for snap_point in &snap_points {
        for mut draggable in &mut draggables {
            if !draggable.draggable.is_snapped 
                && *draggable.interaction != Interaction::Pressed {
                let draggable_rect = draggable.node.logical_rect(draggable.global_transform);

                // if the width is 0, then the UI component hasn't loaded yet - this can 
                // cause bugs, so escape and wait for next frame
                if draggable_rect.width() == 0.0 { return; }

                let is_on_snap_point = draggable_rect
                    .inset(10.0)
                    .contains(snap_point.0.translation().xy());

                if is_on_snap_point {
                    commands.entity(draggable.entity).set_parent(snap_point.1);

                    draggable.style.left = Val::Auto;
                    draggable.style.top = Val::Auto;
                    draggable.style.position_type = PositionType::Relative;
                    draggable.draggable.is_snapped = true;

                    let mut sentence = sentences.get_mut(snap_point.3.sentence).unwrap();
                    sentence.sentence[snap_point.3.for_phrase].word = 
                        Some(draggable.draggable.word_id);

                    change_events.send(SentenceStructureChanged {
                        on: snap_point.3.sentence,
                    });
                }
            }
        }
    }
}
