use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(nesium_has_vrc7_native)");
    generate_cartridge_db();

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

fn generate_cartridge_db() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let out_path = out_dir.join("cartridge_db_generated.rs");

    if env::var_os("CARGO_FEATURE_CARTRIDGE_DB").is_none() {
        fs::write(
            &out_path,
            "pub(crate) static NES_DB: phf::Map<u32, CartridgeDbEntry> = phf::phf_map! {};\n",
        )
        .expect("write disabled cartridge DB stub");
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let db_path = manifest_dir.join("data").join("MesenNesDB.txt");
    println!("cargo:rerun-if-changed={}", db_path.display());

    let db_text = fs::read_to_string(&db_path)
        .unwrap_or_else(|err| panic!("failed to read cartridge DB '{}': {err}", db_path.display()));

    let mut entries = Vec::new();
    for line in db_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let fields: Vec<_> = trimmed.split(',').collect();
        if fields.len() < 16 {
            continue;
        }

        let crc32 = u32::from_str_radix(fields[0], 16).unwrap_or_else(|err| {
            panic!(
                "invalid CRC '{}' in '{}': {err}",
                fields[0],
                db_path.display()
            )
        });
        let mapper = parse_csv_u16(fields[5]);
        let submapper = parse_csv_u8(fields[15]);
        let prg_rom_size = parse_csv_usize(fields[6]) * 1024;
        let chr_rom_size = parse_csv_usize(fields[7]) * 1024;
        let chr_ram_size = parse_csv_usize(fields[8]) * 1024;
        let work_ram_size = parse_csv_usize(fields[9]) * 1024;
        let save_ram_size = parse_csv_usize(fields[10]) * 1024;
        let has_battery = parse_csv_u8(fields[11]) != 0;

        let value = format!(
            "CartridgeDbEntry {{ rom_body_crc32: 0x{crc32:08X}, mapper: {mapper}, submapper: {submapper}, prg_rom_size: {prg_rom_size}, chr_rom_size: {chr_rom_size}, chr_ram_size: {chr_ram_size}, work_ram_size: {work_ram_size}, save_ram_size: {save_ram_size}, has_battery: {has_battery} }}"
        );
        entries.push((crc32, value));
    }

    let mut map = phf_codegen::Map::new();
    for (key, value) in &entries {
        map.entry(key, value);
    }

    let generated = format!(
        "pub(crate) static NES_DB: phf::Map<u32, CartridgeDbEntry> = {};\n",
        map.build()
    );
    fs::write(&out_path, generated).expect("write generated cartridge DB");
}

fn parse_csv_usize(text: &str) -> usize {
    text.trim().parse::<usize>().unwrap_or(0)
}

fn parse_csv_u16(text: &str) -> u16 {
    text.trim().parse::<u16>().unwrap_or(0)
}

fn parse_csv_u8(text: &str) -> u8 {
    text.trim().parse::<u8>().unwrap_or(0)
}
