// allows for faster prototyping without warnings.
// in general, try to run cargo fix --allow-dirty before each commit.
#![allow(unused_imports)]

#![feature(let_chains)]
#![feature(exact_size_is_empty)]

// put a use crate::prelude::* at the top of every file
pub(crate) mod prelude {
    #![allow(ambiguous_glob_reexports)]
    pub use bevy::{prelude::*, utils::{HashMap, HashSet}, 
        ecs::{query::WorldQuery, system::SystemParam}};
    pub use bevy_rapier2d::prelude::*;
    pub use slotmap::*;
    pub use itertools::Itertools;
    pub use serde::*;
    pub use graybox::*;
    pub use grid::*;

    pub use super::load_assets::MiscAssets;
    pub use super::word::*;
    pub use super::camera::*;

    pub const CONTROL_KEY: KeyCode = 
        if cfg!(windows) { KeyCode::ControlLeft } else { KeyCode::SuperLeft };

    pub fn lerp(a: f32, b: f32, n: f32) -> f32 {
        debug_assert!(n >= 0. && n <= 1.);
        a * (1. - n) + b * n
    }

    pub fn reflect(dir: Vec2, mirror: Vec2) -> Vec2 {
        // https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
        dir - 2. * (dir.dot(mirror)) * mirror
    }

    use std::ops::*;
    pub fn avg<T: Add<Output = T> + Div<f32, Output = T>>(a: T, b: T) -> T {
        (a + b) / 2.
    }

    pub fn empty_grid<T: Default>() -> Grid<T> { Grid::new(0, 0) }

    pub fn grid_size<T>(grid: &Grid<T>) -> Vec2 {
        Vec2::new(grid.rows() as f32, grid.cols() as f32)
    }
}

use bevy::window::CompositeAlphaMode;
use prelude::*;

mod word;
mod world;
mod load_assets;
mod helpers;
mod frame_stop;
mod camera;

use word::*;
use load_assets::*;
use world::*;
pub use helpers::*;
use frame_stop::*;
use camera::*;

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
                         mode: bevy::window::WindowMode::Windowed,
                         decorations: true,
                         ..default()
                     }),
                     ..default()
                 }),
            word::PlayerPlugin,
            load_assets::AssetPlugin,
            world::WorldPlugin,
            camera::CameraPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin { enabled: false, ..default() },
            GrayboxPlugin {
                open_graybox_command: vec![CONTROL_KEY, KeyCode::G],
            },
            FrameStopPlugin,
        ))
        .add_systems(Update, optional_debug_physics_view)
        .enable_inspection::<Transform>()
        .enable_inspection::<Velocity>()
        .insert_resource(Msaa::Off) // disable anti-aliasing, this is a pixel game
        .insert_resource(ClearColor(Color::ANTIQUE_WHITE))
        .add_plugins(InputCapturePlugin)
        .insert_resource(PlaybackFilePath::new("input_records/record.ron"))
        .insert_resource(InputModesCaptured::ENABLE_ALL)
        .run();
}


fn optional_debug_physics_view(
    keyboard: Res<Input<KeyCode>>,
    mut physics_debug_context: ResMut<DebugRenderContext>,
) {
    if keyboard.pressed(CONTROL_KEY) && keyboard.just_pressed(KeyCode::D) {
        physics_debug_context.enabled = !physics_debug_context.enabled;
    }
}
