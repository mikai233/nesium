#[cfg(feature = "c-impl")]
use std::{env, path::PathBuf};

fn main() {
    #[cfg(feature = "c-impl")]
    build_c_impl();
}

#[cfg(feature = "c-impl")]
fn build_c_impl() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = manifest_dir.join("vendor");
    let header = vendor.join("blip_buf.h");
    let source = vendor.join("blip_buf.cpp");
    let license = vendor.join("LGPL.txt");

    for path in [&header, &source, &license] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    cc::Build::new()
        .cpp(true)
        .file(&source)
        .include(&vendor)
        .flag_if_supported("-std=c++17")
        .compile("blip_buf_vendor");

    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .clang_arg("-std=c++17")
        .clang_arg("-xc++")
        .clang_arg(format!("-I{}", vendor.display()))
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
