use std::{fs, path::Path};

use anyhow::{Result, anyhow};
use nesium_core::{
    CpuSnapshot, Nes,
    ppu::{buffer::ColorFormat, palette::PaletteKind},
};

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

pub fn run_trace<P: AsRef<Path>>(rom_path: P, log_path: P) -> Result<()> {
    let mut nes = Nes::default();
    nes.load_cartridge_from_file(rom_path)?;

    let log = fs::read_to_string(log_path)?;
    let trace_rows: Vec<_> = log.lines().filter_map(parse_trace_line).collect();
    if trace_rows.is_empty() {
        return Err(anyhow!("trace log appears empty or unparsable"));
    }

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

pub fn run_frame_report<P: AsRef<Path>>(rom: P, frames: usize) -> Result<()> {
    let mut nes = Nes::new(ColorFormat::Argb8888);
    nes.set_palette(PaletteKind::RawLinear.palette());
    nes.load_cartridge_from_file(rom)?;

    for _ in 0..frames {
        nes.run_frame(false);
    }

    let fb = nes.render_buffer();
    let mut counts = vec![0usize; 256];
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
