use nesium_core::ppu::buffer::{ColorFormat, NearestPostProcessor, VideoPostProcessor};
use nesium_core::ppu::palette::Color;

use crate::video::ntsc_bisqwit::ntsc_bisqwit_apply_argb8888;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NtscBisqwitOptions {
    pub brightness: f64,
    pub contrast: f64,
    pub hue: f64,
    pub saturation: f64,
    pub y_filter_length: f64,
    pub i_filter_length: f64,
    pub q_filter_length: f64,
}

impl Default for NtscBisqwitOptions {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 0.0,
            hue: 0.0,
            saturation: 0.0,
            y_filter_length: 0.0,
            i_filter_length: 0.5,
            q_filter_length: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NtscBisqwitPostProcessor {
    scale: u8,
    options: NtscBisqwitOptions,
    ppu: Vec<u16>,
    output_argb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl NtscBisqwitPostProcessor {
    pub fn new(scale: u8, options: NtscBisqwitOptions) -> Self {
        Self {
            scale,
            options,
            ppu: Vec::new(),
            output_argb: Vec::new(),
            fallback: NearestPostProcessor::default(),
        }
    }
}

impl VideoPostProcessor for NtscBisqwitPostProcessor {
    fn process(
        &mut self,
        src_indices: &[u8],
        src_width: usize,
        src_height: usize,
        _palette: &[Color; 64],
        dst: &mut [u8],
        dst_pitch: usize,
        dst_width: usize,
        dst_height: usize,
        dst_format: ColorFormat,
    ) {
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return;
        }

        let scale = self.scale.clamp(2, 8);
        if !(scale == 2 || scale == 4 || scale == 8) {
            self.fallback.process(
                src_indices,
                src_width,
                src_height,
                _palette,
                dst,
                dst_pitch,
                dst_width,
                dst_height,
                dst_format,
            );
            return;
        }

        let expected_w = src_width.saturating_mul(scale as usize);
        let expected_h = src_height.saturating_mul(scale as usize);
        if dst_width != expected_w || dst_height != expected_h {
            self.fallback.process(
                src_indices,
                src_width,
                src_height,
                _palette,
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
                _palette,
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

        self.ppu.resize(expected_in, 0);
        for (i, &idx) in src_indices.iter().enumerate() {
            self.ppu[i] = (idx & 0x3F) as u16;
        }

        self.output_argb.resize(expected_out, 0);
        ntsc_bisqwit_apply_argb8888(
            self.ppu.as_slice(),
            src_width,
            src_height,
            self.output_argb.as_mut_slice(),
            scale as usize,
            self.options.brightness,
            self.options.contrast,
            self.options.hue,
            self.options.saturation,
            self.options.y_filter_length,
            self.options.i_filter_length,
            self.options.q_filter_length,
            0,
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
                    _palette,
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
