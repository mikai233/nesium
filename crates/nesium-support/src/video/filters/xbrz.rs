use nesium_core::ppu::buffer::{
    ColorFormat, NearestPostProcessor, SourceFrame, TargetFrameMut, VideoPostProcessor,
};
use nesium_core::ppu::palette::Color;
use xbrz::scale_rgba;

#[derive(Debug, Clone)]
pub struct XbrzPostProcessor {
    scale: u8,
    input_argb: Vec<u32>,
    fallback: NearestPostProcessor,
}

impl XbrzPostProcessor {
    pub fn new(scale: u8) -> Self {
        Self {
            scale,
            input_argb: Vec::new(),
            fallback: NearestPostProcessor,
        }
    }
}

impl VideoPostProcessor for XbrzPostProcessor {
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

        let scale = self.scale.clamp(2, 6) as usize;
        let expected_w = src_width.saturating_mul(scale);
        let expected_h = src_height.saturating_mul(scale);
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

        self.input_argb.resize(expected_in, 0);

        for (i, &idx) in src_indices.iter().enumerate() {
            let c = palette[(idx & 0x3F) as usize];
            // xbrz-rs takes RGBA bytes (R, G, B, A).
            // We pack into u32.
            // On Little Endian: 0xAABBGGRR -> [RR, GG, BB, AA]
            self.input_argb[i] =
                0xFF00_0000 | ((c.b as u32) << 16) | ((c.g as u32) << 8) | (c.r as u32);
        }

        // Cast u32 slice to u8 slice
        // SAFETY: u32 to u8 cast is safe for read.
        let src_bytes = unsafe {
            std::slice::from_raw_parts(
                self.input_argb.as_ptr() as *const u8,
                self.input_argb.len() * 4,
            )
        };

        let output_rgba = scale_rgba(src_bytes, src_width, src_height, scale);

        // Check output size
        if output_rgba.len() != dst_width * dst_height * 4 {
            // Should not happen if xbrz works as expected
            return;
        }

        match dst_format {
            ColorFormat::Rgba8888 => {
                for y in 0..dst_height {
                    let src_start = y * dst_width * 4;
                    let src_end = src_start + dst_width * 4;
                    let row_src = &output_rgba[src_start..src_end];

                    let dst_start = y * dst_pitch;
                    let dst_end = dst_start + row_bytes;
                    let row_dst = &mut dst[dst_start..dst_end];

                    row_dst.copy_from_slice(row_src);
                }
            }
            ColorFormat::Bgra8888 => {
                for y in 0..dst_height {
                    let src_start = y * dst_width * 4;
                    let src_end = src_start + dst_width * 4;
                    let row_src = &output_rgba[src_start..src_end];

                    let dst_start = y * dst_pitch;
                    let dst_end = dst_start + row_bytes;
                    let row_dst = &mut dst[dst_start..dst_end];

                    for x in 0..dst_width {
                        let off = x * 4;
                        // src: R G B A
                        // dst: B G R A
                        row_dst[off] = row_src[off + 2]; // B <- B
                        row_dst[off + 1] = row_src[off + 1]; // G <- G
                        row_dst[off + 2] = row_src[off]; // R <- R
                        row_dst[off + 3] = row_src[off + 3]; // A <- A
                    }
                }
            }
            ColorFormat::Argb8888 => {
                for y in 0..dst_height {
                    let src_start = y * dst_width * 4;
                    let src_end = src_start + dst_width * 4;
                    let row_src = &output_rgba[src_start..src_end];

                    let dst_start = y * dst_pitch;
                    let dst_end = dst_start + row_bytes;
                    let row_dst = &mut dst[dst_start..dst_end];

                    for x in 0..dst_width {
                        let off = x * 4;
                        // src: R G B A
                        // dst: A R G B
                        row_dst[off] = row_src[off + 3]; // A <- A
                        row_dst[off + 1] = row_src[off]; // R <- R
                        row_dst[off + 2] = row_src[off + 1]; // G <- G
                        row_dst[off + 3] = row_src[off + 2]; // B <- B
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
