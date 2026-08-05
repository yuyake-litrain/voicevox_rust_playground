use const_format::concatcp;
use voicevox_core::blocking::{Onnxruntime};

fn main() {
    const PATH_TO_DYLIB: &str = concatcp!(
        "./voicevox_core/onnxruntime/lib/",
        Onnxruntime::LIB_VERSIONED_FILENAME
        );
    println!("{PATH_TO_DYLIB}",)
}
