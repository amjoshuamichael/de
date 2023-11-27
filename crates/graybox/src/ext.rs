use bevy::{prelude::*, reflect::*, ecs::component::Components};

use crate::{*, apply::*, timed_interaction::TimedInteraction};

pub trait GrayboxExt {
    fn enable_inspection<T: Component + Reflect>(self: &mut Self) -> &mut App;
}

impl GrayboxExt for App {
    fn enable_inspection<T: Component + Reflect>(self: &mut Self) -> &mut App {
        self.add_systems(Update, (
            show_in_inspector::<T>,
            apply_modifications::<T>,
            update_in_inspector::<T>,
        ).chain().in_set(UIUpdateStages::UpdateData));
        self
    }
}

fn show_in_inspector<T: Component + Reflect>(
    inspectors: Query<(&Inspector, Entity), Added<Inspector>>,
    items: Query<&T>,
    components: &Components,
    mut commands: Commands,
) {
    for inspector in &inspectors {
        let Ok(item) = items.get(inspector.0.on) else { continue };

        let section = commands.spawn((
            NodeBundle {
                style: Style { 
                    width: Val::Percent(100.), 
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(10.)),
                    ..default() 
                },
                ..default()
            },
            InspectorSection {
                on: components.component_id::<T>().unwrap(),
            },
        )).set_parent(inspector.1).id();

        commands.spawn(TextBundle {
            text: Text::from_section(
                std::any::type_name::<T>(), 
                TextStyle::default(),
            ),
            ..default()
        }).set_parent(section);

        spawn_ui_for(item.reflect_ref(), section, &mut commands);
    }
}

fn spawn_ui_for(
    reflected: ReflectRef, 
    parent: Entity, 
    commands: &mut Commands,
) {
    match reflected {
        ReflectRef::Struct(reflected_struct) => {
            for (f, reflected_field) in reflected_struct.iter_fields().into_iter().enumerate() {
                let is_primitive = matches!(reflected_field.reflect_ref(), ReflectRef::Value(_));
                let field_name = reflected_struct.name_at(f).unwrap_or("");
                
                let sub_menu = commands.spawn((
                    NodeBundle {
                        style: Style {
                            padding: UiRect::left(Val::Px(10.0)),
                            flex_direction: if is_primitive { 
                                    FlexDirection::Row 
                                } else {
                                    FlexDirection::Column 
                                },
                            ..default()
                        },
                        ..default()
                    },
                    InspectorSubmenu {
                        path_kind: PathKind::Field(field_name.to_string()),
                    },
                )).set_parent(parent).id();

                commands.spawn(
                    TextBundle {
                        text: Text::from_section(
                            field_name,
                            TextStyle {
                                ..default()
                            }
                        ),
                        style: Style {
                            flex_direction: FlexDirection::RowReverse,
                            overflow: Overflow::clip_x(),
                            margin: if is_primitive { 
                                    UiRect::right(Val::Px(10.0))
                                } else {
                                    default()
                                },
                            ..default()
                        },
                        ..default()
                    },
                ).set_parent(sub_menu);
                
                let reflect_ref = reflected_field.reflect_ref();

                spawn_ui_for(reflect_ref, sub_menu, commands)
            }
        },
        ReflectRef::TupleStruct(_) => todo!(),
        ReflectRef::Tuple(_) => todo!(),
        ReflectRef::List(_) => todo!(),
        ReflectRef::Array(_) => todo!(),
        ReflectRef::Map(_) => todo!(),
        ReflectRef::Enum(_) => todo!(),
        ReflectRef::Value(refval) if refval.is::<f32>() || refval.is::<f64>() => {
            let (val, string): (Box<dyn Reflect>, _) = 
                if let Some(f32) = refval.downcast_ref::<f32>() { 
                    (Box::new(*f32), f32.to_string())
                } else if let Some(f64) = refval.downcast_ref::<f64>() {
                    (Box::new(*f64), f64.to_string())
                } else { unreachable!() };

            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        string,
                        TextStyle::default(),
                    ),
                    style: Style { 
                        overflow: Overflow::clip_x(),
                        flex_grow: 2.0,
                        ..default()
                    },
                    ..default()
                },
                EditableFloat { is_64_bit: refval.is::<f64>(), },
                Editable { val, },
                TextInput::default(),
                TimedInteraction::default(),
                Interaction::None,
            )).set_parent(parent);
        },
        ReflectRef::Value(refval) if refval.is::<String>() => {

        }
        _ => todo!(),
    }
}

fn update_in_inspector<T: Component + Reflect>(
    inspectors: Query<(&Inspector, &Children)>,
    inspector_sections: Query<(&InspectorSection, Entity, &Children)>,
    submenus: Query<(&InspectorSubmenu, Entity, &Children)>,
    items: Query<(&T, Entity), Changed<T>>,
    mut editables: Query<&mut Editable>,
    mut texts: Query<&mut Text>,
    components: &Components,
) {
    let comp_id = components.component_id::<T>().unwrap();

    for (changed_data, changed_entity) in &items {
        let Some(inspector) = 
            inspectors.iter().find(|inspector| inspector.0.on == changed_entity) 
                else { continue };
        
        let Some(section) = inspector.1.iter().find(|sec| {
                let Ok(section) = inspector_sections.get(**sec) else { return false };
                section.0.on == comp_id
            }) else { continue };

        let (_, parent_entity, parent_children) = 
            inspector_sections.get(*section).unwrap();

        update_ui_for(
            changed_data.reflect_ref(), 
            (parent_entity, parent_children), 
            &submenus, 
            &mut editables,
            &mut texts,
        );
    }
}

fn update_ui_for(
    val: ReflectRef,
    parent_submenu: (Entity, &Children),
    all_submenus: &Query<(&InspectorSubmenu, Entity, &Children)>,
    editables: &mut Query<&mut Editable>,
    texts: &mut Query<&mut Text>,
) {
    for child in parent_submenu.1 {
        if let Ok(submenu) = all_submenus.get(*child) {
            match &submenu.0.path_kind {
                PathKind::Field(name) => {
                    let ReflectRef::Struct(val) = val else { panic!() };
                    update_ui_for(
                        val.field(&**name).unwrap().reflect_ref(),
                        (submenu.1, submenu.2),
                        all_submenus,
                        editables,
                        texts,
                    )
                },
            }
        } else if let Ok(mut editable) = editables.get_mut(*child) {
            let parent_submenu = all_submenus.get(parent_submenu.0).unwrap();
            let mut text = texts.get_mut(parent_submenu.2[1]).unwrap();
            
            let ReflectRef::Value(val) = val else { panic!() };
            let (val, as_string): (Box<dyn Reflect>, String) = 
                if let Some(f32) = val.downcast_ref::<f32>() {
                    (Box::new(*f32), f32.to_string())
                } else if let Some(f64) = val.downcast_ref::<f64>() {
                    (Box::new(*f64), f64.to_string())
                } else { unreachable!() };

            editable.val = val;
            text.sections = vec![as_string.into()];
        } else {
            debug_assert!(texts.get(*child).is_ok());
        }
    }
}
