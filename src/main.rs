use std::{env, fs::File, io::Write};

use voicevox_core::{CharacterMeta, StyleMeta, blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile}};

fn main() {
    let current_exe_tree = |path: &str| {
        format!(
                "{}{}",
                env::current_exe().unwrap().parent().unwrap().to_str().unwrap(),
                path
                )
    };

    let path_to_dylib = current_exe_tree(format!("/voicevox_core/onnxruntime/lib/{}",Onnxruntime::LIB_VERSIONED_FILENAME).as_str());
    let ojt_dic_dir = current_exe_tree("/voicevox_core/dict/open_jtalk_dic_utf_8-1.11");
    let vvm = current_exe_tree("/voicevox_core/models/vvms/0.vvm");

    let synth = {
        let ort = Onnxruntime::load_once().filename(path_to_dylib).perform().unwrap();
        let ojt = OpenJtalk::new(ojt_dic_dir).unwrap();
        Synthesizer::builder(ort).text_analyzer(ojt).build().unwrap()
    };

    dbg!(synth.is_gpu_mode());

    synth.load_voice_model(&VoiceModelFile::open(vvm).unwrap()).unwrap();
    let StyleMeta { id: style_id, .. } = synth
        .metas()
        .into_iter()
        .filter(|CharacterMeta {name, ..}| name == "ずんだもん")
        .flat_map(|CharacterMeta { styles, .. }| styles)
        .find(|StyleMeta { name, ..}| name == "ノーマル").unwrap();

    let wav = &synth.tts("こんにちは", style_id).perform().unwrap();

    let mut file = File::create("zuda.wav").unwrap();
    file.write_all(wav).unwrap();
    // dbg!(wav);
}
