// allows for faster prototyping without warnings.
// in general, try to run cargo fix --allow-dirty before each commit.
#![allow(unused_variables)]
#![allow(unused_imports)]

use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;
use slotmap::*;

mod word;
mod world;
mod load_assets;

use word::*;
use load_assets::*;
use world::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(bevy::asset::AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    ..default()
                 }),
            word::ui::UIPlugin,
            load_assets::AssetPlugin,
            world::WorldPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin { enabled: false, ..default() },
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            optional_debug_physics_view,
            remake_player_character,
            camera,
        ))
        .add_systems(FixedUpdate, movement)
        .insert_resource(Msaa::Off) // disable anti-aliasing, this is a pixel game
        .insert_resource::<Words>(Words({
            let mut map = HashMap::new();
            map.insert(WordID::Baby, WordData { basic: "Baby".into(), });
            map.insert(WordID::Wide, WordData { basic: "Wide".into(), });
            map.insert(WordID::Tall, WordData { basic: "Tall".into(), });
            map
        }))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far: 1000.,
            near: -1000.,
            scale: 0.25,
            ..default()
        },
        ..default()
    });

    let mut map = SlotMap::<PhraseID, PhraseData>::with_key();
    let adjective = map.insert(PhraseData {
        word: None,
        kind: PhraseKind::Adjective,
    });
    let root = map.insert(PhraseData {
        word: None,
        kind: PhraseKind::Noun { adjective },
    });
    commands.spawn(SentenceStructure {
        sentence: map,
        root,
    });

    commands.spawn((
        PlayerBundle {
            transform: Transform {
                translation: Vec3::new(1.0, 4.0, 0.0) * 16.0,
                ..default()
            },
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(16.0, 16.0),
    ));
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

fn movement(
    input: Res<Input<KeyCode>>,
    mut movers: Query<&mut Transform, With<Player>>,
) {
    const MOVE_X_SPEED: f32 = 2.0;
    for mut mover in &mut movers {
        if input.pressed(KeyCode::D) {
            mover.translation.x += MOVE_X_SPEED;
        } else if input.pressed(KeyCode::A) {
            mover.translation.x -= MOVE_X_SPEED;
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct WordObjectBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

fn remake_player_character(
    mut structure_change_evt: EventReader<SentenceStructureChanged>,
    sentences: Query<&SentenceStructure>,
    current_players: Query<(&Transform, Entity), With<Player>>,
    mut commands: Commands,
    assets: Res<DeAssets>,
) {
    // TODO: make this work for multiple players
    let player = current_players.single();

    for change in structure_change_evt.read() {
        commands.entity(player.1).despawn_descendants();
        let sentence = sentences.get(change.on).unwrap();
        
        spawn_with_noun(sentence.root, &sentence, &mut commands, &*assets, player);
    }
}

fn spawn_with_noun(
    word: PhraseID,
    sentence: &SentenceStructure,
    commands: &mut Commands,
    assets: &DeAssets,
    player_parent: (&Transform, Entity),
) {
    match &sentence.sentence[sentence.root] {
        PhraseData { word: None, .. } => {},
        PhraseData { word: Some(word), kind: PhraseKind::Noun { adjective }} => {
            let mut bundle = WordObjectBundle::default();

            modify_with_adjective(*adjective, &sentence, &mut bundle, &*assets);

            match word {
                WordID::Baby => {
                    bundle.texture = assets.square_pale.clone();
                    //bundle.transform = *player_parent.0;
                    commands.spawn(bundle).set_parent(player_parent.1);
                },
                _ => {},
            }
        }
        _ => todo!(),
    }
}

fn modify_with_adjective(
    word: PhraseID,
    sentence: &SentenceStructure,
    bundle: &mut WordObjectBundle,
    assets: &DeAssets,
) {
    match &sentence.sentence[word] {
        PhraseData { word: None, .. } => {}
        PhraseData { word: Some(word), kind: PhraseKind::Adjective } => {
            match word {
                WordID::Wide => 
                    { bundle.transform.scale.x = 4.0; },
                WordID::Tall => 
                    { bundle.transform.scale.y = 4.0; },
                _ => todo!(),
            }
        }
        _ => {},
    }
}

fn camera(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    const CAMERA_SPEED: f32 = 0.1;
    let player = player.single();
    let mut camera = camera.single_mut();
    camera.translation = camera.translation.lerp(player.translation, CAMERA_SPEED);
}

pub const CONTROL_KEY: KeyCode = 
    if cfg!(windows) { KeyCode::ControlLeft } else { KeyCode::SuperLeft };

fn optional_debug_physics_view(
    keyboard: Res<Input<KeyCode>>,
    mut physics_debug_context: ResMut<DebugRenderContext>,
) {
    if keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::D) {
        physics_debug_context.enabled = !physics_debug_context.enabled;
    }
}
