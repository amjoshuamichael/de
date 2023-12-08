use crate::{prelude::*, word::{SentenceStructure, ui::SentenceSection, movement::Player, spawn::WordObject}};

use super::{WorldObject, player_spawner::PlayerSpawner, death_zone::DeathZone};

#[derive(Default, Component)]
pub struct CameraZone;

#[derive(Default, Bundle)]
pub struct CameraZoneBundle {
    word_tag: CameraZone,
    spatial: SpatialBundle,
    collider: Collider,
    colliding: CollidingEntities,
    rigidbody: RigidBody,
    events: ActiveEvents,
    sensor: Sensor,
    name: Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraZoneInWorld {
    pub transform: Transform,
}

impl WorldObject for CameraZone {
    type Bundle = CameraZoneBundle;
    type InWorld = CameraZoneInWorld;

    fn bundle(in_world: &CameraZoneInWorld, _: &MiscAssets) -> Self::Bundle {
        CameraZoneBundle {
            spatial: SpatialBundle::from_transform(in_world.transform),
            collider: Collider::cuboid(8., 8.),
            rigidbody: RigidBody::Fixed,
            events: ActiveEvents::all(),
            name: Name::new("Camera Zone"),
            ..default()
        }
    }
}

pub fn update(
    zones: Query<(&Collider, &GlobalTransform), With<CameraZone>>,
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut GameCamera>,
) {
    let player = player.single();

    let intersecting_zone = zones.iter()
        .map(|zone| {
            let col_extents = zone.0.as_cuboid().unwrap().raw.half_extents;
            let zone_pos = zone.1.translation().xy();

            Rect::from_corners(
                zone_pos + Vec2::from(col_extents),
                zone_pos - Vec2::from(col_extents),
            )
        })
        .find(|zone| zone.contains(player.translation.xy()))
        .unwrap_or_default();

    camera.single_mut().bounds = intersecting_zone;
}
