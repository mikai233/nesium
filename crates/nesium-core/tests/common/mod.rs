#![allow(dead_code)]

use std::{path::Path, time::Instant};

use anyhow::{Context, Result, bail};
use nesium_core::NES;

pub const ROM_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/nes-test-roms");
pub const STATUS_ADDR: u16 = 0x6000;
pub const STATUS_MESSAGE_ADDR: u16 = 0x6004;
pub const STATUS_MAX_BYTES: usize = 256;

/// Status byte protocol used by blargg-style test ROMs (see individual READMEs):
/// - $80: test is running
/// - $81: test requests a reset after a short delay
/// - $00-$7F: final result code (0 = pass; 1+ = fail / error code)
const STATUS_RUNNING: u8 = 0x80;
const STATUS_NEEDS_RESET: u8 = 0x81;
const STATUS_MAGIC: [u8; 3] = [0xDE, 0xB0, 0x61];
/// Number of NTSC frames to wait after a ROM requests reset via $81.
/// README recommends waiting at least 100ms; ~6 frames at 60Hz is sufficient.
const RESET_DELAY_FRAMES: usize = 6;

#[derive(Debug)]
pub enum Progress {
    Running {
        message: String,
        needs_reset: bool,
    },
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

    let mut reset_delay_frames: Option<usize> = None;

    for _ in 0..frames {
        // Apply any pending reset once the requested delay has elapsed.
        if let Some(counter) = reset_delay_frames.as_mut() {
            if *counter == 0 {
                nes.reset();
                reset_delay_frames = None;
            } else {
                *counter -= 1;
            }
        }

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
            Progress::Running { message, needs_reset } => {
                if !message.is_empty() {
                    last_status = message;
                }
                if needs_reset && reset_delay_frames.is_none() {
                    reset_delay_frames = Some(RESET_DELAY_FRAMES);
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
        Progress::Running { message, .. } => {
            if !message.is_empty() {
                last_status = message;
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

/// Runs a ROM for `frames` without depending on $6000 status handshakes, then calls `verify`.
pub fn run_rom_frames<F>(rom_rel_path: &str, frames: usize, mut verify: F) -> Result<()>
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

    for _ in 0..frames {
        nes.run_frame();
    }

    verify(&mut nes)
}

/// Simple heuristic to ensure the framebuffer isn't blank: require at least `min_unique` distinct color indices.
pub fn require_color_diversity(nes: &NES, min_unique: usize) -> Result<()> {
    let mut seen = [false; 256];
    let fb = nes.framebuffer();
    for &b in fb {
        seen[b as usize] = true;
    }
    let unique = seen.iter().filter(|b| **b).count();
    if unique < min_unique {
        bail!("framebuffer has only {} unique color indices (expected at least {})", unique, min_unique);
    }
    Ok(())
}

fn poll_status(nes: &mut NES) -> Progress {
    let status = nes.peek_cpu_byte(STATUS_ADDR);
    let message = read_status_message(nes);
    let has_magic = has_status_magic(nes);

    // Until the blargg signature appears at $6001-$6003, the contents of $6000
    // are not guaranteed to follow the documented protocol, so treat the test
    // as still running regardless of the current value.
    if !has_magic {
        return Progress::Running {
            message,
            needs_reset: false,
        };
    }

    match status {
        STATUS_RUNNING => Progress::Running {
            message,
            needs_reset: false,
        },
        STATUS_NEEDS_RESET => Progress::Running {
            message,
            needs_reset: true,
        },
        0 => Progress::Passed(message),
        code @ 0x01..=0x7F => Progress::Failed(code, message),
        code => Progress::Failed(code, message),
    }
}

fn has_status_magic(nes: &mut NES) -> bool {
    let mut buf = [0u8; 3];
    nes.peek_cpu_slice(STATUS_ADDR + 1, &mut buf);
    buf == STATUS_MAGIC
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
