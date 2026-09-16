use crate::Character;
use crate::CharacterStyle;
use std::error::Error;
use std::fs::{self};
use std::sync::Arc;
use std::{env, option::Option};

use voicevox_core::{
    CharacterMeta, StyleMeta,
    blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
};

pub struct VVModelContext {
    pub synth: Synthesizer<OpenJtalk>,
    pub current_style: Option<CharacterStyle>,
    pub current_character: Option<Character>,
    pub characters: Vec<Character>,
}

fn current_exe_tree(path: &str) -> String {
    format!(
        "{}{}",
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_str()
            .unwrap(),
        path
    )
}

impl VVModelContext {
    pub fn new(path_to_dylib: String, ojt_dic_dir: String) -> Result<Self, Box<dyn Error>> {

        let vvm_files = {
            let dir_reader = fs::read_dir(current_exe_tree("/voicevox_core/models/vvms"))?
                .collect::<Result<Vec<_>, _>>()?;
            dir_reader.into_iter().filter(|item| {
                item.path().is_file() && item.path().extension().is_some_and(|i| i == "vvm")
            })
        };

        let vvms = {
            let mut vvms = vec![];
            for vvm in vvm_files {
                vvms.push(Arc::new(VoiceModelFile::open(
                    vvm.path()
                        .to_str()
                        .ok_or("vvm path is not generated correctly")?,
                )?));
            }
            vvms
        };

        let synth = {
            let ort = Onnxruntime::load_once().filename(path_to_dylib).perform()?;
            let ojt = OpenJtalk::new(ojt_dic_dir).unwrap();
            Synthesizer::builder(ort)
                .acceleration_mode(voicevox_core::AccelerationMode::Auto)
                .text_analyzer(ojt)
                .build()?
        };

        let characters: Vec<Character> = {
            let mut characters: Vec<Character> = vec![];
            for vvm in &vvms {
                for meta in vvm.metas() {
                    match characters
                        .iter_mut()
                        .find(|Character { uuid, .. }| uuid == &meta.speaker_uuid)
                    {
                        Some(a) => {
                            for s in &meta.styles {
                                a.styles.push(CharacterStyle {
                                    name: s.name.clone(),
                                    style_id: s.id,
                                    vvm_ref: Arc::clone(vvm),
                                })
                            }
                        }
                        None => {
                            characters.push(Character {
                                uuid: meta.speaker_uuid.clone(),
                                name: meta.name.clone(),
                                styles: {
                                    let mut styles: Vec<CharacterStyle> = vec![];
                                    for s in &meta.styles {
                                        styles.push(CharacterStyle {
                                            name: s.name.clone(),
                                            style_id: s.id,
                                            vvm_ref: Arc::clone(vvm),
                                        })
                                    }
                                    styles
                                },
                            });
                        }
                    };
                }
            }
            characters
        };

        dbg!(synth.is_gpu_mode());

        Ok(Self {
            synth,
            current_style: None,
            current_character: None,
            characters,
        })
    }

    pub fn tts(&self, text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let current_character = self.current_character.as_ref().ok_or("")?;
        let current_style = self
            .current_style
            .as_ref()
            .ok_or("Style is not Specified!")?;

        let file = &Arc::clone(&current_style.vvm_ref);
        if !self.synth.is_loaded_voice_model(file.id()) {
            self.synth.load_voice_model(file)?;
        }

        let StyleMeta { id: style_id, .. } = self
            .synth
            .metas()
            .into_iter()
            .filter(
                |CharacterMeta {
                     name, speaker_uuid, ..
                 }| {
                    name == current_character.name.as_str()
                        && speaker_uuid == current_character.uuid.as_str()
                },
            )
            .flat_map(|CharacterMeta { styles, .. }| styles)
            .find(|StyleMeta { name, id, .. }| {
                name == current_style.name.as_str() && id == &current_style.style_id
            })
            .ok_or("style not found")?;

        Ok(self.synth.tts(text, style_id).perform()?)
    }
}
