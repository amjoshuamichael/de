use bevy::{prelude::*, ecs::component::Components};

use super::*;

pub fn show_in_inspector<T: Component + GrayboxFunctions>(
    sections: Query<(&InspectorSection, &Parent, Entity), Added<InspectorSection>>,
    components: &Components,
    mut commands: Commands,
) {
    let Some(component_id) = components.component_id::<T>() else { return };

    for section in &sections {
        if section.0.component_id != component_id { continue }

        for (f, (function_name, _)) in T::functions().into_iter().enumerate() {
            commands.spawn((
                TextBundle {
                    text: Text::from_section(function_name, TextStyle::default()),
                    style: Style { 
                        overflow: Overflow::clip_x(),
                        flex_grow: 2.0,
                        ..default()
                    },
                    ..default()
                },
                FunctionButton { function_index: f },
                Interaction::None,
            )).set_parent(section.2);
        }
    }
}

pub fn function_buttons<T: Component + GrayboxFunctions>(
    function_buttons: Query<(&FunctionButton, &Parent, &Interaction), Changed<Interaction>>,
    sections: Query<(&InspectorSection, &Parent)>,
    inspectors: Query<&Inspector>,
    components: &Components,
    mut items: Query<&mut T>,
) {
    for function_button in function_buttons.iter() {
        if *function_button.2 != Interaction::Pressed { continue }

        let section = sections.get(**function_button.1).unwrap();

        if components.component_id::<T>() != Some(section.0.component_id) { continue }

        let inspector = inspectors.get(**section.1).unwrap();
        
        let mut item = items.get_mut(inspector.on).unwrap();

        T::functions()[function_button.0.function_index].1(&mut *item);
    }
}
