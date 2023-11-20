use bevy::ecs::query::WorldQuery;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::ui::Interaction;

use super::{PhraseID, SentenceStructure, PhraseData, SentenceStructureChanged};

#[derive(Component, Default)]
pub struct DraggableWord {
    is_being_dragged: bool,
    is_snapped: bool,
}

#[derive(Component, Default)]
pub struct DraggableWordParent;

#[derive(Component, Default)]
pub struct WordSnapParent;

#[derive(Component, Default)]
pub struct WordSnapPoint {
    for_phrase: PhraseID,
}

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, create_word_ui)
            .add_event::<SentenceStructureChanged>()
            .add_systems(Update, (
                update_ui,
                unsnap,
                do_drag.after(unsnap),
                do_snap.after(do_drag),
            ));
    }
}

const TEXT_OBJECTS_Z_INDEX: ZIndex = ZIndex::Global(30);

fn create_word_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "baby",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: asset_server.load("fonts/tempfont.ttf"),
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
        DraggableWord::default(),
        Interaction::default(),
    )).set_parent(draggable_word_parent);

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
}

pub fn update_ui(
    changed_structures: Query<
        (&SentenceStructure, Option<&Children>),
        Or<(Changed<SentenceStructure>, Added<SentenceStructure>)>
    >,
    word_snap_parent: Query<Entity, With<WordSnapParent>>,
    mut commands: Commands,
) {
    let word_snap_parent = word_snap_parent.single();

    for sentence in &changed_structures {
        // despawn the current children of the sentence structure
        if let Some(children) = sentence.1 {
            for child_node in children {
                commands.entity(*child_node).despawn();
            }
        }

        spawn_snap_for(
            sentence.0.root,
            &sentence.0,
            &mut commands,
            word_snap_parent,
        );
    }
}

pub fn spawn_snap_for(
    phrase: PhraseID, 
    sentence: &SentenceStructure, 
    mut commands: &mut Commands,
    word_snap_parent: Entity,
) {
    match &sentence.sentence[phrase] {
        PhraseData::Noun { word, adjective } => {
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
                },
            )).set_parent(noun_parent);
        },
        PhraseData::Adjective { word } => {
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
                },
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

fn do_drag(
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

fn unsnap(
    mut draggables: Query<QDraggableWord>,
    draggable_parent: Query<Entity, With<DraggableWordParent>>,
    mut commands: Commands,
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
                commands.entity(draggable.entity).set_parent(draggable_parent);
            }
        }
    }
}

fn do_snap(
    mut draggables: Query<QDraggableWord>,
    snap_points: Query<(&GlobalTransform, Entity, &Node), With<WordSnapPoint>>,
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

                    change_events.send(SentenceStructureChanged {
                        on: draggable.entity,
                    })
                }
            }
        }
    }
}
