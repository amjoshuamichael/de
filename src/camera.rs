use crate::prelude::*;

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
    mut camera: Query<(&mut Transform, &OrthographicProjection, &GameCamera)>,
    player: Query<&Transform, (With<Player>, Without<OrthographicProjection>)>,
) {
    const CAMERA_SPEED: f32 = 0.1;
    let player = player.single();
    let mut camera = camera.single_mut();
    camera.0.translation = camera.0.translation.lerp(player.translation, CAMERA_SPEED);

    if camera.2.move_mode == CameraMoveMode::SnapToBounds {
        let campos = &mut camera.0.translation;
        campos.x = campos.x.max(camera.2.bounds.min.x - camera.1.area.min.x);
        campos.x = campos.x.min(camera.2.bounds.max.x - camera.1.area.max.x);
        campos.y = campos.y.max(camera.2.bounds.min.y - camera.1.area.min.y);
        campos.y = campos.y.min(camera.2.bounds.max.y - camera.1.area.max.y);
    }
}
