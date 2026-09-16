use std::sync::Arc;
use std::fmt::Display;

use voicevox_core::StyleId;
use voicevox_core::{
    blocking::{OpenJtalk, Synthesizer, VoiceModelFile},
};

pub struct VVModelContext {
    pub synth: Synthesizer<OpenJtalk>,
    pub current_style: Option<CharacterStyle>,
    pub current_character: Option<Character>,
    pub characters: Vec<Character>,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub uuid: String,
    pub name: String,
    pub styles: Vec<CharacterStyle>,
}
#[derive(Debug, Clone)]
pub struct CharacterStyle {
    pub name: String,
    pub style_id: StyleId,
    pub vvm_ref: Arc<VoiceModelFile>,
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for CharacterStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

