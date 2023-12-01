// The word module handles everything related to words, spawning them in the world, and
// applying their effects.

use crate::prelude::*;

pub mod ui;
pub mod movement;
pub mod spawn;
pub mod apply_words;

use bevy::utils::HashSet;
pub use movement::*;

use self::ui::VocabChange;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, ui::setup_word_ui)
            .add_event::<SentenceStructureChanged>()
            .add_event::<VocabChange>()
            .add_systems(Update, (
                (
                    ui::update_vocabulary,
                    ui::update_sentence_ui,
                    ui::update_word_ui,
                ).chain(),
                (
                    ui::do_unsnap,
                    ui::do_drag,
                    ui::do_snap,
                ).chain(),
            ))
            .add_systems(Update, (
                spawn::remake_player_character,
                spawn::deactivate_inactive_sentence_structures,
            ).chain())
            .add_systems(FixedUpdate, (
                apply_words::apply_wide,
                apply_words::apply_tall,
            ))
            .add_systems(Startup, movement::spawn_player)
            .add_systems(Update, movement::do_movement);
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
}

#[derive(Debug, Default)]
pub struct WordData {
    pub basic: &'static str,
    pub tag_handle: Handle<Image>,
    // TODO: this would store more information about the word, like tenses, etc..
}

#[derive(Resource, Default)]
pub struct Words(pub HashMap<WordID, WordData>);

new_key_type! { pub struct PhraseID; }

#[derive(Debug)]
pub struct PhraseData {
    pub word: Option<WordID>,
    pub kind: PhraseKind,
}

#[derive(Debug)]
pub enum PhraseKind {
    Noun { 
        adjective: PhraseID,
    },
    Adjective,
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
