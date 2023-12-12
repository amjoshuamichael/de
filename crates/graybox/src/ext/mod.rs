use bevy::{prelude::*, ecs::component::Components};

use crate::{*, timed_interaction::TimedInteraction};

mod inspection;
mod functions;
mod apply;

pub trait GrayboxExt {
    fn enable_inspection<T: Component + Reflect>(self: &mut Self) -> &mut App;
    fn enable_functions<T: Component + GrayboxFunctions>(self: &mut Self) -> &mut App;
}

pub trait GrayboxFunctions: Component {
    fn functions() -> Vec<(&'static str, fn(&mut Self))>;
}

impl GrayboxExt for App {
    fn enable_inspection<T: Component + Reflect>(self: &mut Self) -> &mut App {
        self.add_systems(Update, (
            create_inspector_section_for_component::<T>,
            inspection::show_in_inspector::<T>,
            apply::apply_modifications::<T>.in_set(UIUpdateStages::UpdateData),
            inspection::update_in_inspector::<T>.in_set(UIUpdateStages::UpdateInterfaces),
        ));
            
        self
    }

    fn enable_functions<T: Component + GrayboxFunctions>(self: &mut Self) -> &mut App {
        self.add_systems(Update, (
            create_inspector_section_for_component::<T>,
            functions::show_in_inspector::<T>,
            functions::function_buttons::<T>.in_set(UIUpdateStages::InterfaceSubmits),
        ).chain())
    }
}

pub fn create_inspector_section_for_component<T: Component>(
    inspectors: Query<(&Inspector, Entity), Added<Inspector>>,
    components: &Components,
    items: Query<&T>,
    mut commands: Commands,
) {
    for inspector in &inspectors {
        if items.get(inspector.0.on).is_err() { continue }

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
                component_id: components.component_id::<T>().unwrap(),
            },
        )).set_parent(inspector.1).id();

        commands.spawn(TextBundle {
            text: Text::from_section(
                format!("{}//{:?}", std::any::type_name::<T>(), inspector.0.on),
                TextStyle::default(),
            ),
            ..default()
        }).set_parent(section);
    }
}
