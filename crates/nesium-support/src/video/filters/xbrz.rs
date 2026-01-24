use nesium_core::ppu::buffer::{ColorFormat, NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::palette::Color;

use crate::video::xbrz::xbrz_scale_argb8888;

#[derive(Debug, Clone)]
pub struct XbrzPostProcessor {
    scale: u8,
    input_argb: Vec<u32>,
    output_argb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl XbrzPostProcessor {
    pub fn new(scale: u8) -> Self {
        Self {
            scale,
            input_argb: Vec::new(),
            output_argb: Vec::new(),
            fallback: NearestPostProcessor::default(),
        }
    }
}

impl VideoPostProcessor for XbrzPostProcessor {
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
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return;
        }

        let scale = self.scale.clamp(2, 6) as usize;
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
        if bpp != 4 {
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

        let row_bytes = dst_width.saturating_mul(4);
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

        let expected_in = match src_width.checked_mul(src_height) {
            Some(v) => v,
            None => return,
        };
        if src_indices.len() != expected_in {
            return;
        }
        let expected_out = match dst_width.checked_mul(dst_height) {
            Some(v) => v,
            None => return,
        };

        self.input_argb.resize(expected_in, 0);
        self.output_argb.resize(expected_out, 0);

        for (i, &idx) in src_indices.iter().enumerate() {
            let c = palette[(idx & 0x3F) as usize];
            self.input_argb[i] =
                0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
        }

        xbrz_scale_argb8888(
            scale,
            self.input_argb.as_slice(),
            src_width,
            src_height,
            self.output_argb.as_mut_slice(),
        );

        match dst_format {
            ColorFormat::Rgba8888 => {
                for y in 0..dst_height {
                    let row_src = &self.output_argb[y * dst_width..(y + 1) * dst_width];
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
                    let row_src = &self.output_argb[y * dst_width..(y + 1) * dst_width];
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
                    let row_src = &self.output_argb[y * dst_width..(y + 1) * dst_width];
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
