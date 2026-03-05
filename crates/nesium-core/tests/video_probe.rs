use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use nesium_core::Nes;
use nesium_core::cartridge::{mapper::Mapper26, mapper_downcast_ref};
use nesium_core::controller::Button;
use nesium_core::ppu::buffer::{ColorFormat, FrameBuffer};
use nesium_core::ppu::palette::PaletteKind;

fn parse_frames_csv(text: &str) -> Result<Vec<usize>> {
    let mut frames = Vec::new();
    for token in text.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let frame = trimmed
            .parse::<usize>()
            .with_context(|| format!("invalid frame '{trimmed}'"))?;
        frames.push(frame);
    }
    if frames.is_empty() {
        bail!("frame list is empty");
    }
    frames.sort_unstable();
    frames.dedup();
    Ok(frames)
}

fn fnv1a32_rgb24_from_rgba8888(frame_rgba: &[u8]) -> Result<u32> {
    if !frame_rgba.len().is_multiple_of(4) {
        bail!(
            "rgba buffer length must be multiple of 4, got {}",
            frame_rgba.len()
        );
    }
    let mut hash: u32 = 0x811C9DC5;
    for px in frame_rgba.chunks_exact(4) {
        for b in [px[0], px[1], px[2]] {
            hash ^= u32::from(b);
            hash = hash.wrapping_mul(0x01000193);
        }
    }
    Ok(hash)
}

fn resolve_rom_path(rom_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(rom_path);
    if path.is_absolute() {
        if path.exists() {
            return Ok(path);
        }
        bail!("ROM not found: {}", path.display());
    }

    let joined = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/nes-test-roms")
        .join(path);
    if !joined.exists() {
        bail!("ROM not found: {}", joined.display());
    }
    Ok(joined)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InputEvent {
    frame: usize,
    pad: usize,
    state: u8,
}

fn parse_u8_auto(text: &str) -> Result<u8> {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return u8::from_str_radix(hex, 16).with_context(|| format!("invalid hex byte '{text}'"));
    }
    text.parse::<u8>()
        .with_context(|| format!("invalid byte '{text}'"))
}

fn parse_input_event_token(token: &str) -> Result<InputEvent> {
    let mut parts = token.split(':');
    let first = parts
        .next()
        .with_context(|| format!("invalid input token '{token}'"))?;
    let second = parts
        .next()
        .with_context(|| format!("invalid input token '{token}'"))?;
    let third = parts.next();

    let (frame_text, pad_text, state_text) = if let Some(state) = third {
        (first, second, state)
    } else {
        (first, "0", second)
    };

    if parts.next().is_some() {
        bail!("invalid input token '{}': too many ':'", token);
    }

    let frame_signed = frame_text
        .parse::<isize>()
        .with_context(|| format!("invalid frame '{}' in token '{}'", frame_text, token))?;
    let frame = if frame_signed < 0 {
        0
    } else {
        frame_signed as usize
    };
    let pad = pad_text
        .parse::<usize>()
        .with_context(|| format!("invalid pad '{}' in token '{}'", pad_text, token))?;
    let state = parse_u8_auto(state_text)?;

    Ok(InputEvent { frame, pad, state })
}

fn load_video_probe_input_events() -> Result<Vec<InputEvent>> {
    let input_file = std::env::var("NESIUM_VIDEO_PROBE_INPUT_FILE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let input_csv = std::env::var("NESIUM_VIDEO_PROBE_INPUT_EVENTS")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let input_frame_offset = std::env::var("NESIUM_VIDEO_PROBE_INPUT_FRAME_OFFSET")
        .ok()
        .map(|v| {
            v.parse::<i64>()
                .with_context(|| format!("invalid NESIUM_VIDEO_PROBE_INPUT_FRAME_OFFSET '{}'", v))
        })
        .transpose()?
        .unwrap_or(0);

    let mut tokens = Vec::new();
    if let Some(path) = input_file.as_deref() {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading NESIUM_VIDEO_PROBE_INPUT_FILE '{}'", path))?;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            tokens.push(trimmed.to_string());
        }
    } else if let Some(csv) = input_csv.as_deref() {
        for token in csv.split(',') {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                continue;
            }
            tokens.push(trimmed.to_string());
        }
    }

    let mut events = Vec::with_capacity(tokens.len());
    for token in tokens {
        events.push(parse_input_event_token(&token)?);
    }

    if input_frame_offset != 0 {
        for evt in &mut events {
            if input_frame_offset < 0 {
                let amount = input_frame_offset.unsigned_abs() as usize;
                evt.frame = evt.frame.saturating_sub(amount);
            } else {
                evt.frame = evt.frame.saturating_add(input_frame_offset as usize);
            }
        }
    }

    events.sort_unstable_by_key(|e| (e.frame, e.pad));
    Ok(events)
}

fn set_pad_state(nes: &mut Nes, pad: usize, state: u8) {
    const BUTTONS: [Button; 8] = [
        Button::A,
        Button::B,
        Button::Select,
        Button::Start,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ];

    for (bit, button) in BUTTONS.into_iter().enumerate() {
        let pressed = (state & (1u8 << bit)) != 0;
        nes.set_button(pad, button, pressed);
    }
}

fn apply_input_events_until_frame(
    nes: &mut Nes,
    input_events: &[InputEvent],
    input_idx: &mut usize,
    frame: usize,
) {
    // Align with Mesen scripts: events tagged for logical frame N are applied
    // before running frame N.
    let apply_until = frame.saturating_add(1);
    while *input_idx < input_events.len() && input_events[*input_idx].frame <= apply_until {
        let evt = input_events[*input_idx];
        if evt.pad < 2 {
            set_pad_state(nes, evt.pad, evt.state);
        }
        *input_idx += 1;
    }
}

#[test]
#[ignore = "manual video rgb24 hash probe"]
fn video_rgb24_hash_probe() -> Result<()> {
    let rom = std::env::var("NESIUM_VIDEO_PROBE_ROM")
        .context("missing NESIUM_VIDEO_PROBE_ROM (absolute path or vendor-relative path)")?;
    let frames_csv =
        std::env::var("NESIUM_VIDEO_PROBE_FRAMES").context("missing NESIUM_VIDEO_PROBE_FRAMES")?;

    let frames = parse_frames_csv(&frames_csv)?;
    let rom_path = resolve_rom_path(&rom)?;

    let mut nes = Nes::builder()
        .framebuffer(FrameBuffer::new(ColorFormat::Rgba8888))
        .build();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;
    nes.set_palette(PaletteKind::Mesen2C02.palette());

    let max_frame = *frames.last().expect("frames not empty");
    let mut target_idx = 0usize;
    let input_events = load_video_probe_input_events()?;
    let mut input_idx = 0usize;

    println!("[video-probe] rom={rom}");
    let mut frame = nes.ppu.frame_count() as usize;
    apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);
    while target_idx < frames.len() && frame == frames[target_idx] {
        let packed = nes
            .try_render_buffer()
            .context("packed render buffer unavailable (swapchain backend)")?;
        let hash = fnv1a32_rgb24_from_rgba8888(packed)?;
        println!("[video-probe] frame={frame} hash={hash:08x}");
        target_idx += 1;
    }

    while target_idx < frames.len() && frame < max_frame {
        nes.run_frame(false);
        frame = nes.ppu.frame_count() as usize;
        apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);

        while target_idx < frames.len() && frame == frames[target_idx] {
            let packed = nes
                .try_render_buffer()
                .context("packed render buffer unavailable (swapchain backend)")?;
            let hash = fnv1a32_rgb24_from_rgba8888(packed)?;
            println!("[video-probe] frame={frame} hash={hash:08x}");
            target_idx += 1;
        }
    }

    if target_idx != frames.len() {
        bail!(
            "failed to capture all requested frames: captured {} of {}",
            target_idx,
            frames.len()
        );
    }

    Ok(())
}

#[test]
#[ignore = "manual video cpu/ppu state probe"]
fn video_cpu_ppu_state_probe() -> Result<()> {
    let rom = std::env::var("NESIUM_VIDEO_PROBE_ROM")
        .context("missing NESIUM_VIDEO_PROBE_ROM (absolute path or vendor-relative path)")?;
    let frames_csv =
        std::env::var("NESIUM_VIDEO_PROBE_FRAMES").context("missing NESIUM_VIDEO_PROBE_FRAMES")?;

    let frames = parse_frames_csv(&frames_csv)?;
    let rom_path = resolve_rom_path(&rom)?;

    let mut nes = Nes::builder()
        .framebuffer(FrameBuffer::new(ColorFormat::Rgba8888))
        .build();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;
    nes.set_palette(PaletteKind::Mesen2C02.palette());

    let max_frame = *frames.last().expect("frames not empty");
    let mut target_idx = 0usize;
    let input_events = load_video_probe_input_events()?;
    let mut input_idx = 0usize;

    println!("[video-state-probe] rom={rom}");
    let mut frame = nes.ppu.frame_count() as usize;
    apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);

    while target_idx < frames.len() && frame == frames[target_idx] {
        let cpu = nes.cpu_snapshot();
        let (scanline, dot, _ppu_frame, _ctrl, _mask, _status, _oam_addr, v, t, _x) =
            nes.ppu_debug_state();
        let mapper = nes
            .get_cartridge()
            .and_then(|c| mapper_downcast_ref::<Mapper26>(c.mapper()))
            .map(|m| m.debug_state());
        println!(
            "[video-state-probe] frame={frame} pc={:04x} a={:02x} x={:02x} y={:02x} sp={:02x} p={:02x} scanline={} dot={} v={:04x} t={:04x} mapper={:?}",
            cpu.pc, cpu.a, cpu.x, cpu.y, cpu.s, cpu.p, scanline, dot, v, t, mapper
        );
        target_idx += 1;
    }

    while target_idx < frames.len() && frame < max_frame {
        nes.run_frame(false);
        frame = nes.ppu.frame_count() as usize;

        apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);

        while target_idx < frames.len() && frame == frames[target_idx] {
            let cpu = nes.cpu_snapshot();
            let (scanline, dot, _ppu_frame, _ctrl, _mask, _status, _oam_addr, v, t, _x) =
                nes.ppu_debug_state();
            let mapper = nes
                .get_cartridge()
                .and_then(|c| mapper_downcast_ref::<Mapper26>(c.mapper()))
                .map(|m| m.debug_state());
            println!(
                "[video-state-probe] frame={frame} pc={:04x} a={:02x} x={:02x} y={:02x} sp={:02x} p={:02x} scanline={} dot={} v={:04x} t={:04x} mapper={:?}",
                cpu.pc, cpu.a, cpu.x, cpu.y, cpu.s, cpu.p, scanline, dot, v, t, mapper
            );
            target_idx += 1;
        }
    }

    if target_idx != frames.len() {
        bail!(
            "failed to capture all requested frames: captured {} of {}",
            target_idx,
            frames.len()
        );
    }

    Ok(())
}

#[test]
#[ignore = "manual video rgb24 dump probe"]
fn video_rgb24_dump_probe() -> Result<()> {
    let rom = std::env::var("NESIUM_VIDEO_PROBE_ROM")
        .context("missing NESIUM_VIDEO_PROBE_ROM (absolute path or vendor-relative path)")?;
    let frames_csv =
        std::env::var("NESIUM_VIDEO_PROBE_FRAMES").context("missing NESIUM_VIDEO_PROBE_FRAMES")?;
    let out_prefix = std::env::var("NESIUM_VIDEO_PROBE_RGB_OUT_PREFIX")
        .context("missing NESIUM_VIDEO_PROBE_RGB_OUT_PREFIX")?;

    let frames = parse_frames_csv(&frames_csv)?;
    let rom_path = resolve_rom_path(&rom)?;

    let mut nes = Nes::builder()
        .framebuffer(FrameBuffer::new(ColorFormat::Rgba8888))
        .build();
    nes.load_cartridge_from_file(&rom_path)
        .with_context(|| format!("loading {}", rom_path.display()))?;
    nes.set_palette(PaletteKind::Mesen2C02.palette());

    let max_frame = *frames.last().expect("frames not empty");
    let mut target_idx = 0usize;
    let input_events = load_video_probe_input_events()?;
    let mut input_idx = 0usize;

    let prefix = PathBuf::from(out_prefix);
    if let Some(parent) = prefix.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create output dir {}", parent.display()))?;
    }

    println!("[video-rgb-probe] rom={rom}");
    let mut frame = nes.ppu.frame_count() as usize;
    apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);

    while target_idx < frames.len() && frame == frames[target_idx] {
        let packed = nes
            .try_render_buffer()
            .context("packed render buffer unavailable (swapchain backend)")?;
        let mut rgb = Vec::with_capacity(packed.len() / 4 * 3);
        for px in packed.chunks_exact(4) {
            rgb.push(px[0]);
            rgb.push(px[1]);
            rgb.push(px[2]);
        }
        let path = prefix.with_file_name(format!(
            "{}_f{}.rgb24",
            prefix.file_name().unwrap_or_default().to_string_lossy(),
            frame
        ));
        std::fs::write(&path, &rgb).with_context(|| format!("writing {}", path.display()))?;
        println!(
            "[video-rgb-probe] frame={frame} bytes={} path={}",
            rgb.len(),
            path.display()
        );
        target_idx += 1;
    }

    while target_idx < frames.len() && frame < max_frame {
        nes.run_frame(false);
        frame = nes.ppu.frame_count() as usize;
        apply_input_events_until_frame(&mut nes, &input_events, &mut input_idx, frame);

        while target_idx < frames.len() && frame == frames[target_idx] {
            let packed = nes
                .try_render_buffer()
                .context("packed render buffer unavailable (swapchain backend)")?;
            let mut rgb = Vec::with_capacity(packed.len() / 4 * 3);
            for px in packed.chunks_exact(4) {
                rgb.push(px[0]);
                rgb.push(px[1]);
                rgb.push(px[2]);
            }
            let path = prefix.with_file_name(format!(
                "{}_f{}.rgb24",
                prefix.file_name().unwrap_or_default().to_string_lossy(),
                frame
            ));
            std::fs::write(&path, &rgb).with_context(|| format!("writing {}", path.display()))?;
            println!(
                "[video-rgb-probe] frame={frame} bytes={} path={}",
                rgb.len(),
                path.display()
            );
            target_idx += 1;
        }
    }

    if target_idx != frames.len() {
        bail!(
            "failed to capture all requested frames: captured {} of {}",
            target_idx,
            frames.len()
        );
    }

    Ok(())
}
