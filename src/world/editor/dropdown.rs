use bevy::prelude::*;

#[derive(Default, Component)]
pub struct Dropdown {
    pub choices: Vec<&'static str>,
    pub chosen: usize,
}

#[derive(Component)]
struct DropdownChoice {
    choice_index: usize,
}

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            open_dropdowns,
            update_dropdown_ui,
            dropdown_selection,
        ).chain());
    }
}

#[derive(Bundle)]
pub struct DropdownBundle<Marker: Component + Default> {
    pub dropdown: Dropdown,
    pub text: TextBundle,
    pub interaction: Interaction,
    pub marker: Marker,
}

impl<Marker: Component + Default> Default for DropdownBundle<Marker> {
    fn default() -> Self {
        DropdownBundle {
            dropdown: Dropdown::default(),
            text: TextBundle {
                style: Style { 
                    flex_direction: FlexDirection::Column, 
                    width: Val::Percent(100.),
                    ..default() 
                },
                background_color: Color::RED.into(),
                ..default()
            },
            interaction: Interaction::None,
            marker: Marker::default(),
        }
    }
}

fn open_dropdowns(
    dropdowns: Query<(&Dropdown, &Interaction, Entity), Changed<Interaction>>, 
    mut commands: Commands
) {
    for (dropdown, d_interaction, dropdown_entity) in &dropdowns {
        if *d_interaction == Interaction::Pressed {
            for (o, choice) in dropdown.choices.iter().enumerate() {
                commands.spawn((
                    TextBundle {
                        text: Text::from_section(
                            *choice, 
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                        background_color: Color::DARK_GRAY.into(),
                        style: Style {
                            width: Val::Percent(100.),
                            ..default()
                        },
                        ..default()
                    },
                    DropdownChoice { choice_index: o },
                    Interaction::None,
                )).set_parent(dropdown_entity);
            }
        }
    }
}

fn update_dropdown_ui(
    mut dropdown_choices: Query<
        (&Interaction, &mut BackgroundColor), 
        (Changed<Interaction>, With<DropdownChoice>)
    >, 
    mut dropdowns: Query<(&Dropdown, &mut Text), Or<(Changed<Dropdown>, Added<Dropdown>)>>,
) {
    for mut choice in &mut dropdown_choices {
        *choice.1 = match *choice.0 {
            Interaction::Pressed => Color::BLACK.into(),
            Interaction::Hovered => Color::GRAY.into(),
            Interaction::None => Color::DARK_GRAY.into(),
        }
    }

    for (dropdown, mut text) in &mut dropdowns {
        let chosen_string = &dropdown.choices[dropdown.chosen];
        text.sections = vec![chosen_string.to_string().into()];
    }
}

fn dropdown_selection(
    choice: Query<(&Interaction, &DropdownChoice, &Parent), Changed<Interaction>>,
    mut dropdowns: Query<&mut Dropdown>,
    mut commands: Commands,
) {
    for choice in &choice {
        if *choice.0 == Interaction::Pressed {
            let choice_index = choice.1.choice_index;
            let mut dropdown = dropdowns.get_mut(**choice.2).unwrap();
            dropdown.chosen = choice_index;

            commands.entity(**choice.2).despawn_descendants();
        }
    }
}
