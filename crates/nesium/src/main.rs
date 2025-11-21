use std::{
    env, fs, thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use nesium_core::{
    CpuSnapshot, NES,
    controller::Button,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, palette::PaletteKind},
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

const WINDOW_SCALE: u32 = 3;
const TARGET_FRAME: Duration = Duration::from_nanos(16_683_000); // ~59.94 Hz

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut rom_path: Option<String> = None;
    let mut trace_log: Option<String> = None;
    let mut start_pc: Option<u16> = None;
    let mut check_frames: Option<usize> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trace-log" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--trace-log requires a log file path"))?;
                trace_log = Some(path);
            }
            "--start-pc" => {
                let pc_str = args
                    .next()
                    .ok_or_else(|| anyhow!("--start-pc requires a hex address (e.g. 0xC000)"))?;
                let pc = pc_str.trim_start_matches("0x");
                start_pc = Some(
                    u16::from_str_radix(pc, 16)
                        .map_err(|_| anyhow!("--start-pc expects a hex address, got {pc_str}"))?,
                );
            }
            "--check-frame" => {
                let frames = args
                    .next()
                    .ok_or_else(|| anyhow!("--check-frame requires a frame count"))?;
                check_frames = Some(frames.parse()?);
            }
            _ if rom_path.is_none() => rom_path = Some(arg),
            _ => return Err(anyhow!("unexpected argument: {arg}")),
        }
    }

    let rom_path = rom_path.ok_or_else(|| {
        anyhow!(
            "usage: nesium <path-to-rom.nes> [--trace-log <path-to-nestest.log>] [--start-pc <hex>] [--check-frame <n>]"
        )
    })?;

    // Trace mode: compare CPU state to nestest.log instead of opening a window.
    if let Some(log_path) = trace_log {
        return run_trace(&rom_path, &log_path);
    }

    let mut nes = NES::new();
    nes.set_palette_kind(PaletteKind::NesdevNtsc);
    nes.load_cartridge_from_file(&rom_path)?;
    if let Some(pc) = start_pc {
        let snapshot = CpuSnapshot {
            pc,
            a: 0,
            x: 0,
            y: 0,
            s: 0xFD,
            p: 0x24,
        };
        nes.set_cpu_snapshot(snapshot);
    }

    if let Some(frames) = check_frames {
        return run_frame_report(nes, frames);
    }

    let sdl = sdl2::init().map_err(|e| anyhow!("initializing SDL2: {e}"))?;
    let video = sdl
        .video()
        .map_err(|e| anyhow!("initializing video subsystem: {e}"))?;
    let window = video
        .window(
            "Nesium",
            SCREEN_WIDTH as u32 * WINDOW_SCALE,
            SCREEN_HEIGHT as u32 * WINDOW_SCALE,
        )
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| anyhow!("creating SDL2 window: {e}"))?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| anyhow!("creating renderer: {e}"))?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            // ABGR8888 stores bytes in RGBA order on little-endian hosts, matching our writes below.
            PixelFormatEnum::ABGR8888,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
        .map_err(|e| anyhow!("allocating texture: {e}"))?;

    let mut event_pump = sdl
        .event_pump()
        .map_err(|e| anyhow!("creating event pump: {e}"))?;

    'running: loop {
        let frame_start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        if let Some(button) = map_key_to_button(key) {
                            nes.set_button(0, button, true);
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key),
                    repeat,
                    ..
                } => {
                    if !repeat {
                        if let Some(button) = map_key_to_button(key) {
                            nes.set_button(0, button, false);
                        }
                    }
                }
                _ => {}
            }
        }

        nes.run_frame();

        let palette = *nes.palette();
        let frame = nes.framebuffer();
        texture
            .with_lock(None, |buffer, pitch| {
                let pitch = pitch as usize;
                for y in 0..SCREEN_HEIGHT {
                    let src_row = &frame[y * SCREEN_WIDTH..(y + 1) * SCREEN_WIDTH];
                    let dst_row = &mut buffer[y * pitch..y * pitch + SCREEN_WIDTH * 4];
                    for (x, &index) in src_row.iter().enumerate() {
                        let color = palette.color(index);
                        let base = x * 4;
                        dst_row[base] = color.r;
                        dst_row[base + 1] = color.g;
                        dst_row[base + 2] = color.b;
                        dst_row[base + 3] = 0xFF;
                    }
                }
            })
            .map_err(|e| anyhow!("uploading frame to texture: {e}"))?;

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|e| anyhow!("copying texture to canvas: {e}"))?;
        canvas.present();

        // Frame pacing: VSYNC already blocks on present(), but on mismatched refresh rates
        // or headless paths we can still drift. Sleep the remainder toward 59.94 Hz.
        let elapsed = frame_start.elapsed();
        if elapsed < TARGET_FRAME {
            thread::sleep(TARGET_FRAME - elapsed);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TraceRow {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    sp: u8,
}

impl From<TraceRow> for CpuSnapshot {
    fn from(value: TraceRow) -> Self {
        CpuSnapshot {
            pc: value.pc,
            a: value.a,
            x: value.x,
            y: value.y,
            s: value.sp,
            p: value.p,
        }
    }
}

impl From<CpuSnapshot> for TraceRow {
    fn from(value: CpuSnapshot) -> Self {
        TraceRow {
            pc: value.pc,
            a: value.a,
            x: value.x,
            y: value.y,
            p: value.p,
            sp: value.s,
        }
    }
}

fn parse_hex_u16(token: &str) -> Option<u16> {
    u16::from_str_radix(token, 16).ok()
}

fn parse_hex_u8(token: &str) -> Option<u8> {
    u8::from_str_radix(token, 16).ok()
}

fn map_key_to_button(key: Keycode) -> Option<Button> {
    match key {
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::Return => Some(Button::Start),
        Keycode::RShift | Keycode::LCtrl | Keycode::RCtrl => Some(Button::Select),
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        _ => None,
    }
}

fn parse_trace_line(line: &str) -> Option<TraceRow> {
    let mut parts = line.split_whitespace();
    let pc = parse_hex_u16(parts.next()?)?;

    let mut a = None;
    let mut x = None;
    let mut y = None;
    let mut p = None;
    let mut sp = None;

    for token in line.split_whitespace() {
        if let Some(val) = token.strip_prefix("A:") {
            a = parse_hex_u8(val);
        } else if let Some(val) = token.strip_prefix("X:") {
            x = parse_hex_u8(val);
        } else if let Some(val) = token.strip_prefix("Y:") {
            y = parse_hex_u8(val);
        } else if let Some(val) = token.strip_prefix("P:") {
            p = parse_hex_u8(val);
        } else if let Some(val) = token.strip_prefix("SP:") {
            sp = parse_hex_u8(val);
        }
    }

    Some(TraceRow {
        pc,
        a: a?,
        x: x?,
        y: y?,
        p: p?,
        sp: sp?,
    })
}

fn run_trace(rom_path: &str, log_path: &str) -> Result<()> {
    let mut nes = NES::new();
    nes.set_palette_kind(PaletteKind::NesdevNtsc);
    nes.load_cartridge_from_file(rom_path)?;

    let log = fs::read_to_string(log_path)?;
    let trace_rows: Vec<_> = log.lines().filter_map(parse_trace_line).collect();
    if trace_rows.is_empty() {
        return Err(anyhow!("trace log appears empty or unparsable"));
    }

    // Seed CPU state to the first log entry (nestest expects manual start at $C000).
    let first = trace_rows[0];
    nes.set_cpu_snapshot(first.into());

    for (idx, expected) in trace_rows.iter().enumerate() {
        let actual = nes.cpu_snapshot();
        let actual_row: TraceRow = actual.into();

        if actual_row != *expected {
            println!("Mismatch at instruction {idx}");
            println!(
                "Expected PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                expected.pc, expected.a, expected.x, expected.y, expected.p, expected.sp
            );
            println!(
                "Actual   PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                actual_row.pc,
                actual_row.a,
                actual_row.x,
                actual_row.y,
                actual_row.p,
                actual_row.sp
            );
            if let Some(line) = log.lines().nth(idx) {
                println!("Source log line: {}", line);
            }
            return Ok(());
        }

        nes.step_instruction();
    }

    println!("Trace matched all log entries");
    Ok(())
}

fn run_frame_report(mut nes: NES, frames: usize) -> Result<()> {
    for _ in 0..frames {
        nes.run_frame();
    }

    let fb = nes.framebuffer();
    let mut counts = [0usize; 64];
    for &idx in fb {
        counts[idx as usize] += 1;
    }

    let mut entries: Vec<(u8, usize)> = counts
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0)
        .map(|(i, &c)| (i as u8, c))
        .collect();
    entries.sort_by_key(|&(_, c)| std::cmp::Reverse(c));

    println!("Frame report after {frames} frame(s):");
    for (i, (color, count)) in entries.iter().take(8).enumerate() {
        println!("{:>2}. index {:02X} count {}", i + 1, color, count);
    }

    if let Some((top_idx, top_count)) = entries.first() {
        let percent = (*top_count as f64 / fb.len() as f64) * 100.0;
        println!(
            "Dominant color index {:02X}: {} pixels ({percent:.2}%)",
            top_idx, top_count
        );
    }

    Ok(())
}
