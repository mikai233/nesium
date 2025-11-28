/// Double-buffered PPU framebuffer used by the NES core.
///
/// This module provides a simple front/back framebuffer with two modes:
/// - index mode: stores raw palette indices for debugging or PPU inspection
/// - color mode: stores packed RGB/RGBA pixels ready to be consumed by a frontend (SDL, libretro, Flutter, etc.)
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, palette::Palette};

/// Describes how a logical RGB color is packed into the underlying byte buffer.
///
/// The format controls both the number of bytes per pixel and the channel ordering
/// when writing color values into the framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorFormat {
    /// 16-bit packed RGB using 5 bits per channel (0RRRRRGGGGGBBBBB).
    Rgb555,
    /// 16-bit packed RGB using 5/6/5 bits (RRRRRGGGGGGBBBBB).
    Rgb565,
    /// Packed 24-bit RGB, 3 bytes per pixel in R, G, B order.
    Rgb888,
    /// Packed 32-bit RGBA, 4 bytes per pixel in R, G, B, A order.
    Rgba8888,
    /// Packed 32-bit BGRA, 4 bytes per pixel in B, G, R, A order.
    Bgra8888,
    /// Packed 32-bit ARGB, 4 bytes per pixel in A, R, G, B order.
    Argb8888,
}

impl ColorFormat {
    /// Returns the number of bytes used to represent a single pixel in this format.
    #[inline]
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            ColorFormat::Rgb555 | ColorFormat::Rgb565 => 2,
            ColorFormat::Rgb888 => 3,
            ColorFormat::Rgba8888 | ColorFormat::Bgra8888 | ColorFormat::Argb8888 => 4,
        }
    }
}

/// A double-buffered framebuffer for the NES PPU.
///
/// Internally this maintains two planes:
/// - one is actively written to by the PPU
/// - one is exposed for rendering by the frontend
///
/// The `mode` controls whether the planes store palette indices or packed colors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameBuffer {
    active_index: usize,
    planes: [Box<[u8]>; 2],
    mode: BufferMode,
}

/// Selects how framebuffer data is stored.
///
/// `Index` mode stores one byte per pixel as a palette index.
/// `Color` mode stores packed RGB/RGBA pixels according to the chosen `ColorFormat`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum BufferMode {
    /// Palette index buffer (1 byte per pixel).
    #[default]
    Index,
    /// Packed color buffer using a palette and a concrete `ColorFormat`.
    Color {
        palette: Palette,
        format: ColorFormat,
    },
}

impl FrameBuffer {
    /// Creates a new `FrameBuffer` with the given mode and raw buffer length.
    ///
    /// This is a low-level constructor. Prefer the `new_*` convenience constructors
    /// when you want a framebuffer sized to the NES screen.
    pub fn new(mode: BufferMode, len: usize) -> Self {
        if let BufferMode::Color { format, .. } = &mode {
            let expected = SCREEN_WIDTH * SCREEN_HEIGHT * format.bytes_per_pixel();
            debug_assert!(
                len == expected,
                "FrameBuffer len ({len}) does not match expected pixel buffer size ({expected}) for {:?}",
                format
            );
        }

        Self {
            active_index: 0,
            planes: [
                vec![0; len].into_boxed_slice(),
                vec![0; len].into_boxed_slice(),
            ],
            mode,
        }
    }

    /// Creates a new index-mode framebuffer sized to the NES screen.
    ///
    /// Each pixel is stored as a single palette index byte.
    pub fn new_index() -> Self {
        Self::new(BufferMode::Index, SCREEN_WIDTH * SCREEN_HEIGHT)
    }

    /// Creates a new color framebuffer with the given palette and format,
    /// sized to the NES screen.
    pub fn new_color(palette: Palette, format: ColorFormat) -> Self {
        let len = SCREEN_WIDTH * SCREEN_HEIGHT * format.bytes_per_pixel();
        Self::new(BufferMode::Color { palette, format }, len)
    }

    /// Creates a new 16-bit RGB555 framebuffer using the given palette.
    pub fn new_rgb555(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Rgb555)
    }

    /// Creates a new 16-bit RGB565 framebuffer using the given palette.
    pub fn new_rgb565(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Rgb565)
    }

    /// Creates a new 24-bit RGB888 framebuffer using the given palette.
    pub fn new_rgb888(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Rgb888)
    }

    /// Creates a new 32-bit RGBA8888 framebuffer using the given palette.
    pub fn new_rgba8888(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Rgba8888)
    }

    /// Creates a new 32-bit BGRA8888 framebuffer using the given palette.
    pub fn new_bgra8888(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Bgra8888)
    }

    /// Creates a new 32-bit ARGB8888 framebuffer using the given palette.
    pub fn new_argb8888(palette: Palette) -> Self {
        Self::new_color(palette, ColorFormat::Argb8888)
    }

    /// Returns a read-only view of the currently active plane for rendering.
    ///
    /// The returned slice is interpreted according to the current `BufferMode`:
    /// - `Index`: 1 byte per pixel containing a palette index
    /// - `Color`: packed pixels in the selected `ColorFormat`
    pub fn render(&self) -> &[u8] {
        &self.planes[self.active_index]
    }

    /// Returns a mutable view of the currently active plane for PPU writes.
    ///
    /// Typically the PPU will write into this slice and the frontend will read
    /// from it on the next frame after a `swap()`.
    pub fn write(&mut self) -> &mut [u8] {
        &mut self.planes[self.active_index]
    }

    /// Swaps the front and back planes and clears the new write plane to zero.
    ///
    /// After calling this, the previously rendered plane becomes writable and
    /// the previously written plane becomes the render source.
    pub fn swap(&mut self) {
        self.active_index = 1 - self.active_index;
        self.write().fill(0);
    }

    /// Clears both planes to zero.
    ///
    /// This is useful when resetting the PPU or when you need a fully blank frame.
    pub fn clear(&mut self) {
        for plane in &mut self.planes {
            plane.fill(0);
        }
    }

    /// Writes a single pixel at `(x, y)` using a palette index.
    ///
    /// In `Index` mode the index is written directly into the buffer.
    /// In `Color` mode the index is resolved through the `Palette` and then
    /// encoded into the underlying buffer according to the active `ColorFormat`.
    pub fn write_pixel(&mut self, x: usize, y: usize, index: u8) {
        match &mut self.mode {
            BufferMode::Index => {
                let idx = y * SCREEN_WIDTH + x;
                self.write()[idx] = index;
            }
            BufferMode::Color { palette, format } => {
                let buffer = &mut self.planes[self.active_index];
                let color = palette.color(index);
                let bpp = format.bytes_per_pixel();
                let idx = (y * SCREEN_WIDTH + x) * bpp;
                debug_assert!(idx + bpp <= buffer.len());

                match format {
                    ColorFormat::Rgb555 => {
                        // 5 bits per channel: use high bits of 8-bit channels
                        let r5 = (color.r as u16) >> 3;
                        let g5 = (color.g as u16) >> 3;
                        let b5 = (color.b as u16) >> 3;
                        let packed = (r5 << 10) | (g5 << 5) | b5;
                        buffer[idx] = (packed & 0xFF) as u8;
                        buffer[idx + 1] = (packed >> 8) as u8;
                    }
                    ColorFormat::Rgb565 => {
                        let r5 = (color.r as u16) >> 3;
                        let g6 = (color.g as u16) >> 2;
                        let b5 = (color.b as u16) >> 3;
                        let packed = (r5 << 11) | (g6 << 5) | b5;
                        buffer[idx] = (packed & 0xFF) as u8;
                        buffer[idx + 1] = (packed >> 8) as u8;
                    }
                    ColorFormat::Rgb888 => {
                        // 8 bits per channel, 3 bytes: R, G, B
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                    }
                    ColorFormat::Rgba8888 => {
                        // 8 bits per channel, 4 bytes: R, G, B, A
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                        buffer[idx + 3] = 0xFF; // opaque alpha
                    }
                    ColorFormat::Bgra8888 => {
                        // 8 bits per channel, 4 bytes: B, G, R, A
                        buffer[idx] = color.b;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.r;
                        buffer[idx + 3] = 0xFF; // opaque alpha
                    }
                    ColorFormat::Argb8888 => {
                        // 8 bits per channel, 4 bytes: A, R, G, B
                        buffer[idx] = 0xFF; // opaque alpha
                        buffer[idx + 1] = color.r;
                        buffer[idx + 2] = color.g;
                        buffer[idx + 3] = color.b;
                    }
                }
            }
        }
    }

    /// Replaces the palette used by the color buffer.
    ///
    /// Panics if called while the framebuffer is in `Index` mode.
    pub fn set_palette(&mut self, palette: Palette) {
        match &mut self.mode {
            BufferMode::Index => panic!("Cannot set palette on index buffer"),
            BufferMode::Color { palette: p, .. } => {
                *p = palette;
            }
        }
    }

    /// Returns a reference to the palette used by the color buffer.
    ///
    /// Panics if called while the framebuffer is in `Index` mode.
    pub fn get_palette(&self) -> &Palette {
        match &self.mode {
            BufferMode::Index => panic!("Cannot get palette on index buffer"),
            BufferMode::Color { palette, .. } => palette,
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new(BufferMode::Index, SCREEN_WIDTH * SCREEN_HEIGHT)
    }
}
