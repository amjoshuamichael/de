use crate::{prelude::*, world::{LoadedLevel, helpers::level_is_in_position}};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_camera)
            .add_systems(Update, camera_update);
    }
}

#[derive(Default, Component)]
pub struct GameCamera {
    pub bounds: Rect,
    pub move_mode: CameraMoveMode,
}

#[derive(Default, PartialEq, Eq)]
pub enum CameraMoveMode {
    #[default]
    SnapToBounds,
    Free,
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
        GameCamera::default(),
        Name::new("Camera"),
    ));
}

fn camera_update(
    mut camera: Query<(&mut Transform, &OrthographicProjection, &mut GameCamera)>,
    player: Query<&Transform, (With<Player>, Without<OrthographicProjection>)>,
    levels: Query<(&LoadedLevel, &Transform), Without<GameCamera>>,
) {
    const CAMERA_SPEED: f32 = 0.1;
    let player = player.single();
    let mut camera = camera.single_mut();

    for level in &levels {
        if let Some(bounds) = level_is_in_position(level, player.translation.xy()) {
            camera.2.bounds = bounds;
        }
    }

    let target = match camera.2.move_mode {
        CameraMoveMode::SnapToBounds => {
            let mut target = player.translation.xy();
            target.x = target.x.max(camera.2.bounds.min.x - camera.1.area.min.x);
            target.x = target.x.min(camera.2.bounds.max.x - camera.1.area.max.x);
            target.y = target.y.max(camera.2.bounds.min.y - camera.1.area.min.y);
            target.y = target.y.min(camera.2.bounds.max.y - camera.1.area.max.y);
            target
        },
        CameraMoveMode::Free => {
            player.translation.xy()
        }
    };

    camera.0.translation = camera.0.translation.lerp(target.extend(0.), CAMERA_SPEED);
}
