use crate::prelude::*;

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

    pub fn part_of_speech(self) -> &'static [PartOfSpeech] {
        use PartOfSpeech::*;

        match self {
            WordID::Baby => &[Noun, Adjective],
            WordID::Wide => &[Adjective],
            WordID::Tall => &[Adjective],
            WordID::Fast => &[Adjective],
            WordID::Horse => &[Noun],
            WordID::And => &[Conjuction],
            WordID::FlutteringUp => &[Adjective],
            WordID::FlutteringRight => &[Adjective],
        }
    }
}

#[derive(Debug, Default)]
pub struct WordForms {
    pub basic: &'static str,
    pub filename: &'static str,
    pub tag_handle: Handle<Image>,
}

#[derive(PartialEq)]
pub enum PartOfSpeech {
    Noun,
    Adjective,
    Conjuction,
}
