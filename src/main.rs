use std::error::Error;
use std::fmt::Display;
use std::fs::{self};
use std::io::Cursor;
use std::sync::Arc;
use std::{env, fs::File, io::Write};

use iced::widget::{Column, button, column, combo_box, row, text, text_input};
use iced::{Alignment, Theme};
use voicevox_core::StyleId;
use voicevox_core::{
    CharacterMeta, StyleMeta,
    blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
};

const APP_NAME: &str = "VOICEVOX Rust GUI";

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

fn main() -> iced::Result {
    let path_to_dylib = current_exe_tree(
        format!(
            "/voicevox_core/onnxruntime/lib/{}",
            Onnxruntime::LIB_VERSIONED_FILENAME
        )
        .as_str(),
    );
    let ojt_dic_dir = current_exe_tree("/voicevox_core/dict/open_jtalk_dic_utf_8-1.11");

    iced::application(
        move || IcedVVGUIState::new(path_to_dylib.clone(), ojt_dic_dir.clone()),
        IcedVVGUIState::update,
        IcedVVGUIState::view,
    )
    .theme(Theme::Dark)
    .title(APP_NAME)
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    TextEdited(String),
    TTSBtnPressed,
    SayBtnPressed,
    CharacterSelected(Character),
    CharacterStyleSelected(CharacterStyle),
}

struct IcedVVGUIState {
    model_context: VVModelContext,
    current_text: String,
    characters: combo_box::State<Character>,
    current_character: Option<Character>,
    current_character_styles: combo_box::State<CharacterStyle>,
    current_character_style: Option<CharacterStyle>,
}

impl IcedVVGUIState {
    fn new(path_to_dylib: String, ojt_dic_dir: String) -> Self {
        let model_context = VVModelContext::new(path_to_dylib, ojt_dic_dir).unwrap();
        let state = combo_box::State::new(model_context.characters.clone());
        let state_styles = combo_box::State::new(vec![]);

        Self {
            model_context,
            current_text: String::default(),
            characters: state,
            current_character: None,
            current_character_styles: state_styles,
            current_character_style: None,
        }
    }

    fn update(state: &mut IcedVVGUIState, message: Message) {
        match message {
            Message::TextEdited(text) => {
                state.current_text = text;
            }
            Message::TTSBtnPressed => {
                let wav = state.model_context.tts(&state.current_text).unwrap();
                let mut file = File::create(format!(
                    "{}_{}.wav",
                    state.current_character.as_ref().ok_or("").unwrap().name,
                    state.current_text
                ))
                .unwrap();
                file.write_all(&wav).unwrap()
            }
            Message::SayBtnPressed => {
                let wav = state.model_context.tts(&state.current_text).unwrap();
                let wav = Cursor::new(wav);
                let sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
                let player = rodio::play(sink_handle.mixer(), wav).unwrap();
                player.sleep_until_end();
            }
            Message::CharacterSelected(character) => {
                state.current_character = Some(character.clone());
                state.model_context.current_character = state.current_character.clone();
                state.current_character_styles = combo_box::State::new(character.styles.clone());
                state.current_character_style = None;
            }
            Message::CharacterStyleSelected(style) => {
                state.current_character_style = Some(style.clone());
                state.model_context.current_style = state.current_character_style.clone();
            }
        }
    }

    fn view<'a>(state: &'a IcedVVGUIState) -> Column<'a, Message> {
        column![
            text(format!(
                "以下の文字列が音声合成され保存されます\n{}",
                &state.current_text
            )),
            row![
                column![
                    combo_box(
                        &state.characters,
                        "Select Characters",
                        state.current_character.as_ref(),
                        Message::CharacterSelected
                    ),
                    combo_box(
                        &state.current_character_styles,
                        "Select Characters",
                        state.current_character_style.as_ref(),
                        Message::CharacterStyleSelected,
                    )
                ],
                text_input("Input text...", &state.current_text).on_input(Message::TextEdited),
            ],
            row![
                button("TTS & Save!").on_press(Message::TTSBtnPressed),
                button("Say").on_press(Message::SayBtnPressed)
            ]
        ]
        .align_x(Alignment::Center)
    }
}

#[derive(Debug, Clone)]
struct Character {
    uuid: String,
    name: String,
    styles: Vec<CharacterStyle>,
}
#[derive(Debug, Clone)]
struct CharacterStyle {
    name: String,
    style_id: StyleId,
    vvm_ref: Arc<VoiceModelFile>,
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

struct VVModelContext {
    synth: Synthesizer<OpenJtalk>,
    current_style: Option<CharacterStyle>,
    current_character: Option<Character>,
    characters: Vec<Character>,
}

impl VVModelContext {
    fn new(path_to_dylib: String, ojt_dic_dir: String) -> Result<Self, Box<dyn Error>> {
        // let vvm = current_exe_tree("/voicevox_core/models/vvms/0.vvm");

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

    fn tts(&self, text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
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
