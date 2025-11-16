use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

const DEFAULT_HEADER_URL: &str =
    "https://raw.githubusercontent.com/libretro/libretro-common/master/include/libretro.h";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/libretro.h");
    println!("cargo:rerun-if-env-changed=LIBRETRO_BRIDGE_OFFLINE");
    println!("cargo:rerun-if-env-changed=LIBRETRO_BRIDGE_HEADER_URL");
    println!("cargo:rerun-if-env-changed=LIBRETRO_BRIDGE_FETCH");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    let downloaded_header = out_dir.join("libretro.h");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_header = manifest_dir.join("vendor/libretro.h");
    ensure_header(&downloaded_header, &vendor_header);

    generate_bindings(&downloaded_header, &out_dir.join("libretro_bindings.rs"));
}

fn ensure_header(download_path: &Path, vendor_header: &Path) {
    if let Some(parent) = download_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }

    let offline_env = env::var("LIBRETRO_BRIDGE_OFFLINE").ok();
    let offline = match offline_env.as_deref() {
        Some(value) => value != "0",
        None => env::var_os("LIBRETRO_BRIDGE_FETCH").is_none(),
    };
    let offline_forced = matches!(offline_env.as_deref(), Some(value) if value != "0");
    let header_url = env::var("LIBRETRO_BRIDGE_HEADER_URL")
        .unwrap_or_else(|_| DEFAULT_HEADER_URL.to_string());

    if !offline {
        match fetch_remote_header(&header_url, download_path) {
            Ok(()) => return,
            Err(err) => println!(
                "cargo:warning=Failed to download libretro.h from {header_url}: {err}. Falling back to the vendored header."
            ),
        }
    } else if offline_forced {
        println!("cargo:warning=LIBRETRO_BRIDGE_OFFLINE is set; using the vendored libretro.h.");
    }

    fs::copy(vendor_header, download_path)
        .unwrap_or_else(|err| panic!("Failed to copy vendored libretro.h: {err}"));
}

fn fetch_remote_header(url: &str, destination: &Path) -> Result<(), String> {
    let response = ureq::get(url)
        .call()
        .map_err(|err| format!("request failed: {err}"))?;

    let status = response.status();
    if !(200..300).contains(&status) {
        return Err(format!(
            "server returned {} {}",
            status,
            response.status_text()
        ));
    }

    let mut reader = response.into_reader();
    let mut file = File::create(destination)
        .map_err(|err| format!("unable to create {}: {err}", destination.display()))?;

    io::copy(&mut reader, &mut file)
        .map_err(|err| format!("failed to write {}: {err}", destination.display()))?;

    file.flush()
        .map_err(|err| format!("failed to flush {}: {err}", destination.display()))?;

    Ok(())
}

fn generate_bindings(header: &Path, output: &Path) {
    let mut builder = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_comments(false)
        .allowlist_type("retro_.*")
        .allowlist_function("retro_.*")
        .allowlist_var("RETRO_.*")
        .layout_tests(false)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true);

    if cfg!(target_os = "windows") {
        builder = builder.clang_arg("-D_CRT_SECURE_NO_WARNINGS");
    }

    let bindings = builder
        .generate()
        .expect("bindgen failed to produce libretro bindings");

    let patched = bindings
        .to_string()
        .replace("extern \"C\" {", "unsafe extern \"C\" {");

    fs::write(output, patched)
        .unwrap_or_else(|err| panic!("failed to write bindings to {}: {err}", output.display()));
}
