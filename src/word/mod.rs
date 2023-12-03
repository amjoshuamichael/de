// The word module handles everything related to words, spawning them in the world, and
// applying their effects.

use crate::prelude::*;

pub mod ui;
pub mod movement;
pub mod spawn;
pub mod apply_words;

use bevy::utils::HashSet;
pub use movement::*;

use self::{ui::*, spawn::SentenceSpawn};

pub struct PlayerPlugin;

#[derive(SystemSet, Hash, PartialEq, Eq, Debug, Clone)]
pub struct SentenceModificationRoutine;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, ui::setup_word_ui)
            .add_event::<SentenceUIChanged>()
            .add_event::<SentenceStructureChanged>()
            .add_event::<SentenceSpawn>()
            .add_event::<VocabChange>()
            .add_systems(Update, (
                (
                    ui::do_unsnap,
                    ui::do_drag,
                    ui::do_snap,
                    ui::sentence_section_docks.run_if(on_event::<SentenceUIChanged>()),
                    ui::update_sentence_ui,
                    ui::indicate_sentence_section_locks,
                    ui::reorder_sentence_ui,
                    spawn::remake_player_character,
                    spawn::disable_physics_for_invalid_sentence_structures,
                ).in_set(SentenceModificationRoutine).chain(),
                (
                    ui::update_vocabulary,
                    ui::words_init,
                ).chain(),
            ))
            .add_systems(FixedUpdate, (
                apply_words::apply_wide,
                apply_words::apply_tall,
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

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum WordID {
    #[default]
    Baby,
    Wide,
    Tall,
    Horse,
    And,
    FlutteringUp,
    FlutteringRight,
}

#[derive(Debug, Default)]
pub struct WordData {
    pub basic: &'static str,
    pub filename: &'static str,
    pub tag_handle: Handle<Image>,
}

impl WordData {
    pub fn new(basic: &'static str, filename: &'static str) -> Self {
        WordData { basic, filename, tag_handle: Handle::default() }
    }
}

#[derive(Resource, Default)]
pub struct Words(pub HashMap<WordID, WordData>);

new_key_type! { pub struct PhraseID; }

#[derive(Copy, Clone, Debug)]
pub struct PhraseData {
    pub word: Option<WordID>,
    pub kind: PhraseKind,
}

impl PhraseData {
    pub fn kind(kind: PhraseKind) -> Self {
        Self { word: None, kind }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PhraseKind {
    Noun { 
        adjective: PhraseID,
    },
    Adjective,
    CombineAdjectives {
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
