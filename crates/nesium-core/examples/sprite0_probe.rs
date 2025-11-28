use nesium_core::Nes;
use std::{env, path::Path};

fn main() -> anyhow::Result<()> {
    let rom = env::args()
        .nth(1)
        .expect("usage: sprite0_probe <rom> [frames]");
    let frames: usize = env::args()
        .nth(2)
        .map(|s| s.parse().unwrap_or(120))
        .unwrap_or(120);

    let mut nes = Nes::default();
    nes.load_cartridge_from_file(Path::new(&rom))?;

    let mut last_hit: Option<nesium_core::ppu::Sprite0HitDebug> = None;
    for _ in 0..frames {
        nes.run_frame();
        last_hit = nes.sprite0_hit_pos();
    }

    println!("After {frames} frame(s):");
    match last_hit {
        Some(hit) => {
            let frame = nes.ppu_nmi_debug().frame;
            println!(
                "Sprite0 hit at frame {}: scanline={} cycle={} oam=[{:02X},{:02X},{:02X},{:02X}]",
                frame,
                hit.pos.scanline,
                hit.pos.cycle,
                hit.oam[0],
                hit.oam[1],
                hit.oam[2],
                hit.oam[3],
            );
        }
        None => println!("Sprite0 hit not observed"),
    }

    Ok(())
}
