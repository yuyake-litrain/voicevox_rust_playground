use std::{env};

use voicevox_core::blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile};

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
    dbg!(synth.metas());
}
