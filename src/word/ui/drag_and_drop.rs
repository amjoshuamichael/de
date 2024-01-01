use crate::prelude::*;

use super::*;

use bevy::ecs::query::WorldQuery;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::MouseMotion;
use bevy::ui::{Interaction, ui_layout_system};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct QDraggableWord {
    style: &'static mut Style,
    node: &'static Node,
    interaction: &'static Interaction,
    transform: &'static Transform,
    background_color: &'static mut BackgroundColor,
    border_color: &'static mut BorderColor,
    pub draggable: &'static mut DraggableWord,
    pub global_transform: &'static GlobalTransform,
    entity: Entity,
    parent: &'static Parent,
}

#[derive(Component)]
pub struct Dragging;

#[derive(Event)]
pub struct SentenceUIChanged {
    pub ui_parent: Entity,
    pub word_entity: Entity,
    pub word_pos: Vec2,
}

impl<'a> QDraggableWordItem<'a> {
    fn set_pos_absolute(&mut self) {
        let center = self.node.logical_rect(self.global_transform);
        self.style.left = Val::Px(center.min.x);
        self.style.top = Val::Px(center.min.y);
        self.style.position_type = PositionType::Absolute;
    }

    fn set_pos_relative(&mut self) {
        self.style.left = Val::Auto;
        self.style.top = Val::Auto;
        self.style.position_type = PositionType::Relative;
    }
}

pub fn do_drag(
    mut dragging: Query<QDraggableWord, With<Dragging>>,
    not_dragging: Query<QDraggableWord, Without<Dragging>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut windows: Query<&mut Window>,
) {
    windows.single_mut().cursor.icon = if !dragging.is_empty() {
        CursorIcon::Grabbing
    } else if not_dragging.iter().any(|d| *d.interaction == Interaction::Hovered) {
        CursorIcon::Grab
    } else {
        CursorIcon::Default
    };

    let motion_delta: Vec2 = mouse_motion.read().map(|motion| motion.delta).sum();
    for mut draggable in &mut dragging {
        let Val::Px(left) = &mut draggable.style.left else { continue };
        *left += motion_delta.x;
        let Val::Px(top) = &mut draggable.style.top else { continue };
        *top += motion_delta.y;
        dbg!(&draggable.style.left);
        dbg!(&draggable.style.top);
        dbg!(&draggable.style.position_type);
        dbg!(&draggable.parent);
    }
}

pub fn do_snap(
    mut draggables: Query<QDraggableWord, With<Dragging>>,
    sentence_ui_parents: Query<(&Node, &GlobalTransform, Entity), With<SentenceUIParent>>, 
    inventory: Query<Entity, With<Inventory>>,
    mouse: Res<Input<MouseButton>>,
    mut commands: Commands,
    mut ui_changes: EventWriter<SentenceUIChanged>,
) {
    if mouse.pressed(MouseButton::Left) || draggables.is_empty() { return };

    let inventory = inventory.single();

    for mut draggable in &mut draggables {
        let new_parent = sentence_ui_parents
            .iter()
            .find_map(|ui_parent| {
                let rect = ui_parent.0.logical_rect(ui_parent.1);
                rect.contains(draggable.global_transform.translation().xy())
                    .then_some(ui_parent.2)
            })
            .unwrap_or(inventory);

        draggable.set_pos_relative();
        commands.entity(draggable.entity)
            .remove::<Dragging>()
            .set_parent(dbg!(new_parent));
        
        ui_changes.send(SentenceUIChanged { 
            ui_parent: new_parent,
            word_entity: draggable.entity,
            word_pos: draggable.global_transform.translation().xy(),
        });
    }
}

pub fn do_unsnap(
    mut draggables: Query<QDraggableWord, (Changed<Interaction>, Without<Dragging>)>,
    drag_parent: Query<Entity, With<DraggingParent>>,
    mut commands: Commands,
    mut ui_changes: EventWriter<SentenceUIChanged>,
) {
    for mut draggable in &mut draggables {
        if *draggable.interaction == Interaction::Pressed {
            draggable.set_pos_absolute();

            commands.entity(draggable.entity)
                .insert(Dragging)
                .set_parent(dbg!(drag_parent.single()));

            ui_changes.send(SentenceUIChanged { 
                ui_parent: **draggable.parent,
                word_entity: draggable.entity,
                word_pos: draggable.global_transform.translation().xy(),
            });
        }
    }
}
