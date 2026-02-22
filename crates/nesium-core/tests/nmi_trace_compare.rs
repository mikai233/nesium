mod common;

use anyhow::{Context, Result, bail};
use common::run_rom_frames;
use nesium_core::Nes;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn emit_nmi_sync_trace() -> Result<()> {
    let rom = std::env::var("NESIUM_NMI_TRACE_ROM")
        .unwrap_or_else(|_| "nmi_sync/demo_ntsc.nes".to_string());

    let frames = std::env::var("NESIUM_NMI_TRACE_FRAMES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(260);

    run_rom_frames(&rom, frames, |_| Ok(()))
}

fn compute_foreground_mask(index_buffer: &[u8]) -> Vec<u8> {
    let mut counts = [0usize; 256];
    for &px in index_buffer {
        counts[px as usize] += 1;
    }

    let mut bg_index = 0usize;
    let mut bg_count = 0usize;
    for (idx, &count) in counts.iter().enumerate() {
        if count > bg_count {
            bg_count = count;
            bg_index = idx;
        }
    }

    let mut mask = Vec::with_capacity(index_buffer.len().div_ceil(8));
    let mut packed = 0u8;
    let mut bit = 0u8;
    for &px in index_buffer {
        if px as usize != bg_index {
            packed |= 1u8 << bit;
        }
        bit += 1;
        if bit == 8 {
            mask.push(packed);
            packed = 0;
            bit = 0;
        }
    }
    if bit != 0 {
        mask.push(packed);
    }
    mask
}

fn parse_frames_csv(value: &str) -> Result<Vec<usize>> {
    let mut frames = BTreeSet::new();
    for token in value.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let frame = trimmed
            .parse::<usize>()
            .with_context(|| format!("invalid frame token `{trimmed}`"))?;
        frames.insert(frame);
    }
    if frames.is_empty() {
        bail!("frame list must not be empty");
    }
    Ok(frames.into_iter().collect())
}

fn parse_u32_csv(value: &str) -> Result<Vec<u32>> {
    let mut values = BTreeSet::new();
    for token in value.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let n = trimmed
            .parse::<u32>()
            .with_context(|| format!("invalid token `{trimmed}`"))?;
        values.insert(n);
    }
    if values.is_empty() {
        bail!("value list must not be empty");
    }
    Ok(values.into_iter().collect())
}

fn default_out_prefix(env_key: &str, file_stem: &str) -> String {
    std::env::var(env_key).unwrap_or_else(|_| format!("target/compare/{file_stem}"))
}

#[test]
#[ignore = "debug utility: dumps foreground masks for selected frames"]
fn dump_nmi_sync_masks() -> Result<()> {
    let rom = std::env::var("NESIUM_NMI_MASK_ROM")
        .unwrap_or_else(|_| "nmi_sync/demo_ntsc.nes".to_string());
    let frames_env =
        std::env::var("NESIUM_NMI_MASK_FRAMES").unwrap_or_else(|_| "240,241".to_string());
    let frames = parse_frames_csv(&frames_env)?;
    let out_prefix = default_out_prefix("NESIUM_NMI_MASK_OUT_PREFIX", "nesium_nmi_sync_mask");

    let max_frame = *frames.last().expect("frames not empty");
    let frame_set: BTreeSet<usize> = frames.iter().copied().collect();
    let row_trace = std::env::var("NESIUM_NMI_MASK_TRACE_ROW")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let x0 = std::env::var("NESIUM_NMI_MASK_TRACE_X0")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let x1 = std::env::var("NESIUM_NMI_MASK_TRACE_X1")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(255);
    let trace_frames = std::env::var("NESIUM_NMI_MASK_TRACE_FRAMES")
        .ok()
        .map(|v| parse_u32_csv(&v))
        .transpose()?
        .map_or_else(BTreeSet::new, |v| v.into_iter().collect::<BTreeSet<_>>());

    let rom_path = Path::new(common::ROM_ROOT).join(&rom);
    if !rom_path.exists() {
        bail!("ROM not found: {}", rom_path.display());
    }

    let mut nes = Nes::default();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;

    for frame in 0..=max_frame {
        nes.run_frame(false);
        if !frame_set.contains(&frame) {
            continue;
        }

        let mask = compute_foreground_mask(nes.render_index_buffer());
        let out_path = format!("{}_f{}.bin", out_prefix, frame);
        if let Some(parent) = Path::new(&out_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, &mask).with_context(|| format!("writing {}", out_path))?;
        eprintln!("NMI_MASK_DUMP|frame={}|path={}", frame, out_path);

        if let Some(y) = row_trace {
                let ppu_frame = nes.ppu.frame_count();
            if trace_frames.is_empty() || trace_frames.contains(&ppu_frame) {
                let fb = nes.render_index_buffer();
                let width = 256usize;
                let mut row_bits = String::new();
                let mut row_vals = String::new();
                for x in x0..=x1.min(width - 1) {
                    let idx = y.saturating_mul(width) + x;
                    if idx >= fb.len() {
                        break;
                    }
                    let px = fb[idx];
                    let bit = (mask[idx / 8] >> (idx % 8)) & 1;
                    row_bits.push(if bit != 0 { '1' } else { '0' });
                    if !row_vals.is_empty() {
                        row_vals.push(' ');
                    }
                    row_vals.push_str(&format!("{:02X}", px));
                }
                eprintln!(
                    "NMI_MASK_ROW|frame={}|ppu_frame={}|y={}|x0={}|x1={}|bits={}|vals={}",
                    frame, ppu_frame, y, x0, x1, row_bits, row_vals
                );
            }
        }
    }

    Ok(())
}
