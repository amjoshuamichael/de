use crate::prelude::*;

pub mod ui;
pub mod movement;
pub mod spawn;

pub use movement::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, ui::setup_word_ui)
            .add_event::<SentenceStructureChanged>()
            .add_systems(Update, (
                ui::update_sentence_ui,
                ui::update_word_ui,
                ui::do_unsnap,
                ui::do_drag.after(ui::do_unsnap),
                ui::do_snap.after(ui::do_drag),
            ))
            .add_systems(Update, (
                spawn::remake_player_character,
                spawn::deactivate_inactive_sentence_structures.after(spawn::remake_player_character),
            ))
            .add_systems(Startup, movement::spawn_player)
            .add_systems(Update, movement::do_movement);
    }
}

#[derive(Component, Default)]
pub struct WordControl {
    //wide: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum WordID {
    Baby,
    Wide,
    Tall,
}

pub struct WordData {
    pub basic: String,
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
    pub active: bool,
}

#[derive(Event)]
pub struct SentenceStructureChanged {
    pub on: Entity,
}
