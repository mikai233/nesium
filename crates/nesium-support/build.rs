use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        return;
    }

    let has_hqx = env::var_os("CARGO_FEATURE_HQX").is_some();
    let has_ntsc = env::var_os("CARGO_FEATURE_NTSC").is_some();
    let has_sai_cpp = env::var_os("CARGO_FEATURE_SAI_CPP").is_some();
    let has_lcd_grid = env::var_os("CARGO_FEATURE_LCD_GRID").is_some();
    let has_scanline = env::var_os("CARGO_FEATURE_SCANLINE").is_some();
    let has_xbrz = env::var_os("CARGO_FEATURE_XBRZ").is_some();
    let has_ntsc_bisqwit = env::var_os("CARGO_FEATURE_NTSC_BISQWIT").is_some();
    if !has_hqx
        && !has_ntsc
        && !has_sai_cpp
        && !has_lcd_grid
        && !has_scanline
        && !has_xbrz
        && !has_ntsc_bisqwit
    {
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

    if has_sai_cpp {
        let vendor_dir = crate_dir.join("vendor").join("sai");
        for rel in [
            "2xSai.cpp",
            "Super2xSai.cpp",
            "SuperEagle.cpp",
            "sai_wrapper.cpp",
        ] {
            println!("cargo:rerun-if-changed={}", vendor_dir.join(rel).display());
        }

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.files([
            vendor_dir.join("2xSai.cpp"),
            vendor_dir.join("Super2xSai.cpp"),
            vendor_dir.join("SuperEagle.cpp"),
            vendor_dir.join("sai_wrapper.cpp"),
        ]);

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_sai");
    }

    if has_lcd_grid {
        let vendor_dir = crate_dir.join("vendor").join("lcd_grid");
        println!(
            "cargo:rerun-if-changed={}",
            vendor_dir.join("lcd_grid.cpp").display()
        );

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.file(vendor_dir.join("lcd_grid.cpp"));

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_lcd_grid");
    }

    if has_scanline {
        let vendor_dir = crate_dir.join("vendor").join("scanline");
        println!(
            "cargo:rerun-if-changed={}",
            vendor_dir.join("scanline.cpp").display()
        );

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.file(vendor_dir.join("scanline.cpp"));

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_scanline");
    }

    if has_xbrz {
        let vendor_dir = crate_dir.join("vendor").join("xbrz");
        for rel in ["config.h", "xbrz.h", "xbrz.cpp", "xbrz_wrapper.cpp"] {
            println!("cargo:rerun-if-changed={}", vendor_dir.join(rel).display());
        }

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.files([
            vendor_dir.join("xbrz.cpp"),
            vendor_dir.join("xbrz_wrapper.cpp"),
        ]);

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_xbrz");
    }

    if has_ntsc_bisqwit {
        let vendor_dir = crate_dir.join("vendor").join("ntsc_bisqwit");
        println!(
            "cargo:rerun-if-changed={}",
            vendor_dir.join("bisqwit_ntsc.cpp").display()
        );

        let mut build = cc::Build::new();
        build.cpp(true);
        build.include(&vendor_dir);
        build.file(vendor_dir.join("bisqwit_ntsc.cpp"));

        build.warnings(false);
        build.flag_if_supported("/std:c++17");
        build.flag_if_supported("-std=c++17");

        build.compile("nesium_support_ntsc_bisqwit");
    }
}
