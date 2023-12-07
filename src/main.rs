// allows for faster prototyping without warnings.
// in general, try to run cargo fix --allow-dirty before each commit.
#![allow(unused_imports)]

#![feature(let_chains)]
#![feature(exact_size_is_empty)]

// put a use crate::prelude::* at the top of every file
pub mod prelude {
    #![allow(ambiguous_glob_reexports)]
    pub use bevy::{prelude::*, utils::{HashMap, HashSet}, 
        ecs::{query::WorldQuery, system::SystemParam}};
    pub use bevy_rapier2d::prelude::*;
    pub use slotmap::*;
    pub use serde::*;
    pub use graybox::*;

    pub use super::load_assets::MiscAssets;
    pub use super::word::*;

    pub const CONTROL_KEY: KeyCode = 
        if cfg!(windows) { KeyCode::ControlLeft } else { KeyCode::SuperLeft };

    pub fn lerp(a: f32, b: f32, n: f32) -> f32 {
        debug_assert!(n >= 0. && n <= 1.);
        a * (1. - n) + b * n
    }
}

use bevy::window::CompositeAlphaMode;
use prelude::*;

mod word;
mod world;
mod load_assets;
mod helpers;
mod frame_stop;

use word::*;
use load_assets::*;
use world::*;
pub use helpers::*;
use frame_stop::*;

use leafwing_input_playback::{input_capture::*, input_playback::*, serde::*};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(bevy::asset::AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    ..default()
                 })
                 .set(WindowPlugin {
                     primary_window: Some(Window {
                         decorations: false,
                         ..default()
                     }),
                     ..default()
                 }),
            word::PlayerPlugin,
            load_assets::AssetPlugin,
            world::WorldPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin { enabled: false, ..default() },
            GrayboxPlugin::default(),
            FrameStopPlugin,
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (
            optional_debug_physics_view,
            camera_update,
        ))
        .enable_inspection::<Transform>()
        .enable_inspection::<Velocity>()
        .insert_resource(Msaa::Off) // disable anti-aliasing, this is a pixel game
        .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
        .add_plugins(InputCapturePlugin)
        .insert_resource(PlaybackFilePath::new("input_records/record.ron"))
        .insert_resource(InputModesCaptured::ENABLE_ALL)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scale: 0.25,
                ..default()
            },
            ..default()
        },
        Name::new("Camera"),
    ));
}


fn camera_update(
    mut camera: Query<&mut Transform, With<Camera2d>>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    const CAMERA_SPEED: f32 = 0.1;
    let player = player.single();
    let mut camera = camera.single_mut();
    camera.translation = camera.translation.lerp(player.translation, CAMERA_SPEED);
}

fn optional_debug_physics_view(
    keyboard: Res<Input<KeyCode>>,
    mut physics_debug_context: ResMut<DebugRenderContext>,
) {
    if keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::D) {
        physics_debug_context.enabled = !physics_debug_context.enabled;
    }
}
