#![cfg(feature = "sai")]

use nesium_core::ppu::buffer::{ColorFormat, VideoPostProcessor};
use nesium_core::ppu::palette::Color;
use nesium_support::video::filters::{SaiPostProcessor, SaiVariant};

fn solid_palette(color: Color) -> [Color; 64] {
    let mut palette = [Color::BLACK; 64];
    palette[0] = color;
    palette
}

#[test]
fn sai_filters_solid_color_is_solid() {
    let src_w = 8usize;
    let src_h = 6usize;
    let src = vec![0u8; src_w * src_h];
    let palette = solid_palette(Color {
        r: 0x12,
        g: 0x34,
        b: 0x56,
    });

    for variant in [
        SaiVariant::Sai2x,
        SaiVariant::Super2xSai,
        SaiVariant::SuperEagle,
    ] {
        let mut processor = SaiPostProcessor::new(variant);
        let dst_w = src_w * 2;
        let dst_h = src_h * 2;
        let mut dst = vec![0u8; dst_w * dst_h * 4];
        processor.process(
            &src,
            src_w,
            src_h,
            &palette,
            &mut dst,
            dst_w * 4,
            dst_w,
            dst_h,
            ColorFormat::Rgba8888,
        );

        for px in dst.chunks_exact(4) {
            assert_eq!(px, &[0x12, 0x34, 0x56, 0xFF]);
        }
    }
}
