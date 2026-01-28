use crate::ppu::palette::Color;
use core::fmt;
use dyn_clone::DynClone;

use super::{ColorFormat, pack_line, pack_pixel};

/// Post-processing stage that converts the canonical PPU output (palette indices, 256×240)
/// into a packed pixel buffer (RGBA8888/RGB565/etc.) of an arbitrary runtime-selected size.
///
/// The runtime currently applies post-processing on the emulation thread, so implementors
/// only need to be `Send`. `process` receives `&mut self` and may reuse internal scratch
/// buffers without additional synchronization.
pub trait VideoPostProcessor: fmt::Debug + DynClone + Send {
    /// Convert `src_indices` (palette indices) into packed pixels in `dst`.
    ///
    /// - `src_indices` length must be `src_width * src_height`.
    /// - `dst` length must be at least `dst_pitch * dst_height`.
    /// - `dst_pitch` is in bytes and may be larger than `dst_width * bytes_per_pixel`.
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
    );
}

dyn_clone::clone_trait_object!(VideoPostProcessor);

#[derive(Debug, Default, Clone)]
pub struct NearestPostProcessor;

impl VideoPostProcessor for NearestPostProcessor {
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
        debug_assert_eq!(src_indices.len(), src_width * src_height);
        if dst_width == 0 || dst_height == 0 {
            return;
        }
        let bpp = dst_format.bytes_per_pixel();
        let row_bytes = dst_width * bpp;
        if dst_pitch < row_bytes || dst.len() < dst_pitch * dst_height {
            return;
        }

        let dst_ptr = dst.as_mut_ptr();
        unsafe {
            if dst_width == src_width && dst_height == src_height {
                for y in 0..src_height {
                    let row_indices = &src_indices[y * src_width..(y + 1) * src_width];
                    let row_dst = dst_ptr.add(y * dst_pitch);
                    pack_line(row_indices, row_dst, dst_format, palette);
                }
            } else {
                for y_out in 0..dst_height {
                    let src_y = (y_out * src_height) / dst_height;
                    let dst_row = dst_ptr.add(y_out * dst_pitch);
                    for x_out in 0..dst_width {
                        let src_x = (x_out * src_width) / dst_width;
                        let idx = src_indices[src_y * src_width + src_x];
                        let color = palette[(idx & 0x3F) as usize];
                        pack_pixel(color, dst_row.add(x_out * bpp), dst_format);
                    }
                }
            }
        }
    }
}
