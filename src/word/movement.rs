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
            let adjective = map.insert(PhraseData {
                word: None,
                kind: PhraseKind::Adjective,
            });
            let root = map.insert(PhraseData {
                word: None,
                kind: PhraseKind::Noun { adjective },
            });
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
}

pub fn do_movement(
    input: Res<Input<KeyCode>>,
    mut player: Query<PlayerQuery>,
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
    
    player.velocity.linvel.x = lerp(player.velocity.linvel.x, goal_speed, MOVE_X_ACC);
}
