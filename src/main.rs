use std::env;

use crate::tts::characters::{Character, CharacterStyle};
use iced::Theme;
use ui::IcedVVGUIState;
use voicevox_core::blocking::Onnxruntime;

mod tts;
mod ui;

const APP_NAME: &str = "VVRustGUI";

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
