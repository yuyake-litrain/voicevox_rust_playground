use std::{
    fs::File,
    io::{Cursor, Write},
};

use crate::tts::{
    characters::{Character, CharacterStyle},
    model_context::VVModelContext,
};
use iced::widget::{Container, button, column, combo_box, container, row, text, text_input};
use iced::{Alignment, Background, Length, Padding, Pixels, color};

pub mod menu_bar;
use menu_bar::menu_bar;

#[derive(Debug, Clone)]
pub enum Message {
    TextEdited(String),
    TTSBtnPressed,
    SayBtnPressed,
    CharacterSelected(Character),
    CharacterStyleSelected(CharacterStyle),
    MenuBarBtnPressed,
}

pub struct IcedVVGUIState {
    model_context: VVModelContext,
    current_text: String,
    characters: combo_box::State<Character>,
    current_character: Option<Character>,
    current_character_styles: combo_box::State<CharacterStyle>,
    current_character_style: Option<CharacterStyle>,
    able_to_tts: bool,
}

impl IcedVVGUIState {
    pub fn new(path_to_dylib: String, ojt_dic_dir: String) -> Self {
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

    pub fn update(state: &mut IcedVVGUIState, message: Message) {
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

    pub fn view<'a>(state: &'a IcedVVGUIState) -> Container<'a, Message> {
        container(column![
            // メニューバー
            menu_bar(state),
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
