use nesium_core::{Nes, cartridge, ppu::buffer::ColorFormat};

fn dummy_loop_rom() -> Vec<u8> {
    let mut rom = Vec::with_capacity(16 + 16 * 1024 + 8 * 1024);
    rom.extend_from_slice(b"NES\x1A");
    rom.push(1); // 16 KiB PRG
    rom.push(1); // 8 KiB CHR
    rom.push(0); // mapper 0
    rom.push(0);
    rom.extend_from_slice(&[0; 8]);

    let mut prg = vec![0xEA; 16 * 1024];
    // $8000: JMP $8000
    prg[0] = 0x4C;
    prg[1] = 0x00;
    prg[2] = 0x80;
    // Vectors
    prg[0x3FFA] = 0x00;
    prg[0x3FFB] = 0x80;
    prg[0x3FFC] = 0x00;
    prg[0x3FFD] = 0x80;
    prg[0x3FFE] = 0x00;
    prg[0x3FFF] = 0x80;
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&vec![0u8; 8 * 1024]);
    rom
}

#[test]
fn audio_frame_flush_regression_counts() {
    let cart = cartridge::load_cartridge(dummy_loop_rom()).expect("load dummy cartridge");
    let mut nes = Nes::new(ColorFormat::Rgb555);
    nes.insert_cartridge(cart);

    let counts: Vec<usize> = (0..8).map(|_| nes.run_frame(true).len()).collect();
    // With the per-PPU-frame residual flush aligned to Mesen, the resampled
    // 48 kHz stereo output for a minimal idle ROM follows this stable cadence.
    // Removing the frame-end flush causes the chunk boundary to drift across
    // frames and changes this sequence.
    assert_eq!(counts, vec![1464, 1598, 1598, 1596, 1598, 1598, 1596, 1598]);
}
