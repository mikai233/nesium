#![cfg(feature = "hqx-cpp")]

use nesium_core::ppu::buffer::{ColorFormat, VideoPostProcessor};
use nesium_core::ppu::palette::Color;
use nesium_support::video::filters::HqxPostProcessor;
use nesium_support::video::hqx::{HqxError, HqxScale, hqx_scale_argb8888};

fn solid_palette(color: Color) -> [Color; 64] {
    let mut palette = [Color::BLACK; 64];
    palette[0] = color;
    palette
}

#[test]
fn hqx_scale_argb8888_rejects_invalid_sizes() {
    let w = 4usize;
    let h = 3usize;
    let src = vec![0xFF11_2233u32; w * h];

    // Wrong input size.
    let mut dst = vec![0u32; (w * 2) * (h * 2)];
    let err = hqx_scale_argb8888(HqxScale::X2, &src[..src.len() - 1], w, h, &mut dst).unwrap_err();
    matches!(err, HqxError::InvalidInputSize { .. });

    // Wrong output size.
    let mut dst = vec![0u32; (w * 2) * (h * 2) - 1];
    let err = hqx_scale_argb8888(HqxScale::X2, &src, w, h, &mut dst).unwrap_err();
    matches!(err, HqxError::InvalidOutputSize { .. });
}

#[test]
fn hqx_scale_argb8888_solid_color_is_solid() {
    let w = 5usize;
    let h = 4usize;
    let src_color = 0xFF12_3456u32;
    let src = vec![src_color; w * h];

    for scale in [HqxScale::X2, HqxScale::X3, HqxScale::X4] {
        let s = scale as usize;
        let mut dst = vec![0u32; (w * s) * (h * s)];
        hqx_scale_argb8888(scale, &src, w, h, &mut dst).unwrap();
        assert!(dst.iter().all(|&p| p == src_color));
    }
}

#[test]
fn hqx_scale_argb8888_is_deterministic() {
    let w = 6usize;
    let h = 6usize;

    let mut src = vec![0u32; w * h];
    for y in 0..h {
        for x in 0..w {
            let r = (x * 17) as u8;
            let g = (y * 19) as u8;
            let b = ((x ^ y) * 23) as u8;
            src[y * w + x] = 0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        }
    }

    let mut out1 = vec![0u32; (w * 2) * (h * 2)];
    let mut out2 = vec![0u32; (w * 2) * (h * 2)];
    hqx_scale_argb8888(HqxScale::X2, &src, w, h, &mut out1).unwrap();
    hqx_scale_argb8888(HqxScale::X2, &src, w, h, &mut out2).unwrap();
    assert_eq!(out1, out2);
}

#[test]
fn hqx_post_processor_packs_rgba_and_respects_pitch() {
    let mut processor = HqxPostProcessor::new(HqxScale::X2);
    let palette = solid_palette(Color::new(0x11, 0x22, 0x33));

    // 2x2 indices -> 4x4 pixels
    let src_w = 2usize;
    let src_h = 2usize;
    let src = vec![0u8; src_w * src_h];

    let dst_w = src_w * 2;
    let dst_h = src_h * 2;
    let bpp = 4usize;
    let row_bytes = dst_w * bpp;
    let dst_pitch = row_bytes + 8;
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

    // Output should be solid and alpha set to 255.
    for y in 0..dst_h {
        let row = &dst[y * dst_pitch..y * dst_pitch + row_bytes];
        for x in 0..dst_w {
            let off = x * 4;
            assert_eq!(row[off], 0x11);
            assert_eq!(row[off + 1], 0x22);
            assert_eq!(row[off + 2], 0x33);
            assert_eq!(row[off + 3], 0xFF);
        }

        // Padding bytes must be untouched.
        let pad = &dst[y * dst_pitch + row_bytes..(y + 1) * dst_pitch];
        assert!(pad.iter().all(|&b| b == 0xAA));
    }
}
