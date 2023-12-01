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
        Velocity::default(),
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
}

pub fn do_movement(
    input: Res<Input<KeyCode>>,
    mut player: Query<PlayerQuery>,
) {
    const MOVE_X_SPEED: f32 = 64.0;
    const MOVE_X_ACC: f32 = 0.1;
    let mut player = player.single_mut();

    if !player.word_object.valid { return }

    let goal_speed = if input.pressed(KeyCode::D) {
        MOVE_X_SPEED
    } else if input.pressed(KeyCode::A) {
        -MOVE_X_SPEED
    } else {
        0.
    };

    player.velocity.linvel.x = lerp(player.velocity.linvel.x, goal_speed, MOVE_X_ACC);
}

