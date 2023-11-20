#![allow(unused)] // allows for faster prototyping without warnings
use bevy::{prelude::*, utils::HashMap};
use slotmap::*;

mod word;
use word::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            word::ui::UIPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, movement)
        .init_resource::<Words>()
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut map = SlotMap::<PhraseID, PhraseData>::with_key();
    let adjective = map.insert(PhraseData::Adjective {
        word: None,
    });
    let root = map.insert(PhraseData::Noun {
        word: None,
        adjective,
    });
    commands.spawn(SentenceStructure {
        sentence: map,
        root,
        ui_parent: None,
    });
}

fn movement(
    input: Res<Input<KeyCode>>,
    mut movers: Query<&mut Transform, With<SentenceStructure>>,
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
