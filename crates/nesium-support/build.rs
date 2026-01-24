use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        return;
    }

    let has_hqx = env::var_os("CARGO_FEATURE_HQX").is_some();
    let has_ntsc = env::var_os("CARGO_FEATURE_NTSC").is_some();
    if !has_hqx && !has_ntsc {
        return;
    }

    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    if has_hqx {
        let vendor_dir = crate_dir.join("vendor").join("hqx");
        for rel in [
            "common.h", "hqx.h", "init.cpp", "hq2x.cpp", "hq3x.cpp", "hq4x.cpp",
        ] {
            println!("cargo:rerun-if-changed={}", vendor_dir.join(rel).display());
        }

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.files([
            vendor_dir.join("init.cpp"),
            vendor_dir.join("hq2x.cpp"),
            vendor_dir.join("hq3x.cpp"),
            vendor_dir.join("hq4x.cpp"),
        ]);

        // Ensure the C ABI is consistent across toolchains (avoid stdcall on windows-gnu).
        build.define("HQX_CALLCONV", "");

        // Avoid leaking vendor warnings into our build logs.
        build.warnings(false);

        // Try to use a reasonable C++ dialect across MSVC/Clang/GCC.
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_hqx");
    }

    if has_ntsc {
        let vendor_dir = crate_dir.join("vendor").join("ntsc");
        for rel in [
            "nes_ntsc.h",
            "nes_ntsc.cpp",
            "nes_ntsc_impl.h",
            "nes_ntsc_config.h",
        ] {
            println!("cargo:rerun-if-changed={}", vendor_dir.join(rel).display());
        }

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.file(vendor_dir.join("nes_ntsc.cpp"));

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_nes_ntsc");
    }
}
