use nesium_core::ppu::buffer::{ColorFormat, NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::palette::Color;

use crate::video::lcd_grid::lcd_grid_2x_argb8888;

#[derive(Debug, Clone)]
pub struct LcdGridPostProcessor {
    strength: f64,
    input_argb: Vec<u32>,
    output_argb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl Default for LcdGridPostProcessor {
    fn default() -> Self {
        Self {
            strength: 1.0,
            input_argb: Vec::new(),
            output_argb: Vec::new(),
            fallback: NearestPostProcessor,
        }
    }
}

impl LcdGridPostProcessor {
    /// Creates a 2x LCD grid effect similar to Mesen2's "LCD Grid".
    ///
    /// `strength` is clamped to `0.0..=1.0`:
    /// - `0.0`: no visible grid (all subpixels at 100% brightness).
    /// - `1.0`: Mesen2 default (TL=100%, others=85%).
    pub fn new(strength: f64) -> Self {
        Self {
            strength: strength.clamp(0.0, 1.0),
            ..Self::default()
        }
    }
}

impl VideoPostProcessor for LcdGridPostProcessor {
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

        let expected_w = src_width.saturating_mul(2);
        let expected_h = src_height.saturating_mul(2);
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

        // Mesen2 default:
        // - TL = 100%, TR = 85%, BL = 85%, BR = 85%
        // We expose a single "strength" knob that interpolates towards the default.
        let tl = 255u8;
        let others = (1.0 - 0.15 * self.strength).clamp(0.0, 1.0);
        let o = (others * 255.0).round() as u8;
        lcd_grid_2x_argb8888(
            src_width,
            src_height,
            self.input_argb.as_slice(),
            src_width,
            self.output_argb.as_mut_slice(),
            dst_width,
            tl,
            o,
            o,
            o,
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
