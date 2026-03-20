use core::ffi::c_void;

use nesium_core::ppu::buffer::{
    ColorFormat, NearestPostProcessor, SourceFrame, TargetFrameMut, VideoPostProcessor,
};
use nesium_core::ppu::palette::Color;

use crate::video::ntsc::{NesNtsc, NesNtscPreset, nes_ntsc_out_width};

const BASE_PALETTE_LEN: usize = 64 * 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NesNtscTuning {
    pub hue: f64,
    pub saturation: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub sharpness: f64,
    pub gamma: f64,
    pub resolution: f64,
    pub artifacts: f64,
    pub fringing: f64,
    pub bleed: f64,
    pub merge_fields: bool,
}

impl Default for NesNtscTuning {
    fn default() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            contrast: 0.0,
            brightness: 0.0,
            sharpness: 0.0,
            gamma: 0.0,
            resolution: 0.0,
            artifacts: 0.0,
            fringing: 0.0,
            bleed: 0.0,
            merge_fields: true,
        }
    }
}

#[derive(Debug)]
pub struct NesNtscPostProcessor {
    preset: NesNtscPreset,
    ntsc: NesNtsc,
    burst_phase: i32,
    tuning: NesNtscTuning,

    base_palette_bytes: Box<[u8; BASE_PALETTE_LEN]>,
    last_palette_bytes: [u8; BASE_PALETTE_LEN],

    input: Vec<u16>,
    tmp_rgb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl NesNtscPostProcessor {
    pub fn new(preset: NesNtscPreset) -> Self {
        Self::new_with_tuning(preset, NesNtscTuning::default())
    }

    pub fn new_with_tuning(preset: NesNtscPreset, tuning: NesNtscTuning) -> Self {
        let mut processor = Self {
            preset,
            ntsc: NesNtsc::new(preset),
            burst_phase: 0,
            tuning,
            base_palette_bytes: Box::new([0; BASE_PALETTE_LEN]),
            last_palette_bytes: [0xFF; BASE_PALETTE_LEN],
            input: Vec::new(),
            tmp_rgb: Vec::new(),
            fallback: NearestPostProcessor,
        };
        processor.reinit_ntsc();
        processor
    }

    pub fn set_tuning(&mut self, tuning: NesNtscTuning) {
        if tuning == self.tuning {
            return;
        }
        self.tuning = tuning;
        self.reinit_ntsc();
    }

    fn reinit_ntsc(&mut self) {
        fn clamp_unit(value: f64) -> f64 {
            value.clamp(-1.0, 1.0)
        }

        let mut setup = *self.preset.setup();

        // Treat tuning values as *deltas* applied on top of the selected preset.
        // This keeps presets meaningful while allowing runtime adjustment.
        setup.hue = clamp_unit(setup.hue + self.tuning.hue);
        setup.saturation = clamp_unit(setup.saturation + self.tuning.saturation);
        setup.contrast = clamp_unit(setup.contrast + self.tuning.contrast);
        setup.brightness = clamp_unit(setup.brightness + self.tuning.brightness);
        setup.sharpness = clamp_unit(setup.sharpness + self.tuning.sharpness);
        setup.gamma = clamp_unit(setup.gamma + self.tuning.gamma);
        setup.resolution = clamp_unit(setup.resolution + self.tuning.resolution);
        setup.artifacts = clamp_unit(setup.artifacts + self.tuning.artifacts);
        setup.fringing = clamp_unit(setup.fringing + self.tuning.fringing);
        setup.bleed = clamp_unit(setup.bleed + self.tuning.bleed);
        setup.merge_fields = self.tuning.merge_fields as i32;
        setup.palette = core::ptr::null();
        setup.base_palette = self.base_palette_bytes.as_ptr();
        self.ntsc.set_setup(setup);
    }

    fn update_base_palette_if_needed(&mut self, palette: &[Color; 64]) {
        for (i, c) in palette.iter().enumerate() {
            let off = i * 3;
            self.base_palette_bytes[off] = c.r;
            self.base_palette_bytes[off + 1] = c.g;
            self.base_palette_bytes[off + 2] = c.b;
        }

        if *self.base_palette_bytes != self.last_palette_bytes {
            self.last_palette_bytes = *self.base_palette_bytes;
            self.reinit_ntsc();
        }
    }
}

impl Clone for NesNtscPostProcessor {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            preset: self.preset,
            ntsc: NesNtsc::new(self.preset),
            burst_phase: self.burst_phase,
            tuning: self.tuning,
            base_palette_bytes: Box::new(*self.base_palette_bytes),
            last_palette_bytes: self.last_palette_bytes,
            input: Vec::new(),
            tmp_rgb: Vec::new(),
            fallback: self.fallback.clone(),
        };
        cloned.reinit_ntsc();
        cloned
    }
}

impl VideoPostProcessor for NesNtscPostProcessor {
    fn process(&mut self, src: SourceFrame<'_>, palette: &[Color; 64], dst: TargetFrameMut<'_>) {
        let SourceFrame {
            indices: src_indices,
            emphasis: _src_emphasis,
            width: src_width,
            height: src_height,
        } = src;
        let TargetFrameMut {
            buffer: dst,
            pitch: dst_pitch,
            width: dst_width,
            height: dst_height,
            format: dst_format,
        } = dst;

        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return;
        }

        let out_w = nes_ntsc_out_width(src_width);
        let out_h = src_height.saturating_mul(2);
        if dst_width != out_w || dst_height != out_h {
            self.fallback.process(
                src,
                palette,
                TargetFrameMut {
                    buffer: dst,
                    pitch: dst_pitch,
                    width: dst_width,
                    height: dst_height,
                    format: dst_format,
                },
            );
            return;
        }

        let bpp = dst_format.bytes_per_pixel();
        if bpp != 4 {
            self.fallback.process(
                src,
                palette,
                TargetFrameMut {
                    buffer: dst,
                    pitch: dst_pitch,
                    width: dst_width,
                    height: dst_height,
                    format: dst_format,
                },
            );
            return;
        }

        let row_bytes = out_w.saturating_mul(4);
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

        self.update_base_palette_if_needed(palette);

        self.input.resize(expected_in, 0);
        for (i, &idx) in src_indices.iter().enumerate() {
            self.input[i] = idx as u16;
        }

        let tmp_len = match out_w.checked_mul(src_height) {
            Some(v) => v,
            None => return,
        };
        self.tmp_rgb.resize(tmp_len, 0);

        unsafe {
            self.ntsc.blit(
                self.input.as_ptr(),
                src_width,
                self.burst_phase,
                src_width,
                src_height,
                self.tmp_rgb.as_mut_ptr() as *mut c_void,
                row_bytes,
            );
        }
        self.burst_phase = (self.burst_phase + 1) % 3;

        match dst_format {
            ColorFormat::Rgba8888 => {
                for y_in in 0..src_height {
                    let src_row = &self.tmp_rgb[y_in * out_w..(y_in + 1) * out_w];
                    let y0 = y_in * 2;
                    let y1 = y0 + 1;
                    for &y_out in &[y0, y1] {
                        let row_dst = &mut dst[y_out * dst_pitch..y_out * dst_pitch + row_bytes];
                        for (x, &rgb) in src_row.iter().enumerate() {
                            let r = ((rgb >> 16) & 0xFF) as u8;
                            let g = ((rgb >> 8) & 0xFF) as u8;
                            let b = (rgb & 0xFF) as u8;
                            let off = x * 4;
                            row_dst[off] = r;
                            row_dst[off + 1] = g;
                            row_dst[off + 2] = b;
                            row_dst[off + 3] = 0xFF;
                        }
                    }
                }
            }
            ColorFormat::Bgra8888 => {
                for y_in in 0..src_height {
                    let src_row = &self.tmp_rgb[y_in * out_w..(y_in + 1) * out_w];
                    let y0 = y_in * 2;
                    let y1 = y0 + 1;
                    for &y_out in &[y0, y1] {
                        let row_dst = &mut dst[y_out * dst_pitch..y_out * dst_pitch + row_bytes];
                        for (x, &rgb) in src_row.iter().enumerate() {
                            let r = ((rgb >> 16) & 0xFF) as u8;
                            let g = ((rgb >> 8) & 0xFF) as u8;
                            let b = (rgb & 0xFF) as u8;
                            let off = x * 4;
                            row_dst[off] = b;
                            row_dst[off + 1] = g;
                            row_dst[off + 2] = r;
                            row_dst[off + 3] = 0xFF;
                        }
                    }
                }
            }
            ColorFormat::Argb8888 => {
                for y_in in 0..src_height {
                    let src_row = &self.tmp_rgb[y_in * out_w..(y_in + 1) * out_w];
                    let y0 = y_in * 2;
                    let y1 = y0 + 1;
                    for &y_out in &[y0, y1] {
                        let row_dst = &mut dst[y_out * dst_pitch..y_out * dst_pitch + row_bytes];
                        for (x, &rgb) in src_row.iter().enumerate() {
                            let r = ((rgb >> 16) & 0xFF) as u8;
                            let g = ((rgb >> 8) & 0xFF) as u8;
                            let b = (rgb & 0xFF) as u8;
                            let off = x * 4;
                            row_dst[off] = 0xFF;
                            row_dst[off + 1] = r;
                            row_dst[off + 2] = g;
                            row_dst[off + 3] = b;
                        }
                    }
                }
            }
            _ => {
                self.fallback.process(
                    src,
                    palette,
                    TargetFrameMut {
                        buffer: dst,
                        pitch: dst_pitch,
                        width: dst_width,
                        height: dst_height,
                        format: dst_format,
                    },
                );
            }
        }
    }
}
