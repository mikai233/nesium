use crate::ppu::palette::Color;
use core::fmt;
use dyn_clone::DynClone;

use super::{ColorFormat, pack_line, pack_pixel};

/// Canonical PPU frame data provided to post-processors.
#[derive(Clone, Copy, Debug)]
pub struct SourceFrame<'a> {
    pub indices: &'a [u8],
    pub emphasis: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> SourceFrame<'a> {
    #[inline]
    pub fn new(indices: &'a [u8], emphasis: &'a [u8], width: usize, height: usize) -> Self {
        let frame = Self {
            indices,
            emphasis,
            width,
            height,
        };
        frame.debug_assert_valid();
        frame
    }

    #[inline]
    pub fn pixel_count(&self) -> usize {
        self.width.saturating_mul(self.height)
    }

    #[inline]
    pub fn debug_assert_valid(&self) {
        debug_assert_eq!(self.indices.len(), self.pixel_count());
        debug_assert_eq!(self.emphasis.len(), self.pixel_count());
    }
}

/// Destination packed framebuffer view provided to post-processors.
#[derive(Debug)]
pub struct TargetFrameMut<'a> {
    pub buffer: &'a mut [u8],
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
    pub format: ColorFormat,
}

impl<'a> TargetFrameMut<'a> {
    #[inline]
    pub fn new(
        buffer: &'a mut [u8],
        pitch: usize,
        width: usize,
        height: usize,
        format: ColorFormat,
    ) -> Self {
        let target = Self {
            buffer,
            pitch,
            width,
            height,
            format,
        };
        target.debug_assert_valid();
        target
    }

    #[inline]
    pub fn row_bytes(&self) -> usize {
        self.width.saturating_mul(self.format.bytes_per_pixel())
    }

    #[inline]
    pub fn debug_assert_valid(&self) {
        debug_assert!(self.pitch >= self.row_bytes());
        if let Some(required) = self.pitch.checked_mul(self.height) {
            debug_assert!(self.buffer.len() >= required);
        }
    }
}

/// Post-processing stage that converts the canonical PPU output (palette indices, 256×240)
/// into a packed pixel buffer (RGBA8888/RGB565/etc.) of an arbitrary runtime-selected size.
///
/// The runtime currently applies post-processing on the emulation thread, so implementors
/// only need to be `Send`. `process` receives `&mut self` and may reuse internal scratch
/// buffers without additional synchronization.
pub trait VideoPostProcessor: fmt::Debug + DynClone + Send {
    /// Convert a canonical source frame into packed pixels in `dst`.
    ///
    /// - `src.indices`/`src.emphasis` length must be `src.width * src.height`.
    /// - `dst.buffer` length must be at least `dst.pitch * dst.height`.
    /// - `dst.pitch` is in bytes and may be larger than `dst.width * bytes_per_pixel`.
    fn process(&mut self, src: SourceFrame<'_>, palette: &[Color; 64], dst: TargetFrameMut<'_>);
}

dyn_clone::clone_trait_object!(VideoPostProcessor);

#[derive(Debug, Default, Clone)]
pub struct NearestPostProcessor;

impl VideoPostProcessor for NearestPostProcessor {
    fn process(&mut self, src: SourceFrame<'_>, palette: &[Color; 64], dst: TargetFrameMut<'_>) {
        src.debug_assert_valid();
        dst.debug_assert_valid();

        let SourceFrame {
            indices: src_indices,
            emphasis: src_emphasis,
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
                    let row_emphasis = &src_emphasis[y * src_width..(y + 1) * src_width];
                    let row_dst = dst_ptr.add(y * dst_pitch);
                    pack_line(row_indices, row_emphasis, row_dst, dst_format, palette);
                }
            } else {
                for y_out in 0..dst_height {
                    let src_y = (y_out * src_height) / dst_height;
                    let dst_row = dst_ptr.add(y_out * dst_pitch);
                    for x_out in 0..dst_width {
                        let src_x = (x_out * src_width) / dst_width;
                        let src_idx = src_y * src_width + src_x;
                        let idx = src_indices[src_idx];
                        let emphasis = src_emphasis[src_idx];
                        let color =
                            super::apply_emphasis(palette[(idx & 0x3F) as usize], idx, emphasis);
                        pack_pixel(color, dst_row.add(x_out * bpp), dst_format);
                    }
                }
            }
        }
    }
}
