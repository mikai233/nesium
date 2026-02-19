mod common;

use anyhow::Result;
use common::run_rom_frames;
use std::fs;

#[test]
fn emit_irq_flag_timing_trace() -> Result<()> {
    let rom = std::env::var("NESIUM_APU_TRACE_ROM")
        .unwrap_or_else(|_| "apu_test/rom_singles/6-irq_flag_timing.nes".to_string());

    let frames = std::env::var("NESIUM_APU_TRACE_FRAMES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(240);

    let dump_status = std::env::var("NESIUM_APU_TRACE_DUMP_STATUS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let dump_serial = std::env::var("NESIUM_APU_TRACE_DUMP_SERIAL")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let dump_ram_path = std::env::var("NESIUM_APU_TRACE_DUMP_RAM_PATH").ok();
    let dump_ram_base = std::env::var("NESIUM_APU_TRACE_DUMP_RAM_BASE")
        .ok()
        .and_then(|v| u16::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x0000);
    let dump_ram_len = std::env::var("NESIUM_APU_TRACE_DUMP_RAM_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0x0800);

    run_rom_frames(&rom, frames, |nes| {
        if dump_status {
            eprintln!(
                "TRACE_STATUS|6000={:02X}|6001={:02X}|6002={:02X}|6003={:02X}|6004={:02X}|6005={:02X}|6006={:02X}|6007={:02X}",
                nes.peek_cpu_byte(0x6000),
                nes.peek_cpu_byte(0x6001),
                nes.peek_cpu_byte(0x6002),
                nes.peek_cpu_byte(0x6003),
                nes.peek_cpu_byte(0x6004),
                nes.peek_cpu_byte(0x6005),
                nes.peek_cpu_byte(0x6006),
                nes.peek_cpu_byte(0x6007)
            );
        }
        if dump_serial {
            let bytes = nes.take_serial_output();
            let mut text = String::new();
            for b in bytes {
                match b {
                    b'\n' | b'\r' => text.push('\n'),
                    0x20..=0x7E => text.push(b as char),
                    _ => {}
                }
            }
            eprintln!("TRACE_SERIAL_BEGIN");
            eprintln!("{}", text);
            eprintln!("TRACE_SERIAL_END");
        }
        if let Some(path) = dump_ram_path.as_deref() {
            let mut buf = vec![0u8; dump_ram_len];
            nes.peek_cpu_slice(dump_ram_base, &mut buf);
            fs::write(path, &buf)?;
            eprintln!(
                "TRACE_RAM_DUMP|path={}|base={:04X}|len={}",
                path, dump_ram_base, dump_ram_len
            );
        }
        Ok(())
    })
}
