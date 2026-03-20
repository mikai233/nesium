use nesium_core::ppu::buffer::{
    ColorFormat, NearestPostProcessor, SourceFrame, TargetFrameMut, VideoPostProcessor,
};
use nesium_core::ppu::palette::Color;

#[cfg(any(not(feature = "sai-cpp"), target_arch = "wasm32"))]
use crate::video::sai::{
    scale_2xsai_xrgb8888, scale_super_2xsai_xrgb8888, scale_supereagle_xrgb8888,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaiVariant {
    Sai2x,
    Super2xSai,
    SuperEagle,
}

#[derive(Debug, Clone)]
pub struct SaiPostProcessor {
    variant: SaiVariant,
    input_xrgb: Vec<u32>,
    output_xrgb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl SaiPostProcessor {
    pub fn new(variant: SaiVariant) -> Self {
        Self {
            variant,
            input_xrgb: Vec::new(),
            output_xrgb: Vec::new(),
            fallback: NearestPostProcessor,
        }
    }
}

impl VideoPostProcessor for SaiPostProcessor {
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

        let expected_w = src_width.saturating_mul(2);
        let expected_h = src_height.saturating_mul(2);
        if dst_width != expected_w || dst_height != expected_h {
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

        self.input_xrgb.resize(expected_in, 0);
        self.output_xrgb.resize(expected_out, 0);

        for (i, &idx) in src_indices.iter().enumerate() {
            let c = palette[(idx & 0x3F) as usize];
            self.input_xrgb[i] =
                0xFF00_0000 | ((c.r as u32) << 16) | ((c.g as u32) << 8) | (c.b as u32);
        }

        match self.variant {
            SaiVariant::Sai2x => {
                #[cfg(all(feature = "sai-cpp", not(target_arch = "wasm32")))]
                {
                    crate::video::sai::scale_2xsai_xrgb8888_cpp(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
                #[cfg(any(not(feature = "sai-cpp"), target_arch = "wasm32"))]
                {
                    scale_2xsai_xrgb8888(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
            }
            SaiVariant::Super2xSai => {
                #[cfg(all(feature = "sai-cpp", not(target_arch = "wasm32")))]
                {
                    crate::video::sai::scale_super_2xsai_xrgb8888_cpp(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
                #[cfg(any(not(feature = "sai-cpp"), target_arch = "wasm32"))]
                {
                    scale_super_2xsai_xrgb8888(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
            }
            SaiVariant::SuperEagle => {
                #[cfg(all(feature = "sai-cpp", not(target_arch = "wasm32")))]
                {
                    crate::video::sai::scale_supereagle_xrgb8888_cpp(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
                #[cfg(any(not(feature = "sai-cpp"), target_arch = "wasm32"))]
                {
                    scale_supereagle_xrgb8888(
                        src_width,
                        src_height,
                        self.input_xrgb.as_slice(),
                        src_width,
                        self.output_xrgb.as_mut_slice(),
                        dst_width,
                    );
                }
            }
        }

        match dst_format {
            ColorFormat::Rgba8888 => {
                for y in 0..dst_height {
                    let row_src = &self.output_xrgb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &xrgb) in row_src.iter().enumerate() {
                        let r = ((xrgb >> 16) & 0xFF) as u8;
                        let g = ((xrgb >> 8) & 0xFF) as u8;
                        let b = (xrgb & 0xFF) as u8;
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
                    let row_src = &self.output_xrgb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &xrgb) in row_src.iter().enumerate() {
                        let r = ((xrgb >> 16) & 0xFF) as u8;
                        let g = ((xrgb >> 8) & 0xFF) as u8;
                        let b = (xrgb & 0xFF) as u8;
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
                    let row_src = &self.output_xrgb[y * dst_width..(y + 1) * dst_width];
                    let row_dst = &mut dst[y * dst_pitch..y * dst_pitch + row_bytes];
                    for (x, &xrgb) in row_src.iter().enumerate() {
                        let r = ((xrgb >> 16) & 0xFF) as u8;
                        let g = ((xrgb >> 8) & 0xFF) as u8;
                        let b = (xrgb & 0xFF) as u8;
                        let off = x * 4;
                        row_dst[off] = 0xFF;
                        row_dst[off + 1] = r;
                        row_dst[off + 2] = g;
                        row_dst[off + 3] = b;
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
