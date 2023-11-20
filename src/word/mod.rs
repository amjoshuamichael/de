use bevy::{prelude::*, utils::HashMap};
use slotmap::*;

pub mod ui;

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

#[derive(Debug, Component)]
pub struct SentenceStructure {
    pub sentence: SlotMap<PhraseID, PhraseData>,
    pub root: PhraseID,
}

#[derive(Event)]
pub struct SentenceStructureChanged {
    pub on: Entity,
}
