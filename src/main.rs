use bevy::prelude::*;

mod word_ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            word_ui::UIPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, movement)
        .run();
}

#[derive(Component, Default)]
pub struct WordControl {
    wide: bool,
}

#[derive(Component)]
pub struct Movement;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("square_pale.bmp"),
            transform: Transform {
                scale: Vec3::ONE * 8.0,
                ..default()
            },
            ..default()
        },
        WordControl::default(),
        Movement,
    ));
}

fn movement(
    input: Res<Input<KeyCode>>,
    mut movers: Query<&mut Transform, With<Movement>>,
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
