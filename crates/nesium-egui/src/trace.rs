use std::path::Path;

use anyhow::Result;
use nesium_core::{
    Nes,
    ppu::{buffer::ColorFormat, palette::PaletteKind},
};

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
