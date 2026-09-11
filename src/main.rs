use std::io::Cursor;
use std::error::Error;
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

    iced::application(
        move || IcedVVGUIState::new(path_to_dylib.clone(), ojt_dic_dir.clone(), vvm.clone()),
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
    model_context: VVModelContext,
    current_text: String,
}

impl IcedVVGUIState {
    fn new(path_to_dylib: String, ojt_dic_dir: String, vvm: String) -> Self {
        let model_context = VVModelContext::new(path_to_dylib, ojt_dic_dir, vvm).unwrap();
        Self {
            model_context,
            current_text: String::default(),
        }
    }

    fn update(state: &mut IcedVVGUIState, message: Message) {
        match message {
            Message::TextEdited(text) => {
                state.current_text = text;
            }
            Message::TTSBtnPressed => {
                let wav = state.model_context.tts(&state.current_text).unwrap();
                let mut file = File::create(format!("zunda_{}.wav", state.current_text)).unwrap();
                file.write_all(&wav).unwrap()
            }
            Message::SayBtnPressed => {
                let wav = state.model_context.tts(&state.current_text).unwrap();
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

struct VVModelContext {
    synth: Synthesizer<OpenJtalk>,
    style_id: StyleId,
}

impl VVModelContext {
    fn new(
        path_to_dylib: String,
        ojt_dic_dir: String,
        vvm: String,
    ) -> Result<Self, Box<dyn Error>> {
        let synth = {
            let ort = Onnxruntime::load_once().filename(path_to_dylib).perform()?;
            let ojt = OpenJtalk::new(ojt_dic_dir).unwrap();
            Synthesizer::builder(ort).text_analyzer(ojt).build()?
        };

        dbg!(synth.is_gpu_mode());

        synth.load_voice_model(&VoiceModelFile::open(vvm)?)?;

        let StyleMeta { id: style_id, .. } = synth
            .metas()
            .into_iter()
            .filter(|CharacterMeta { name, .. }| name == "ずんだもん")
            .flat_map(|CharacterMeta { styles, .. }| styles)
            .find(|StyleMeta { name, .. }| name == "ノーマル")
            .ok_or("style not found")?;

        Ok(Self { synth, style_id })
    }

    fn tts(&self, text: &str) -> Result<Vec<u8>, voicevox_core::Error> {
        self.synth.tts(text, self.style_id).perform()
    }
}
