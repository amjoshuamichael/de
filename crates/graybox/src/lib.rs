#![feature(let_chains)]

// TODO:
// - list entity IDs
// - allow for types to define their own modifier functions (e.g. "snap"
// for Transform), and show them in the menu.

use apply::PathKind;
use bevy::{prelude::*, ecs::component::ComponentId};

mod timed_interaction;
mod ext;
mod apply;
mod interfaces;

pub use ext::GrayboxExt;
use interfaces::{sliders, ModificationEvents, editable_selection, selection_indicator, SelectedEditable, text_input, sliders_text};

pub struct GrayboxPlugin {
    pub open_graybox_command: Vec<KeyCode>,
}

impl Default for GrayboxPlugin {
    fn default() -> Self {
        Self { open_graybox_command: vec![KeyCode::SuperLeft, KeyCode::G] }
    }
}

#[derive(States, Default, PartialEq, Eq, Debug, Hash, Clone, Copy)]
pub enum GrayboxState {
    Opening,
    Open,
    #[default]
    Closed,
}

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum UIUpdateStages {
    Interactivity,
    ExamineInterfaces,
    UpdateData,
    UpdateInterfaces,
    CloseMenus,
}

#[derive(Resource)]
pub struct GrayboxSettings {
    pub font: Handle<Font>,
    pub open_graybox_command: Vec<KeyCode>,
}

impl Plugin for GrayboxPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GrayboxState>()
            .configure_sets(
                Update,
                (
                    UIUpdateStages::Interactivity,
                    UIUpdateStages::ExamineInterfaces,
                    UIUpdateStages::UpdateData,
                    UIUpdateStages::UpdateInterfaces,
                    UIUpdateStages::CloseMenus,
                ).chain().run_if(in_state(GrayboxState::Open)),
            )
            .add_systems(OnEnter(GrayboxState::Opening), open_ui)
            .add_systems(OnEnter(GrayboxState::Open), update_entity_panel)
            .add_systems(OnExit(GrayboxState::Open), close_ui)
            .add_systems(Update, (
                detect_open_command,
                (
                    (
                        entity_panel_ui,
                        timed_interaction::do_timed_interactions,
                    ).in_set(UIUpdateStages::Interactivity),
                    (
                        editable_selection,
                        (selection_indicator, sliders, sliders_text),
                    ).chain().in_set(UIUpdateStages::ExamineInterfaces),
                    text_input.in_set(UIUpdateStages::UpdateInterfaces),
                    inspector_ui.in_set(UIUpdateStages::CloseMenus),
                )
            ))
            .init_resource::<ModificationEvents>()
            .init_resource::<SelectedEditable>()
            .insert_resource(GrayboxSettings {
                font: Handle::default(),
                open_graybox_command: self.open_graybox_command.clone(),
            });
    }
}

#[derive(Component)]
pub struct GBoxOther;

#[derive(Component)]
pub struct UIParentNode;

#[derive(Component)]
pub struct Sidebar;

#[derive(Component)]
pub struct EntityPanel;

#[derive(Component)]
pub struct EntityNameButton {
    on: Entity,
}

#[derive(Component, Debug)]
pub struct Inspector {
    on: Entity,
}

#[derive(Component)]
pub struct InspectorCloseButton;

#[derive(Default, Component)]
pub struct TextInput {
    pub input: Option<String>,
    pub submitted: Option<String>,
}

#[derive(Component)]
pub struct EditableFloat {
    pub is_64_bit: bool,
}

#[derive(Component)]
pub struct Editable {
    val: Box<dyn Reflect>,
}

#[derive(Component)]
pub struct InspectorSection {
    on: ComponentId,
}

#[derive(Debug, Component)]
pub struct InspectorSubmenu {
    path_kind: PathKind,
}

pub type NoGboxFilter = (
    Without<GBoxOther>, 
    Without<EntityPanel>, 
    Without<UIParentNode>,
    Without<Inspector>,
    Without<EntityNameButton>,
    Without<Sidebar>,
    Without<InspectorSection>,
    Without<Editable>,
    Without<InspectorSubmenu>,
);

fn open_ui(mut commands: Commands, mut next_state: ResMut<NextState<GrayboxState>>) {
    let parent_node = commands.spawn((
        NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ..default()
        },
        UIParentNode,
    )).id();

    let sidebar = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                width: Val::Percent(30.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Stretch,
                ..default()
            },
            ..default()
        },
        Interaction::None,
        Sidebar,
    )).set_parent(parent_node).id();

    let _entity_panel = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(10.0)),
                flex_grow: 2.0,
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        },
        EntityPanel,
    )).set_parent(sidebar).id();

    next_state.set(GrayboxState::Open);
}

fn close_ui(parent_node: Query<Entity, With<UIParentNode>>, mut commands: Commands) {
    commands.entity(parent_node.single()).despawn_recursive();
}

fn update_entity_panel(
    mut commands: Commands, 
    entities: Query<(Entity, Option<&Name>), NoGboxFilter>,
    entity_panel: Query<Entity, With<EntityPanel>>,
    settings: Res<GrayboxSettings>,
) {
    let entity_panel = entity_panel.single();
    for (entity, name) in &entities {
        commands.spawn((
            TextBundle {
                text: Text::from_section( 
                    name.map(|name| name.as_str()).unwrap_or("<noname>"),
                    TextStyle {
                        font: settings.font.clone(),
                        ..default()
                    }
                ),
                ..default()
            },
            EntityNameButton { on: entity },
            Interaction::None,
        )).set_parent(entity_panel);
    }
}

fn detect_open_command(
    keyboard: Res<Input<KeyCode>>,
    settings: Res<GrayboxSettings>,
    mut next_state: ResMut<NextState<GrayboxState>>,
    state: Res<State<GrayboxState>>,
) {
    if settings.open_graybox_command.iter().all(|key| keyboard.pressed(*key)) && 
        settings.open_graybox_command.iter().any(|key| keyboard.just_pressed(*key)) {
        if *state == GrayboxState::Open {
            next_state.set(GrayboxState::Closed);
        } else {
            next_state.set(GrayboxState::Opening);
        }
    }
}

fn entity_panel_ui(
    mut entity_panels: Query<
        (&EntityNameButton, &Interaction, &mut BackgroundColor), 
        Changed<Interaction>
    >,
    sidebar: Query<(Entity, &Children), With<Sidebar>>,
    mut commands: Commands,
) {
    for mut entity_panel in &mut entity_panels {
        if *entity_panel.1 == Interaction::Hovered {
            *entity_panel.2 = Color::GRAY.into();
        } else {
            *entity_panel.2 = Color::DARK_GRAY.into();
        }

        if *entity_panel.1 == Interaction::Pressed {
            let sidebar = sidebar.single();
            spawn_inspector(&mut commands, entity_panel.0.on, sidebar.0); 
        }
    }
}

fn spawn_inspector(commands: &mut Commands, on: Entity, sidebar: Entity) {
    let inspector = commands.spawn((
        NodeBundle {
            style: Style {
                flex_grow: 3.0,
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(10.)),
                ..default()
            },
            background_color: Color::DARK_GRAY.into(),
            ..default()
        },
        Inspector { on, },
    )).set_parent(sidebar).id();
    

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(20.),
                height: Val::Px(20.),
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                right: Val::Px(0.),
                ..default()
            },
            background_color: Color::RED.into(),
            ..default()
        },
        InspectorCloseButton,
        Interaction::None,
    )).set_parent(inspector);
}

fn inspector_ui(
    mut inspector_close_buttons: Query<
        (&Interaction, &mut BackgroundColor, &Parent), 
        (With<InspectorCloseButton>, Changed<Interaction>),
    >,
    mut commands: Commands,
) {
    for mut close_button in &mut inspector_close_buttons {
        if *close_button.0 == Interaction::Pressed {
            commands.entity(**close_button.2).despawn_recursive();
        } else if *close_button.0 == Interaction::Hovered {
            *close_button.1 = Color::PINK.into();
        } else {
            *close_button.1 = Color::RED.into();
        }
    }
}
