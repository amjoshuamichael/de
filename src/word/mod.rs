// The word module handles everything related to words, spawning them in the world, and
// applying their effects.

use crate::prelude::*;

pub mod ui;
pub mod movement;
pub mod spawn;
pub mod apply_words;

use bevy::{utils::HashSet, ecs::schedule::ScheduleLabel};
pub use movement::*;

use self::{ui::*, spawn::SentenceSpawn};

pub struct PlayerPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct SentenceModificationRoutine;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
pub struct PostSentenceModificationActionsSet;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, ui::setup_word_ui)
            .add_event::<SentenceUIChanged>()
            .add_event::<SentenceStructureChanged>()
            .add_event::<SentenceSpawn>()
            .add_event::<VocabChange>()
            .add_systems(Update, (
                // sentence ui / word remake routine
                (
                    (ui::do_unsnap, ui::do_drag, ui::do_snap),
                    ui::dock_words_in_sentence_sections
                        .run_if(on_event::<SentenceUIChanged>()),
                    ui::update_sentence_ui,
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
            .add_systems(Startup, movement::spawn_player)
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

pub const ALL_WORDS: [WordID; 8] = [WordID::Baby, WordID::Wide, WordID::Tall, WordID::Fast, WordID::Horse,
    WordID::And, WordID::FlutteringUp, WordID::FlutteringRight];

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WordID {
    #[default]
    Baby,
    Wide,
    Tall,
    Fast,
    Horse,
    And,
    FlutteringUp,
    FlutteringRight,
}

impl WordID {
    pub fn forms(self) -> WordForms {
        fn word(basic: &'static str, filename: &'static str) -> WordForms {
            WordForms { basic, filename, tag_handle: Handle::default() }
        }

        match self {
            WordID::Baby => word("Baby", "baby"),
            WordID::Wide => word("Wide", "wide"),
            WordID::Tall => word("Tall", "tall"),
            WordID::Fast => word("Fast", "fast"),
            WordID::Horse => word("Horse", "horse"),
            WordID::And => word("And", "and"),
            WordID::FlutteringUp => word("Fluttering", "fluttering_up"),
            WordID::FlutteringRight => word("Fluttering", "fluttering_right"),
        }
    }
}

#[derive(Debug, Default)]
pub struct WordForms {
    pub basic: &'static str,
    pub filename: &'static str,
    pub tag_handle: Handle<Image>,
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

/// Components that act as the parent of a word collection. For example, the player has a
/// SentenceStructure.
#[derive(Debug, Component)]
pub struct SentenceStructure {
    pub sentence: SlotMap<PhraseID, PhraseData>,
    pub root: PhraseID,
    pub valid: bool,
}

#[derive(Event)]
pub struct SentenceStructureChanged {
    pub on: Entity,
}
