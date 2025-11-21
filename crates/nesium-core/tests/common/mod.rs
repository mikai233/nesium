#![allow(dead_code)]

use std::{path::Path, time::Instant};

use anyhow::{Context, Result, bail};
use nesium_core::NES;

pub const ROM_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/nes-test-roms");
pub const STATUS_ADDR: u16 = 0x6000;
pub const STATUS_MESSAGE_ADDR: u16 = 0x6004;
pub const STATUS_MAX_BYTES: usize = 256;

#[derive(Debug)]
pub enum Progress {
    Running(String),
    Passed(String),
    Failed(u8, String),
}

/// Runs a ROM until it reports pass/fail via $6000/$6004 or times out.
/// Returns the final status message on success.
pub fn run_rom_status(rom_rel_path: &str, frames: usize) -> Result<Option<String>> {
    run_rom_custom(rom_rel_path, frames, |_| Ok(()))
}

/// Runs a ROM with the standard $6000/$6004 status handshake, then invokes `verify`
/// to allow extra per-ROM assertions once the ROM reports success.
pub fn run_rom_custom<F>(rom_rel_path: &str, frames: usize, mut verify: F) -> Result<Option<String>>
where
    F: FnMut(&mut NES) -> Result<()>,
{
    let path = Path::new(ROM_ROOT).join(rom_rel_path);
    if !path.exists() {
        bail!("ROM not found: {}", path.display());
    }

    let mut nes = NES::new();
    nes.load_cartridge_from_file(&path)
        .with_context(|| format!("loading {}", path.display()))?;

    let mut last_status = String::new();
    let start = Instant::now();

    for _ in 0..frames {
        match poll_status(&mut nes) {
            Progress::Passed(msg) => {
                verify(&mut nes)?;
                return Ok(message_or_none(msg));
            }
            Progress::Failed(code, msg) => {
                bail!(
                    "failed with status byte {:#04X}{}",
                    code,
                    format_status(msg)
                )
            }
            Progress::Running(msg) => {
                if !msg.is_empty() {
                    last_status = msg;
                }
            }
        }

        nes.run_frame();
    }

    match poll_status(&mut nes) {
        Progress::Passed(msg) => {
            verify(&mut nes)?;
            Ok(message_or_none(msg))
        }
        Progress::Failed(code, msg) => {
            bail!(
                "failed with status byte {:#04X}{}",
                code,
                format_status(msg)
            )
        }
        Progress::Running(msg) => {
            if !msg.is_empty() {
                last_status = msg;
            }
            bail!(
                "timed out after {} frames{} (elapsed {:.2?})",
                frames,
                format_status(last_status),
                start.elapsed()
            )
        }
    }
}

fn poll_status(nes: &mut NES) -> Progress {
    let status = nes.peek_cpu_byte(STATUS_ADDR);
    let message = read_status_message(nes);
    let lower = message.to_ascii_lowercase();

    if lower.contains("pass") {
        return Progress::Passed(message);
    }
    if lower.contains("fail") || lower.contains("error") {
        return Progress::Failed(status, message);
    }

    match status {
        0 => Progress::Running(message),
        0x01 | 0x80 => Progress::Passed(message),
        code => Progress::Failed(code, message),
    }
}

fn read_status_message(nes: &mut NES) -> String {
    let mut raw = [0u8; STATUS_MAX_BYTES];
    nes.peek_cpu_slice(STATUS_MESSAGE_ADDR, &mut raw);
    let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
    let mut cleaned = String::new();
    for byte in &raw[..end] {
        match byte {
            b' '..=b'~' => cleaned.push(*byte as char),
            _ => cleaned.push(' '),
        }
    }
    cleaned.trim().to_string()
}

fn message_or_none(msg: String) -> Option<String> {
    if msg.trim().is_empty() {
        None
    } else {
        Some(msg)
    }
}

fn format_status(msg: String) -> String {
    if msg.trim().is_empty() {
        String::from(" without status message")
    } else {
        format!(" with status \"{}\"", msg)
    }
}
