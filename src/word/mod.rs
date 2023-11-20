use bevy::{prelude::*, utils::HashMap};
use slotmap::*;

pub mod ui;

#[derive(Component, Default)]
pub struct WordControl {
    //wide: bool,
}

#[derive(Debug)]
pub enum WordID {
    Baby,
}

pub struct WordData {
    basic: String,
    // TODO: this would store more information about the word, like tenses, etc..
}

#[derive(Resource, Default)]
pub struct Words(HashMap<WordID, WordData>);

new_key_type! { pub struct PhraseID; }

#[derive(Debug)]
pub enum PhraseData {
    Noun {
        word: Option<WordID>,
        adjective: PhraseID,
    },
    Adjective {
        word: Option<WordID>,
    },
}

#[derive(Component)]
pub struct SentenceStructure {
    pub sentence: SlotMap<PhraseID, PhraseData>,
    pub root: PhraseID,
    pub ui_parent: Option<Entity>,
}

#[derive(Event)]
pub struct SentenceStructureChanged {
    pub on: Entity,
}
