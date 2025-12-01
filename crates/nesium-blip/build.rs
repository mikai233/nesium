use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let header = manifest_dir.join("csrc/blip_buf.h");
    let source = manifest_dir.join("csrc/blip_buf.c");
    let license = manifest_dir.join("csrc/license.md");

    // Re-run if any of the C sources or license change.
    println!("cargo:rerun-if-changed={}", header.display());
    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-changed={}", license.display());

    cc::Build::new()
        .file(&source)
        .include(header.parent().unwrap())
        .compile("blip_buf_c");

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .allowlist_function("blip_.*")
        .allowlist_type("blip_.*")
        .allowlist_var("blip_.*")
        .generate()
        .expect("Unable to generate bindings for blip_buf");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
