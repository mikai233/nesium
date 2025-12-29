use nesium_core::ppu::{buffer::ExternalFrameHandle, SCREEN_HEIGHT, SCREEN_WIDTH};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
};
use std::sync::Arc;

/// Custom widget to render the NES frame using half-blocks
pub struct NesFrameWidget {
    frame_handle: Arc<ExternalFrameHandle>,
}

impl NesFrameWidget {
    pub fn new(frame_handle: Arc<ExternalFrameHandle>) -> Self {
        Self { frame_handle }
    }
}

impl Widget for NesFrameWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame_ptr = self.frame_handle.front_slice();
        // Assuming RGBA8888 (4 bytes per pixel)
        // NES: 256x240
        const NES_W: f64 = SCREEN_WIDTH as f64;
        const NES_H: f64 = SCREEN_HEIGHT as f64;

        if frame_ptr.len() < SCREEN_WIDTH * SCREEN_HEIGHT * 4 {
            return;
        }

        let term_w = area.width as f64;
        let term_h = area.height as f64;

        // Target canvas size (virtual pixels) - 2 vertical subpixels per char
        let canvas_h = term_h * 2.0;

        if term_w == 0.0 || term_h == 0.0 {
            return;
        }

        // Calculate scale to fit preserving aspect ratio
        let scale_x = term_w / NES_W;
        let scale_y = canvas_h / NES_H;
        let scale = scale_x.min(scale_y);

        let draw_w = NES_W * scale;
        let draw_h = NES_H * scale;

        let off_x = (term_w - draw_w) / 2.0;
        let off_y = (canvas_h - draw_h) / 2.0;

        for y in 0..area.height {
            for x in 0..area.width {
                let vx = x as f64;
                let vy_top = (y * 2) as f64;
                let vy_bot = (y * 2 + 1) as f64;

                let color_top = sample_pixel(frame_ptr, vx, vy_top, off_x, off_y, scale);
                let color_bot = sample_pixel(frame_ptr, vx, vy_bot, off_x, off_y, scale);

                let cell = buf.get_mut(area.left() + x, area.top() + y);
                cell.set_char('▀').set_fg(color_top).set_bg(color_bot);
            }
        }
    }
}

fn sample_pixel(
    buffer: &[u8],
    vx: f64,
    vy: f64,
    off_x: f64,
    off_y: f64,
    scale: f64,
) -> Color {
    // Map virtual coordinate to NES coordinate
    let nx = (vx - off_x) / scale;
    let ny = (vy - off_y) / scale;

    if nx >= 0.0 && nx < SCREEN_WIDTH as f64 && ny >= 0.0 && ny < SCREEN_HEIGHT as f64 {
        let ix = nx as usize;
        let iy = ny as usize;
        get_pixel_color(buffer, ix, iy)
    } else {
        Color::Black
    }
}

#[inline]
fn get_pixel_color(buffer: &[u8], x: usize, y: usize) -> Color {
    let idx = (y * SCREEN_WIDTH + x) * 4;
    // Safety check mostly for bounds, though logic above should prevent it
    if idx + 3 >= buffer.len() {
        return Color::Magenta;
    }

    let r = buffer[idx];
    let g = buffer[idx + 1];
    let b = buffer[idx + 2];
    // Ignore alpha

    Color::Rgb(r, g, b)
}