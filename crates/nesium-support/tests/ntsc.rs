#![cfg(feature = "ntsc-cpp")]

use nesium_core::ppu::buffer::{ColorFormat, VideoPostProcessor};
use nesium_core::ppu::palette::Color;
use nesium_support::video::filters::{NesNtscPostProcessor, NesNtscPreset};
use nesium_support::video::ntsc::nes_ntsc_out_width;

#[test]
fn nes_ntsc_out_width_matches_expected_for_256() {
    assert_eq!(nes_ntsc_out_width(256), 602);
}

#[test]
fn ntsc_post_processor_outputs_doubled_height_and_respects_pitch() {
    let mut processor = NesNtscPostProcessor::new(NesNtscPreset::Rgb);

    let src_w = 256usize;
    let src_h = 1usize;
    let src = vec![0u8; src_w * src_h];

    let mut palette = [Color::BLACK; 64];
    palette[0] = Color::new(0x12, 0x34, 0x56);

    let dst_w = nes_ntsc_out_width(src_w);
    let dst_h = src_h * 2;
    let row_bytes = dst_w * 4;
    let dst_pitch = row_bytes + 16;
    let mut dst = vec![0xAAu8; dst_pitch * dst_h];

    processor.process(
        &src,
        src_w,
        src_h,
        &palette,
        &mut dst,
        dst_pitch,
        dst_w,
        dst_h,
        ColorFormat::Rgba8888,
    );

    let row0 = &dst[0..row_bytes];
    let row1 = &dst[dst_pitch..dst_pitch + row_bytes];
    assert_eq!(row0, row1, "NTSC filter should duplicate scanlines");

    for x in 0..dst_w {
        let off = x * 4;
        assert_eq!(row0[off + 3], 0xFF);
    }

    // For a solid input field, the output should be near-uniform in the center region.
    // The NTSC kernel can introduce edge roll-off at the far left/right.
    let margin = 32usize.min(dst_w / 4);
    let center_start = margin;
    let center_end = dst_w - margin;
    let mut min_r = 255u8;
    let mut max_r = 0u8;
    let mut min_g = 255u8;
    let mut max_g = 0u8;
    let mut min_b = 255u8;
    let mut max_b = 0u8;
    for x in center_start..center_end {
        let off = x * 4;
        let r = row0[off];
        let g = row0[off + 1];
        let b = row0[off + 2];
        min_r = min_r.min(r);
        max_r = max_r.max(r);
        min_g = min_g.min(g);
        max_g = max_g.max(g);
        min_b = min_b.min(b);
        max_b = max_b.max(b);
    }
    assert!((max_r as i16 - min_r as i16) <= 8);
    assert!((max_g as i16 - min_g as i16) <= 8);
    assert!((max_b as i16 - min_b as i16) <= 8);

    // Padding bytes must be untouched.
    for y in 0..dst_h {
        let pad = &dst[y * dst_pitch + row_bytes..(y + 1) * dst_pitch];
        assert!(pad.iter().all(|&b| b == 0xAA));
    }
}
