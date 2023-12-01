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
    draggable: &'static mut DraggableWord,
    global_transform: &'static GlobalTransform,
    entity: Entity,
    parent: &'static Parent,
}

#[derive(Component)]
pub struct Dragging;

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
    mut draggables: Query<QDraggableWord, With<Dragging>>,
    mut mouse_motion: EventReader<MouseMotion>,
) {
    for mut draggable in &mut draggables {
        for motion in mouse_motion.read() {
            let Val::Px(left) = &mut draggable.style.left else { return };
            *left += motion.delta.x;
            let Val::Px(top) = &mut draggable.style.top else { return };
            *top += motion.delta.y;
        }
    }
}

pub fn do_snap(
    mut draggables: Query<QDraggableWord, With<Dragging>>,
    docks: Query<(&GlobalTransform, Entity, Option<&Children>), With<WordDock>>,
    inventory: Query<Entity, With<Inventory>>,
    mouse: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) { return };

    for mut draggable in &mut draggables {
        let draggable_rect = draggable.node
            .logical_rect(draggable.global_transform)
            .inset(10.0);

        let dock = docks
            .iter()
            .find(|dock| 
                draggable_rect.contains(dock.0.translation().xy()) &&
                dock.2.is_none()
            )
            .unwrap_or_else(|| docks.get(inventory.single()).unwrap());

        draggable.set_pos_relative();
        commands.entity(draggable.entity)
            .remove::<Dragging>()
            .set_parent(dock.1);

        command_snap_routines(&mut commands, dock.1);
    }
}

pub fn do_unsnap(
    mut draggables: Query<QDraggableWord, (Changed<Interaction>, Without<Dragging>)>,
    drag_parent: Query<Entity, With<DraggingParent>>,
    mut commands: Commands,
) {
    for mut draggable in &mut draggables {
        if *draggable.interaction == Interaction::Pressed {
            draggable.set_pos_absolute();
            commands.entity(draggable.entity)
                .insert(Dragging)
                .set_parent(drag_parent.single());

            command_snap_routines(&mut commands, **draggable.parent);
        }
    }
}

pub fn command_snap_routines(commands: &mut Commands, dock: Entity) {
    commands.add(move |world: &mut World| {
        world.run_system_once((move || dock).pipe(super::sentence_section_docks));
    });
}
