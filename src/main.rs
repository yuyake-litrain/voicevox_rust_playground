use std::error::Error;
use std::fmt::Display;
use std::fs::{self};
use std::io::Cursor;
use std::sync::Arc;
use std::{env, fs::File, io::Write};

use iced::widget::button::Status;
use iced::widget::{Container, button, column, combo_box, container, row, text, text_input};
use iced::{Alignment, Background, Border, Color, Length, Padding, Pixels, Theme, color};
use iced_aw::style::menu_bar;
use iced_aw::widget::menu::{Item, Menu, MenuBar};
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
    MenuBarBtnPressed,
}

struct IcedVVGUIState {
    model_context: VVModelContext,
    current_text: String,
    characters: combo_box::State<Character>,
    current_character: Option<Character>,
    current_character_styles: combo_box::State<CharacterStyle>,
    current_character_style: Option<CharacterStyle>,
    able_to_tts: bool,
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
            able_to_tts: false,
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
                state.current_character_style = Some(character.styles[0].clone());
                state.model_context.current_style = state.current_character_style.clone();
                state.able_to_tts = true
            }
            Message::CharacterStyleSelected(style) => {
                state.current_character_style = Some(style.clone());
                state.model_context.current_style = state.current_character_style.clone();
                state.able_to_tts = true
            }
            _ => (),
        }
    }

    fn view<'a>(state: &'a IcedVVGUIState) -> Container<'a, Message> {
        let style = |_: &Theme, status: button::Status| match status {
            Status::Disabled => button::Style {
                background: None,
                text_color: color!(0x444444),
                ..Default::default()
            },
            Status::Active => button::Style {
                background: None,
                text_color: Color::WHITE,
                ..Default::default()
            },
            Status::Hovered => button::Style {
                background: Some(Background::Color(color!(0x222222))),
                text_color: Color::WHITE,
                ..Default::default()
            },
            Status::Pressed => button::Style {
                background: Some(Background::Color(color!(0x444444))),
                text_color: Color::WHITE,
                ..Default::default()
            },
        };
        container(column![
            // メニューバー
            row!({
                let root_file = Item::with_menu(
                    button("File")
                        .style(style)
                        .on_press(Message::MenuBarBtnPressed),
                    Menu::new(vec![
                        Item::new(button("Open Project").style(style)),
                        Item::new(button("Close").style(style)),
                        Item::with_menu(
                            button("Recent Projects").style(style),
                            Menu::new(vec![
                                Item::new(button("hoge_prj1").style(style)),
                                Item::new(button("hoge_prj2").style(style)),
                                Item::new(button("hoge_prj3").style(style)),
                            ]),
                        ),
                        Item::new(button("Export as WAV").style(style).on_press_maybe(
                            match state.able_to_tts {
                                true => Some(Message::TTSBtnPressed),
                                false => None,
                            },
                        )),
                        Item::new(button("Save").style(style)),
                        Item::new(button("Save as...").style(style)),
                        Item::new(button("Exit").style(style)),
                    ]),
                );

                let root_edit = Item::with_menu(
                    button("Edit")
                        .style(style)
                        .on_press(Message::MenuBarBtnPressed),
                    Menu::new(vec![
                        Item::new(button("Say").style(style).on_press_maybe(
                            match state.able_to_tts {
                                true => Some(Message::TTSBtnPressed),
                                false => None,
                            },
                        )),
                        Item::new(button("Cut").style(style)),
                        Item::new(button("Copy").style(style)),
                        Item::new(button("Paste").style(style)),
                        Item::new(button("Select All").style(style)),
                        Item::new(button("Remove").style(style)),
                        Item::new(button("Settings...").style(style)),
                    ]),
                );

                let root_view = Item::with_menu(
                    button("View")
                        .style(style)
                        .on_press(Message::MenuBarBtnPressed),
                    Menu::new(vec![Item::new(button("Set default").style(style))]),
                );
                let root_engine = Item::with_menu(
                    button("Engine")
                        .style(style)
                        .on_press(Message::MenuBarBtnPressed),
                    Menu::new(vec![
                        Item::new(button("Engines...").style(style)),
                        Item::new(button("Models...").style(style)),
                    ]),
                );
                let root_plugins = Item::with_menu(
                    button("Plugins").style(style),
                    Menu::new(vec![Item::new(button("Not Implemented").style(style))]),
                );
                MenuBar::new(vec![
                    root_file,
                    root_edit,
                    root_view,
                    root_engine,
                    root_plugins,
                ])
                .style(|_, _| menu_bar::Style {
                    menu_border: Border {
                        color: color!(0x444444),
                        width: 1.0,
                        ..Default::default()
                    },
                    bar_background: Background::Color(Color::TRANSPARENT),
                    menu_background: Background::Color(color!(0x0F0F0F)),
                    ..Default::default()
                })
            },)
            .width(Length::Fill)
            .align_y(Alignment::Start),
            // 線
            container("")
                .width(Length::Fill)
                .height(2.0)
                .style(|_| container::Style {
                    background: Some(Background::Color(color!(0x444444))),
                    ..Default::default()
                }),
            row![
                column![
                    column![
                        text("以下の文字列が音声合成されます",).size(Pixels::from(12.0)),
                        text(&state.current_text).size(Pixels::from(20.0)),
                    ]
                    .align_x(Alignment::Center),
                    row![
                        combo_box(
                            &state.characters,
                            "Select Characters",
                            state.current_character.as_ref(),
                            Message::CharacterSelected
                        )
                        .width(Length::FillPortion(1)),
                        combo_box(
                            &state.current_character_styles,
                            "Select Characters",
                            state.current_character_style.as_ref(),
                            Message::CharacterStyleSelected,
                        )
                        .width(Length::FillPortion(1)),
                        text_input("Input text...", &state.current_text)
                            .on_input(Message::TextEdited)
                            .width(Length::FillPortion(4)),
                    ]
                    .spacing(10)
                    .padding(Padding::from([30, 10])),
                    row![
                        button("TTS & Save!").on_press_maybe(match state.able_to_tts {
                            true => Some(Message::TTSBtnPressed),
                            false => None,
                        }),
                        button("Say").on_press_maybe(match state.able_to_tts {
                            true => Some(Message::SayBtnPressed),
                            false => None,
                        }),
                    ]
                    .spacing(10)
                    .padding(Padding::from([30, 10])),
                ]
                .width(Length::Fill)
                .align_x(Alignment::Center),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Center)
        ])
        .style(|_| container::Style {
            background: Some(Background::Color(color!(0x0F0F0F))),
            ..Default::default()
        })
        .height(Length::Fill)
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
