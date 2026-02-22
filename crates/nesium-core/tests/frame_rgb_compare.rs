mod common;

use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose};
use nesium_core::Nes;
use nesium_core::ppu::buffer::{ColorFormat, FrameBuffer};
use nesium_core::ppu::palette::PaletteKind;
use sha1::{Digest, Sha1};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

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

fn rgba8888_to_rgb24(frame_rgba: &[u8]) -> Result<Vec<u8>> {
    if !frame_rgba.len().is_multiple_of(4) {
        bail!(
            "rgba buffer length must be multiple of 4, got {}",
            frame_rgba.len()
        );
    }

    let mut rgb = Vec::with_capacity(frame_rgba.len() / 4 * 3);
    for px in frame_rgba.chunks_exact(4) {
        rgb.push(px[0]);
        rgb.push(px[1]);
        rgb.push(px[2]);
    }
    Ok(rgb)
}

fn parse_palette_kind(value: &str) -> Result<PaletteKind> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "nesdev" | "nesdev-ntsc" => Ok(PaletteKind::NesdevNtsc),
        "mesen" | "mesen-2c02" | "mesen2c02" => Ok(PaletteKind::Mesen2C02),
        "fbx" | "fbx-composite-direct" => Ok(PaletteKind::FbxCompositeDirect),
        "sony" | "sony-cxa2025as-us" => Ok(PaletteKind::SonyCxa2025AsUs),
        "pal2c07" | "pal-2c07" => Ok(PaletteKind::Pal2c07),
        "raw" | "raw-linear" => Ok(PaletteKind::RawLinear),
        _ => bail!("unknown palette kind: `{}`", value),
    }
}

fn sha1_base64(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    general_purpose::STANDARD.encode(hasher.finalize())
}

fn default_out_prefix(env_key: &str, file_stem: &str) -> String {
    std::env::var(env_key).unwrap_or_else(|_| {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(manifest_dir);
        workspace_root
            .join("target")
            .join("compare")
            .join(file_stem)
            .to_string_lossy()
            .into_owned()
    })
}

#[test]
#[ignore = "debug utility: dumps RGB24 frames for Mesen2 comparison"]
fn dump_rgb24_frames() -> Result<()> {
    let rom = std::env::var("NESIUM_RGB_DUMP_ROM")
        .unwrap_or_else(|_| "full_palette/flowing_palette.nes".to_string());
    let frames_csv =
        std::env::var("NESIUM_RGB_DUMP_FRAMES").unwrap_or_else(|_| "60,180,360,600".to_string());
    let frames = parse_frames_csv(&frames_csv)?;
    let out_prefix = default_out_prefix("NESIUM_RGB_DUMP_OUT_PREFIX", "nesium_frame_rgb");
    let palette = std::env::var("NESIUM_RGB_DUMP_PALETTE")
        .ok()
        .map(|v| parse_palette_kind(&v))
        .transpose()?;

    if frames.len() < 2 {
        bail!(
            "rgb24 dump requires at least 2 distinct frames, got {}",
            frames.len()
        );
    }

    let rom_path = Path::new(common::ROM_ROOT).join(&rom);
    if !rom_path.exists() {
        bail!("ROM not found: {}", rom_path.display());
    }

    let mut nes = Nes::builder()
        .framebuffer(FrameBuffer::new(ColorFormat::Rgba8888))
        .build();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;
    if let Some(kind) = palette {
        nes.set_palette(kind.palette());
    }

    let targets: BTreeSet<usize> = frames.iter().copied().collect();
    let max_frame = *frames.last().expect("frames not empty");
    let mut frame = nes.ppu.frame_count() as usize;
    if targets.contains(&frame) {
        let packed = nes
            .try_render_buffer()
            .context("packed render buffer unavailable (swapchain backend)")?;
        let rgb = rgba8888_to_rgb24(packed)?;
        let out_path = format!("{}_f{}.rgb24", out_prefix, frame);
        if let Some(parent) = Path::new(&out_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, &rgb).with_context(|| format!("writing {}", out_path))?;
        eprintln!(
            "NESIUM_RGB_DUMP|frame={}|bytes={}|path={}",
            frame,
            rgb.len(),
            out_path
        );
    }

    while frame < max_frame {
        nes.run_frame(false);
        frame = nes.ppu.frame_count() as usize;
        if !targets.contains(&frame) {
            continue;
        }

        let packed = nes
            .try_render_buffer()
            .context("packed render buffer unavailable (swapchain backend)")?;
        let rgb = rgba8888_to_rgb24(packed)?;
        let out_path = format!("{}_f{}.rgb24", out_prefix, frame);
        if let Some(parent) = Path::new(&out_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, &rgb).with_context(|| format!("writing {}", out_path))?;
        eprintln!(
            "NESIUM_RGB_DUMP|frame={}|bytes={}|path={}",
            frame,
            rgb.len(),
            out_path
        );
    }

    Ok(())
}

#[test]
#[ignore = "debug utility: dumps canonical index/emphasis planes for frame-diff analysis"]
fn dump_index_emphasis_frames() -> Result<()> {
    let rom = std::env::var("NESIUM_IDX_DUMP_ROM")
        .unwrap_or_else(|_| "full_palette/flowing_palette.nes".to_string());
    let frames_csv =
        std::env::var("NESIUM_IDX_DUMP_FRAMES").unwrap_or_else(|_| "60,61".to_string());
    let frames = parse_frames_csv(&frames_csv)?;
    let out_prefix = default_out_prefix("NESIUM_IDX_DUMP_OUT_PREFIX", "nesium_frame_idx");

    if frames.len() < 2 {
        bail!(
            "index/emphasis dump requires at least 2 distinct frames, got {}",
            frames.len()
        );
    }

    let rom_path = Path::new(common::ROM_ROOT).join(&rom);
    if !rom_path.exists() {
        bail!("ROM not found: {}", rom_path.display());
    }

    let mut nes = Nes::default();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;

    let targets: BTreeSet<usize> = frames.iter().copied().collect();
    let max_frame = *frames.last().expect("frames not empty");
    let mut frame = nes.ppu.frame_count() as usize;
    if targets.contains(&frame) {
        let idx = nes.render_index_buffer();
        let emph = nes.render_emphasis_buffer();
        if idx.len() != emph.len() {
            bail!(
                "index/emphasis buffer length mismatch: idx={}, emph={}",
                idx.len(),
                emph.len()
            );
        }

        let idx_path = format!("{}_f{}.idx8", out_prefix, frame);
        let emph_path = format!("{}_f{}.emph8", out_prefix, frame);
        if let Some(parent) = Path::new(&idx_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&idx_path, idx).with_context(|| format!("writing {}", idx_path))?;
        fs::write(&emph_path, emph).with_context(|| format!("writing {}", emph_path))?;
        eprintln!(
            "NESIUM_IDX_DUMP|frame={}|pixels={}|idx_path={}|emph_path={}",
            frame,
            idx.len(),
            idx_path,
            emph_path
        );
    }

    while frame < max_frame {
        nes.run_frame(false);
        frame = nes.ppu.frame_count() as usize;
        if !targets.contains(&frame) {
            continue;
        }

        let idx = nes.render_index_buffer();
        let emph = nes.render_emphasis_buffer();
        if idx.len() != emph.len() {
            bail!(
                "index/emphasis buffer length mismatch: idx={}, emph={}",
                idx.len(),
                emph.len()
            );
        }

        let idx_path = format!("{}_f{}.idx8", out_prefix, frame);
        let emph_path = format!("{}_f{}.emph8", out_prefix, frame);
        if let Some(parent) = Path::new(&idx_path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&idx_path, idx).with_context(|| format!("writing {}", idx_path))?;
        fs::write(&emph_path, emph).with_context(|| format!("writing {}", emph_path))?;
        eprintln!(
            "NESIUM_IDX_DUMP|frame={}|pixels={}|idx_path={}|emph_path={}",
            frame,
            idx.len(),
            idx_path,
            emph_path
        );
    }

    Ok(())
}

#[test]
#[ignore = "debug utility: probes hashes across built-in palettes"]
fn probe_flowing_palette_hashes_by_palette_kind() -> Result<()> {
    let rom = std::env::var("NESIUM_RGB_PROBE_ROM")
        .unwrap_or_else(|_| "full_palette/flowing_palette.nes".to_string());
    let frames_csv =
        std::env::var("NESIUM_RGB_PROBE_FRAMES").unwrap_or_else(|_| "60,180,360,600".to_string());
    let frames = parse_frames_csv(&frames_csv)?;
    if frames.len() < 2 {
        bail!(
            "rgb24 probe requires at least 2 distinct frames, got {}",
            frames.len()
        );
    }

    let rom_path = Path::new(common::ROM_ROOT).join(&rom);
    if !rom_path.exists() {
        bail!("ROM not found: {}", rom_path.display());
    }

    let max_frame = *frames.last().expect("frames not empty");
    for kind in PaletteKind::all() {
        let mut nes = Nes::builder()
            .framebuffer(FrameBuffer::new(ColorFormat::Rgba8888))
            .build();
        nes.load_cartridge_from_file(&rom_path)
            .with_context(|| format!("loading {}", rom_path.display()))?;
        nes.set_palette(kind.palette());

        let targets: BTreeSet<usize> = frames.iter().copied().collect();
        let mut frame = nes.ppu.frame_count() as usize;
        if targets.contains(&frame) {
            let packed = nes
                .try_render_buffer()
                .context("packed render buffer unavailable (swapchain backend)")?;
            let rgb = rgba8888_to_rgb24(packed)?;
            let hash = sha1_base64(&rgb);
            eprintln!(
                "NESIUM_RGB_PROBE|palette={}|frame={}|sha1={}",
                kind.as_str(),
                frame,
                hash
            );
        }

        while frame < max_frame {
            nes.run_frame(false);
            frame = nes.ppu.frame_count() as usize;
            if !targets.contains(&frame) {
                continue;
            }

            let packed = nes
                .try_render_buffer()
                .context("packed render buffer unavailable (swapchain backend)")?;
            let rgb = rgba8888_to_rgb24(packed)?;
            let hash = sha1_base64(&rgb);
            eprintln!(
                "NESIUM_RGB_PROBE|palette={}|frame={}|sha1={}",
                kind.as_str(),
                frame,
                hash
            );
        }
    }

    Ok(())
}
