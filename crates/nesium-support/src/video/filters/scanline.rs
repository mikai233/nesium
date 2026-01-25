use nesium_core::ppu::buffer::{ColorFormat, NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::palette::Color;

use crate::video::scanline::scanline_apply_argb8888;

#[derive(Debug, Clone)]
pub struct ScanlinePostProcessor {
    scale: u8,
    /// Scanline intensity in `0.0..=1.0` (0 = off, 1 = strongest).
    intensity: f64,
    output_argb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl ScanlinePostProcessor {
    pub fn new(scale: u8, intensity: f64) -> Self {
        Self {
            scale,
            intensity: intensity.clamp(0.0, 1.0),
            output_argb: Vec::new(),
            fallback: NearestPostProcessor,
        }
    }
}

impl VideoPostProcessor for ScanlinePostProcessor {
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

        if self.intensity <= 0.0 {
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

        let scale = self.scale.max(2) as usize;
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
        self.output_argb.resize(expected_out, 0);

        // Integer nearest-neighbor scaling into ARGB8888 scratch.
        for y_out in 0..dst_height {
            let src_y = (y_out * src_height) / dst_height;
            let row = &mut self.output_argb[y_out * dst_width..(y_out + 1) * dst_width];
            for x_out in 0..dst_width {
                let src_x = (x_out * src_width) / dst_width;
                let idx = src_indices[src_y * src_width + src_x];
                let c = palette[(idx & 0x3F) as usize];
                row[x_out] =
                    0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
            }
        }

        // Mesen2: intensity = (1.0 - scanlineIntensity) * 255.
        let brightness = ((1.0 - self.intensity) * 255.0).round().clamp(0.0, 255.0) as u8;
        scanline_apply_argb8888(
            dst_width,
            dst_height,
            self.output_argb.as_mut_slice(),
            brightness,
            self.scale,
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
