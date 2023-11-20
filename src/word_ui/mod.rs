use bevy::ecs::query::WorldQuery;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::ui::Interaction;

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
pub struct WordSnapPoint;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, create_word_ui)
            .add_systems(Update, (
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
                ..default()
            },
            background_color: BackgroundColor::from(Color::DARK_GREEN),
            z_index: TEXT_OBJECTS_Z_INDEX,
            ..default()
        },
        DraggableWord::default(),
        Interaction::default(),
    )).set_parent(draggable_word_parent);

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

    commands.spawn((
        NodeBundle {
            background_color: BackgroundColor::from(Color::DARK_GRAY.with_a(0.5)),
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
        WordSnapPoint,
    )).set_parent(word_snap_parent);
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
                    //commands.entity(draggable.enitiy).remove_parent();
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
                }
            }
        }
    }
}
