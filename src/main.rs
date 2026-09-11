use std::io::Cursor;
use std::rc::Rc;
use std::{env, fs::File, io::Write};

use iced::Theme;
use iced::widget::{Column, button, column, text, text_input};
use voicevox_core::StyleId;
use voicevox_core::{
    CharacterMeta, StyleMeta,
    blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
};


const APP_NAME: &str = "VOICEVOX Rust GUI";

fn main() -> iced::Result {
    let current_exe_tree = |path: &str| {
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
    };

    let path_to_dylib = current_exe_tree(
        format!(
            "/voicevox_core/onnxruntime/lib/{}",
            Onnxruntime::LIB_VERSIONED_FILENAME
        )
        .as_str(),
    );
    let ojt_dic_dir = current_exe_tree("/voicevox_core/dict/open_jtalk_dic_utf_8-1.11");
    let vvm = current_exe_tree("/voicevox_core/models/vvms/0.vvm");

    let synth = {
        let ort = Onnxruntime::load_once()
            .filename(path_to_dylib)
            .perform()
            .unwrap();
        let ojt = OpenJtalk::new(ojt_dic_dir).unwrap();
        Synthesizer::builder(ort)
            .text_analyzer(ojt)
            .build()
            .unwrap()
    };

    dbg!(synth.is_gpu_mode());

    synth
        .load_voice_model(&VoiceModelFile::open(vvm).unwrap())
        .unwrap();
    let StyleMeta { id: style_id, .. } = synth
        .metas()
        .into_iter()
        .filter(|CharacterMeta { name, .. }| name == "ずんだもん")
        .flat_map(|CharacterMeta { styles, .. }| styles)
        .find(|StyleMeta { name, .. }| name == "ノーマル")
        .unwrap();

    let synth = Rc::new(synth);

    iced::application(
        move || IcedVVGUIState::new(synth.clone(), style_id),
        IcedVVGUIState::update,
        IcedVVGUIState::view,
    )
    .theme(Theme::Dark)
    .title(APP_NAME)
    .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    TextEdited(String),
    TTSBtnPressed,
    SayBtnPressed,
}

struct IcedVVGUIState {
    synth: Rc<Synthesizer<OpenJtalk>>,
    current_text: String,
    style_id: StyleId,
}

impl IcedVVGUIState {
    fn new(synth: Rc<Synthesizer<OpenJtalk>>, style_id: StyleId) -> Self {
        Self {
            synth,
            current_text: String::default(),
            style_id,
        }
    }

    fn update(state: &mut IcedVVGUIState, message: Message) {
        match message {
            Message::TextEdited(text) => {
                state.current_text = text;
            }
            Message::TTSBtnPressed => {
                let wav = &state
                    .synth
                    .tts(&state.current_text, state.style_id)
                    .perform()
                    .unwrap();
                let mut file = File::create(format!("zunda_{}.wav", state.current_text)).unwrap();
                file.write_all(wav).unwrap();
            }
            Message::SayBtnPressed => {
                let wav = state
                    .synth
                    .tts(&state.current_text, state.style_id)
                    .perform()
                    .unwrap();
                let wav = Cursor::new(wav);
                let sink_handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
                let player = rodio::play(sink_handle.mixer(), wav).unwrap();
                player.sleep_until_end();
            }
        }
    }

    fn view<'a>(state: &'a IcedVVGUIState) -> Column<'a, Message> {
        column![
            text(format!(
                "以下の文字列が音声合成され保存されます\n{}",
                &state.current_text
            )),
            text_input("Input text...", &state.current_text).on_input(Message::TextEdited),
            button("TTS & Save!").on_press(Message::TTSBtnPressed),
            button("Say").on_press(Message::SayBtnPressed)
        ]
    }
}

