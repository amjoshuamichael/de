// The word module handles everything related to words, spawning them in the world, and
// applying their effects.

use crate::prelude::*;

pub mod ui;
pub mod movement;
pub mod spawn;
pub mod apply_words;
pub mod word_id;

use bevy::{utils::HashSet, ecs::schedule::ScheduleLabel};
pub use movement::*;
pub use word_id::*;

use self::{ui::*, spawn::SentenceSpawn};

pub struct PlayerPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct SentenceModificationRoutine;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PostSentenceModificationActionsSet;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, 
                 movement::spawn_player.pipe(ui::setup_word_ui),
            )
            .add_event::<SentenceUIChanged>()
            .add_event::<SentenceStructureChanged>()
            .add_event::<SentenceSpawn>()
            .add_event::<VocabChange>()
            .add_systems(Update, (
                // sentence ui / word remake routine
                (
                    ui::regenerate_sentence_structure,
                    ( 
                        ui::do_snap,
                        ui::do_unsnap, 
                        ui::do_drag, 
                    ).chain(),
                    spawn::remake_player_character,
                    spawn::disable_physics_for_invalid_sentence_structures,
                ).in_set(SentenceModificationRoutine).chain(),
                ui::update_vocabulary,
            ))
            .add_systems(
                // these run deffered, after the node spawn commands issued by
                // update_sentence_ui.
                PostSentenceModificationActionsSet, 
                (ui::indicate_sentence_section_locks, ui::reorder_sentence_ui),
            )
            .add_systems(FixedUpdate, (
                apply_words::apply_scalers,
                apply_words::apply_fluttering,
            ).after(SentenceModificationRoutine))
            .add_systems(Update, (
                movement::do_movement,
            ));
    }
}

#[derive(Component, Default)]
pub struct WordControl {
    //wide: bool,
}

#[derive(Component, Default)]
pub struct Vocabulary {
    words: HashSet<WordID>,
}

new_key_type! { pub struct PhraseID; }

#[derive(Copy, Clone, Debug, Default)]
pub struct PhraseData {
    pub word: Option<WordID>,
    pub kind: PhraseKind,
    pub locked: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum PhraseKind {
    Noun { 
        adjective: PhraseID,
    },
    #[default]
    Adjective,
    Combine {
        l: PhraseID,
        r: PhraseID,
    },
}

pub type PhraseMap = SlotMap<PhraseID, PhraseData>;

/// Components that act as the parent of a word collection. For example, the player has a
/// SentenceStructure.
#[derive(Debug, Component)]
pub struct SentenceStructure {
    pub sentence: PhraseMap,
    pub root: PhraseID,
    pub valid: bool,
}

#[derive(Event)]
pub struct SentenceStructureChanged {
    pub on: Entity,
}
