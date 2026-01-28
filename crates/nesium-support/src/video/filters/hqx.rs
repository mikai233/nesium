use nesium_core::ppu::buffer::{ColorFormat, NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::palette::Color;

use crate::video::hqx::{HqxScale, hqx_scale_argb8888};

#[derive(Debug, Clone)]
pub struct HqxPostProcessor {
    scale: HqxScale,
    scratch: HqxScratch,
    fallback: NearestPostProcessor,
}

#[derive(Debug, Default, Clone)]
struct HqxScratch {
    input_argb: Vec<u32>,
    output_argb: Vec<u32>,
}

impl HqxPostProcessor {
    pub fn new(scale: HqxScale) -> Self {
        Self {
            scale,
            scratch: HqxScratch::default(),
            fallback: NearestPostProcessor,
        }
    }
}

impl VideoPostProcessor for HqxPostProcessor {
    fn process(
        &mut self,
        src_indices: &[u8],
        src_width: usize,
        src_height: usize,
        palette: &[Color; 64],
        dst: &mut [u8],
        dst_pitch: usize,
        dst_width: usize,
        dst_height: usize,
        dst_format: ColorFormat,
    ) {
        let scale = self.scale as usize;
        if scale == 0 || src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return;
        }

        let expected_w = src_width.saturating_mul(scale);
        let expected_h = src_height.saturating_mul(scale);
        if dst_width != expected_w || dst_height != expected_h {
            self.fallback.process(
                src_indices,
                src_width,
                src_height,
                palette,
                dst,
                dst_pitch,
                dst_width,
                dst_height,
                dst_format,
            );
            return;
        }

        let bpp = dst_format.bytes_per_pixel();
        let row_bytes = dst_width.saturating_mul(bpp);
        if dst_pitch < row_bytes {
            return;
        }
        let required = match dst_pitch.checked_mul(dst_height) {
            Some(v) => v,
            None => return,
        };
        if dst.len() < required {
            return;
        }

        // HQX expects ARGB8888 pixels (0xAARRGGBB). Convert indices -> ARGB.
        let expected_in = match src_width.checked_mul(src_height) {
            Some(v) => v,
            None => {
                self.fallback.process(
                    src_indices,
                    src_width,
                    src_height,
                    palette,
                    dst,
                    dst_pitch,
                    dst_width,
                    dst_height,
                    dst_format,
                );
                return;
            }
        };
        if src_indices.len() != expected_in {
            self.fallback.process(
                src_indices,
                src_width,
                src_height,
                palette,
                dst,
                dst_pitch,
                dst_width,
                dst_height,
                dst_format,
            );
            return;
        }

        let expected_out = match dst_width.checked_mul(dst_height) {
            Some(v) => v,
            None => return,
        };

        let scratch = &mut self.scratch;
        scratch.input_argb.resize(expected_in, 0);
        scratch.output_argb.resize(expected_out, 0);

        for (i, &idx) in src_indices.iter().enumerate() {
            let c = palette[(idx & 0x3F) as usize];
            scratch.input_argb[i] =
                0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
        }

        let scale_result = hqx_scale_argb8888(
            self.scale,
            scratch.input_argb.as_slice(),
            src_width,
            src_height,
            scratch.output_argb.as_mut_slice(),
        );
        if scale_result.is_err() {
            self.fallback.process(
                src_indices,
                src_width,
                src_height,
                palette,
                dst,
                dst_pitch,
                dst_width,
                dst_height,
                dst_format,
            );
            return;
        }

        // Pack ARGB to the requested destination format.
        match dst_format {
            ColorFormat::Rgba8888 => {
                for y in 0..dst_height {
                    let row_src = &scratch.output_argb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &argb) in row_src.iter().enumerate() {
                        let r = ((argb >> 16) & 0xFF) as u8;
                        let g = ((argb >> 8) & 0xFF) as u8;
                        let b = (argb & 0xFF) as u8;
                        let off = x * 4;
                        row_dst[off] = r;
                        row_dst[off + 1] = g;
                        row_dst[off + 2] = b;
                        row_dst[off + 3] = 0xFF;
                    }
                }
            }
            ColorFormat::Bgra8888 => {
                for y in 0..dst_height {
                    let row_src = &scratch.output_argb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &argb) in row_src.iter().enumerate() {
                        let r = ((argb >> 16) & 0xFF) as u8;
                        let g = ((argb >> 8) & 0xFF) as u8;
                        let b = (argb & 0xFF) as u8;
                        let off = x * 4;
                        row_dst[off] = b;
                        row_dst[off + 1] = g;
                        row_dst[off + 2] = r;
                        row_dst[off + 3] = 0xFF;
                    }
                }
            }
            ColorFormat::Argb8888 => {
                for y in 0..dst_height {
                    let row_src = &scratch.output_argb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &argb) in row_src.iter().enumerate() {
                        let a = 0xFFu8;
                        let r = ((argb >> 16) & 0xFF) as u8;
                        let g = ((argb >> 8) & 0xFF) as u8;
                        let b = (argb & 0xFF) as u8;
                        let off = x * 4;
                        row_dst[off] = a;
                        row_dst[off + 1] = r;
                        row_dst[off + 2] = g;
                        row_dst[off + 3] = b;
                    }
                }
            }
            _ => {
                self.fallback.process(
                    src_indices,
                    src_width,
                    src_height,
                    palette,
                    dst,
                    dst_pitch,
                    dst_width,
                    dst_height,
                    dst_format,
                );
            }
        }
    }
}
