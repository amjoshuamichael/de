use crate::prelude::*;

use super::{PhraseData, PhraseKind, PhraseID, SentenceStructure, Vocabulary, WordID, ui::VocabChange};

#[derive(Component, Default)]
pub struct Player;

pub fn spawn_player(
    mut commands: Commands,
    mut vocab_changes: EventWriter<VocabChange>,
) {
    let player = commands.spawn((
        Player,
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(50.0, 50.0, 0.0) * 16.0,
                ..default()
            },
            ..default()
        },
        RigidBody::default(),
        AdditionalMassProperties::Mass(10.0),
        ReadMassProperties::default(),
        Velocity::default(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        LockedAxes::ROTATION_LOCKED,
        {
            let mut map = SlotMap::<PhraseID, PhraseData>::with_key();
            let adjective = map.insert(
                PhraseData { kind: PhraseKind::Adjective, ..default() });
            let root = map.insert(
                PhraseData { kind: PhraseKind::Noun { adjective }, ..default() });
            SentenceStructure {
                sentence: map,
                root,
                valid: false,
            }
        },
        Vocabulary::default(),
        Name::new("Player"),
    )).id();

    vocab_changes.send(VocabChange::Added {
        word: WordID::Baby,
        to: player,
    });
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct PlayerQuery {
    player: &'static Player,
    word_object: &'static SentenceStructure,
    transform: &'static mut Transform,
    velocity: &'static mut Velocity,
    force: &'static mut ExternalForce,
    mass: &'static ReadMassProperties,
    entity: Entity,
}

pub fn do_movement(
    input: Res<Input<KeyCode>>,
    mut player: Query<PlayerQuery>,
    colliders: Query<(&GlobalTransform, &Collider)>,
    children: Query<&Children>,
    sensors: Query<&Sensor>,
    phys_context: Res<RapierContext>,
) {
    const MAX_X_SPEED: f32 = 32000.0;
    const MOVE_X_ACC: f32 = 0.1;
    let mut player = player.single_mut();

    if !player.word_object.valid { return }

    let max_speed = MAX_X_SPEED / player.mass.mass;
    let goal_speed = if input.pressed(KeyCode::D) {
        max_speed
    } else if input.pressed(KeyCode::A) {
        -max_speed
    } else {
        0.
    };
    
    let colliders: Vec::<(Entity, (&GlobalTransform, &Collider))> =
        children.iter_descendants(player.entity).filter_map(|collider_entity| {
            Some((collider_entity, colliders.get(collider_entity).ok()?))
        }).collect();

    let newvel = lerp(player.velocity.linvel.x, goal_speed, MOVE_X_ACC);

    let mut is_colliding = false;

    for (_, collider) in &colliders {
        let (scale, rotation, translation) = collider.0.to_scale_rotation_translation();

        let shrunk_collider = 
            collider.1.as_typed_shape().raw_scale_by(Vec2::splat(0.99), 2).unwrap();

        let intersection = phys_context.intersection_with_shape(
            translation.xy() + Vec2::new(newvel, 0.).normalize_or_zero() * 0.1 * scale.x,
            rotation.z,
            &Collider::from(shrunk_collider),
            QueryFilter {
                predicate: Some(&|entity| 
                    colliders.iter().any(|(c, _)| *c != entity) &&
                    sensors.get(entity).is_err()
                ),
                ..default()
            },
        );

        if intersection.is_some() {
            is_colliding = true;
        }
    }

    if !is_colliding {
        player.velocity.linvel.x = newvel;
    }
}
