use bevy::{prelude::*, input::mouse::MouseMotion, ecs::query::WorldQuery};

use crate::{*, timed_interaction::TimedInteraction};

pub struct Modification {
    pub set: Box<dyn Reflect>,
    pub editor_in_submenu: Entity,
    pub component_id: ComponentId,
}

// we need to read events multiple times, and consume them: these actions are not
// supported by the bevy event system. we use a Res<Vec<_>> instead.
#[derive(Default, Resource)]
pub struct ModificationEvents(pub Vec<Modification>);

#[derive(Default, Resource)]
pub struct SelectedEditable(pub Option<Entity>);

pub fn editable_selection(
    editables: Query<(&Interaction, &Editable, Entity)>,
    mut selected: ResMut<SelectedEditable>,
    mouse_button_input: Res<Input<MouseButton>>,
) {
    for editable in &editables {
        if *editable.0 == Interaction::Pressed {
            selected.0 = Some(editable.2);
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        selected.0 = None;
    }
}

pub fn selection_indicator(
    mut changed_editables: Query<
        (&Interaction, &mut BackgroundColor), 
        (Changed<Interaction>, With<Editable>),
    >,
) {
    for (interaction, mut background_color) in &mut changed_editables {
        *background_color = match *interaction {
            Interaction::Pressed => Color::GRAY.into(),
            Interaction::Hovered => Color::BLACK.into(),
            Interaction::None => Color::DARK_GRAY.into(),
        }
    }
}

pub fn text_input(
    mut text_inputs: Query<(&mut TextInput, &TimedInteraction, &mut Text, Entity)>,
    keys: Res<Input<KeyCode>>,
) {
    for mut text_input in &mut text_inputs {
        if let Some(text) = text_input.0.input.as_mut() {
            for key in keys.get_just_pressed() {
                use KeyCode::*;

                match key {
                    Delete | Back => {
                        text.pop();
                    },
                    Return => {
                        let submitted = text_input.0.input.take().unwrap();
                        text_input.0.submitted = Some(submitted);
                        return;
                    }
                    _ => {}
                }

                *text += match key {
                    Key1 => "1", Key2 => "2", Key3 => "3", Key4 => "4", Key5 => "5",
                    Key6 => "6", Key7 => "7", Key8 => "8", Key9 => "9", Key0 => "0",
                    A => "A", B => "B", C => "C", D => "D", E => "E", F => "F", G => "G",
                    H => "H", I => "I", J => "J", K => "K", L => "L", M => "M", N => "N",
                    O => "O", P => "P", Q => "Q", R => "R", S => "S", T => "T", U => "U",
                    V => "V", W => "W", X => "X", Y => "Y", Z => "Z", Minus => "-",
                    _ => continue,
                };
            }

            text_input.2.sections[0].value = text.clone();
        } else if text_input.1.single_clicked {
            let existing_text = text_input.2.sections[0].value.clone();
            text_input.0.input = Some(existing_text);
        }
    }
}

#[derive(WorldQuery)]
pub struct EditableQuery<T: Component> {
    interaction: &'static Interaction,
    timed_interaction: &'static TimedInteraction,
    editable: &'static Editable,
    inner: &'static T,
    parent: &'static Parent,
}

pub fn sliders(
    sliders: Query<EditableQuery<EditableFloat>>,
    selected_editable: Res<SelectedEditable>,
    submenus: Query<&Parent, With<InspectorSubmenu>>,
    inspector_sections: Query<&InspectorSection>,
    keys: Res<Input<KeyCode>>,
    mut motion: EventReader<MouseMotion>,
    mut modifications: ResMut<ModificationEvents>,
) {
    let Some(selected) = selected_editable.0 else { return };
    let Ok(slider) = sliders.get(selected) else { return };

    let mut x_delta: f32 = motion.read().map(|motion| motion.delta.x).sum();

    if keys.pressed(KeyCode::ShiftRight) || keys.pressed(KeyCode::ShiftLeft) {
        x_delta /= 100.;
    }

    let new_val: Box<dyn Reflect> = 
        if let Some(f32) = slider.editable.val.downcast_ref::<f32>() { 
            if slider.timed_interaction.double_clicked {
                Box::new(0.0f32)
            } else {
                Box::new(f32 + x_delta as f32)
            }
        } else if let Some(f64) = slider.editable.val.downcast_ref::<f64>() {
            if slider.timed_interaction.double_clicked {
                Box::new(0.0f64)
            } else {
                Box::new(f64 + x_delta as f64)
            }
        } else { unreachable!() };

    let section_root = AncestorIter::new(&submenus, **slider.parent).last().unwrap();
    let inspector_section = inspector_sections.get(section_root).unwrap();

    modifications.0.push(Modification {
        set: new_val,
        editor_in_submenu: **slider.parent,
        component_id: inspector_section.on,
    });
}

pub fn sliders_text(
    mut sliders: Query<(&mut Editable, &mut TextInput, &Parent), With<EditableFloat>>,
    submenus: Query<&Parent, With<InspectorSubmenu>>,
    inspector_sections: Query<&InspectorSection>,
    mut modifications: ResMut<ModificationEvents>,
) {
    for mut slider in &mut sliders {
        if let Some(submitted) = slider.1.submitted.take() {
            let new_val: Box<dyn Reflect> = if slider.0.val.is::<f32>() {
                let Ok(parsed) = submitted.parse::<f32>() else { continue };
                Box::new(parsed)
            } else if slider.0.val.is::<f64>() {
                let Ok(parsed) = submitted.parse::<f64>() else { continue };
                Box::new(parsed)
            } else { 
                continue;
            };

            let section_root = AncestorIter::new(&submenus, **slider.2).last().unwrap();
            let inspector_section = inspector_sections.get(section_root).unwrap();

            modifications.0.push(Modification {
                set: new_val,
                editor_in_submenu: **slider.2,
                component_id: inspector_section.on,
            });
        }
    }
}
