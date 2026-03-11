use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nesium_has_vrc7_native)");

    let target = env::var("TARGET").expect("TARGET not set");
    if target.contains("wasm32") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let emu2413_dir = manifest_dir.join("third_party").join("emu2413");

    println!(
        "cargo:rerun-if-changed={}",
        emu2413_dir.join("emu2413.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        emu2413_dir.join("emu2413.cpp").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        emu2413_dir.join("nesium_vrc7_wrapper.cpp").display()
    );

    cc::Build::new()
        .cpp(true)
        .include(&emu2413_dir)
        .file(emu2413_dir.join("emu2413.cpp"))
        .file(emu2413_dir.join("nesium_vrc7_wrapper.cpp"))
        .flag_if_supported("/std:c++17")
        .flag_if_supported("-std=c++17")
        .warnings(false)
        .compile("nesium_vrc7_emu2413");

    println!("cargo:rustc-cfg=nesium_has_vrc7_native");
}
