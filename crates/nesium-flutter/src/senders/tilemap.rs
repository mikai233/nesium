use std::any::Any;
use std::sync::atomic::{AtomicU8, Ordering};

use flutter_rust_bridge::frb;
use nesium_core::cartridge::header::Mirroring;
use nesium_runtime::{Event, RuntimeEventSender};

use crate::api::events::TilemapSnapshot;
use crate::frb_generated::StreamSink;

/// Auxiliary texture ID for Tilemap Viewer.
pub const TILEMAP_TEXTURE_ID: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TilemapRenderMode {
    Default = 0,
    Grayscale = 1,
    AttributeView = 2,
}

static TILEMAP_RENDER_MODE: AtomicU8 = AtomicU8::new(TilemapRenderMode::Default as u8);

fn tilemap_render_mode() -> TilemapRenderMode {
    match TILEMAP_RENDER_MODE.load(Ordering::Relaxed) {
        1 => TilemapRenderMode::Grayscale,
        2 => TilemapRenderMode::AttributeView,
        _ => TilemapRenderMode::Default,
    }
}

/// Sets the render mode for the tilemap auxiliary texture.
///
/// - `0`: Default
/// - `1`: Grayscale
/// - `2`: Attribute view
pub fn set_tilemap_display_mode(mode: u8) {
    let mode = match mode {
        1 => TilemapRenderMode::Grayscale,
        2 => TilemapRenderMode::AttributeView,
        _ => TilemapRenderMode::Default,
    };
    TILEMAP_RENDER_MODE.store(mode as u8, Ordering::Relaxed);
}

/// Sender that renders TilemapState directly to auxiliary texture (no stream).
pub struct TilemapTextureSender {}

impl RuntimeEventSender for TilemapTextureSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::TilemapState>() {
            // Queue rendering to worker thread instead of blocking NES thread
            let _ = crate::event_worker::event_worker()
                .send(crate::event_worker::EventTask::Tilemap { state, sink: None });
        }
        true
    }
}

/// Sender that updates the tilemap auxiliary texture AND streams TilemapSnapshot to Flutter.
pub struct TilemapTextureAndStateSender {
    sink: StreamSink<TilemapSnapshot>,
}

impl TilemapTextureAndStateSender {
    pub fn new(sink: StreamSink<TilemapSnapshot>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for TilemapTextureAndStateSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::TilemapState>() {
            // Queue all work to worker thread - rendering, palette conversion, and streaming
            let _ =
                crate::event_worker::event_worker().send(crate::event_worker::EventTask::Tilemap {
                    state,
                    sink: Some(self.sink.clone()),
                });
            return true;
        }
        true
    }
}

/// Renders tilemap state to RGBA and updates the auxiliary texture.
pub fn render_tilemap_to_aux(state: &nesium_runtime::TilemapState) {
    // Output: 512x480 (2x2 nametables: 256x240 each)
    const WIDTH: usize = 512;
    const HEIGHT: usize = 480;

    let mut rgba = vec![0u8; WIDTH * HEIGHT * 4];
    let mode = tilemap_render_mode();

    // Get CIRAM offsets for each logical nametable based on mirroring
    let nt0 = mirror_nametable_to_ciram_offset(0, state.mirroring);
    let nt1 = mirror_nametable_to_ciram_offset(1, state.mirroring);
    let nt2 = mirror_nametable_to_ciram_offset(2, state.mirroring);
    let nt3 = mirror_nametable_to_ciram_offset(3, state.mirroring);

    // Render 4 nametables with mirroring applied
    match mode {
        TilemapRenderMode::AttributeView => {
            render_nametable_attribute_view(state, nt0, 0, 0, &mut rgba, WIDTH);
            render_nametable_attribute_view(state, nt1, 256, 0, &mut rgba, WIDTH);
            render_nametable_attribute_view(state, nt2, 0, 240, &mut rgba, WIDTH);
            render_nametable_attribute_view(state, nt3, 256, 240, &mut rgba, WIDTH);
        }
        TilemapRenderMode::Default | TilemapRenderMode::Grayscale => {
            render_nametable(state, nt0, 0, 0, &mut rgba, WIDTH); // Top-Left (NT0 / $2000)
            render_nametable(state, nt1, 256, 0, &mut rgba, WIDTH); // Top-Right (NT1 / $2400)
            render_nametable(state, nt2, 0, 240, &mut rgba, WIDTH); // Bottom-Left (NT2 / $2800)
            render_nametable(state, nt3, 256, 240, &mut rgba, WIDTH); // Bottom-Right (NT3 / $2C00)
        }
    }

    if mode == TilemapRenderMode::Grayscale {
        apply_grayscale_in_place(&mut rgba);
    }

    crate::aux_texture::aux_update(TILEMAP_TEXTURE_ID, &rgba);
}

/// Maps a logical nametable index (0-3) to its physical CIRAM offset (0 or 0x400) based on mirroring mode.
fn mirror_nametable_to_ciram_offset(nt_index: usize, mirroring: Mirroring) -> usize {
    let physical_nt = match mirroring {
        Mirroring::Horizontal => {
            // Horizontal: NT 0,1 -> CIRAM 0x000; NT 2,3 -> CIRAM 0x400
            match nt_index {
                0 | 1 => 0,
                _ => 1,
            }
        }
        Mirroring::Vertical => {
            // Vertical: NT 0,2 -> CIRAM 0x000; NT 1,3 -> CIRAM 0x400
            match nt_index {
                0 | 2 => 0,
                _ => 1,
            }
        }
        Mirroring::FourScreen => {
            // FourScreen: all 4 are independent (needs 4 KiB, but CIRAM only has 2 KiB)
            // Fall back to identity mapping for first 2 pages
            nt_index.min(1)
        }
        Mirroring::SingleScreenLower => {
            // SingleScreenLower: all map to CIRAM 0x000
            0
        }
        Mirroring::SingleScreenUpper => {
            // SingleScreenUpper: all map to CIRAM 0x400
            1
        }
        Mirroring::MapperControlled => {
            // Mapper controlled: identity fallback
            nt_index.min(1)
        }
    };

    physical_nt * 0x400
}

/// Renders a single 256x240 nametable into the RGBA buffer at (offset_x, offset_y).
fn render_nametable(
    state: &nesium_runtime::TilemapState,
    ciram_offset: usize,
    offset_x: usize,
    offset_y: usize,
    rgba: &mut [u8],
    pitch: usize,
) {
    let ciram = &state.ciram;
    let chr = &state.chr;
    let palette = &state.palette;

    // Nametable is 32x30 tiles, each 8x8 pixels
    for tile_y in 0..30 {
        for tile_x in 0..32 {
            // CIRAM offset (0-0x3FF) within this nametable
            let nt_local_addr = tile_y * 32 + tile_x;
            let ciram_addr = ciram_offset + nt_local_addr;
            let tile_index = if ciram_addr < ciram.len() {
                ciram[ciram_addr] as usize
            } else {
                0
            };

            // Attribute table is at offset 0x3C0 within each nametable
            let attr_local_addr = 0x3C0 + (tile_y / 4) * 8 + (tile_x / 4);
            let attr_ciram_addr = ciram_offset + attr_local_addr;
            let attr_byte = if attr_ciram_addr < ciram.len() {
                ciram[attr_ciram_addr]
            } else {
                0
            };

            // Determine which quadrant of the attribute byte applies
            let shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
            let palette_index = ((attr_byte >> shift) & 0x03) as usize;

            // Draw 8x8 tile
            for py in 0..8 {
                // CHR data: each tile is 16 bytes (plane0 + plane1)
                // Use bg_pattern_base to select correct pattern table ($0000 or $1000)
                let chr_offset = state.bg_pattern_base as usize + tile_index * 16 + py;
                let plane0 = if chr_offset < chr.len() {
                    chr[chr_offset]
                } else {
                    0
                };
                let plane1 = if chr_offset + 8 < chr.len() {
                    chr[chr_offset + 8]
                } else {
                    0
                };

                for px in 0..8 {
                    let bit = 7 - px;
                    let color_low = (plane0 >> bit) & 1;
                    let color_high = (plane1 >> bit) & 1;
                    let color_index = (color_high << 1) | color_low;

                    // Palette lookup
                    let pal_offset = palette_index * 4 + color_index as usize;
                    let nes_color = if color_index == 0 {
                        // Background color (universal)
                        palette[0] as usize
                    } else {
                        if pal_offset < palette.len() {
                            palette[pal_offset] as usize
                        } else {
                            0
                        }
                    };
                    let nes_color = nes_color & 0x3F;

                    let screen_x = offset_x + tile_x * 8 + px;
                    let screen_y = offset_y + tile_y * 8 + py;
                    let idx = (screen_y * pitch + screen_x) * 4;

                    if idx + 3 < rgba.len() {
                        // Palette is already in platform-specific format (set in runner.rs)
                        let pixel = state.bgra_palette[nes_color];
                        rgba[idx] = pixel[0];
                        rgba[idx + 1] = pixel[1];
                        rgba[idx + 2] = pixel[2];
                        rgba[idx + 3] = pixel[3];
                    }
                }
            }
        }
    }
}

fn render_nametable_attribute_view(
    state: &nesium_runtime::TilemapState,
    ciram_offset: usize,
    offset_x: usize,
    offset_y: usize,
    rgba: &mut [u8],
    pitch: usize,
) {
    let ciram = &state.ciram;
    let palette = &state.palette;

    for tile_y in 0..30 {
        for tile_x in 0..32 {
            let attr_local_addr = 0x3C0 + (tile_y / 4) * 8 + (tile_x / 4);
            let attr_ciram_addr = ciram_offset + attr_local_addr;
            let attr_byte = if attr_ciram_addr < ciram.len() {
                ciram[attr_ciram_addr]
            } else {
                0
            };

            let shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
            let palette_index = ((attr_byte >> shift) & 0x03) as usize;

            let x0 = offset_x + tile_x * 8;
            let y0 = offset_y + tile_y * 8;

            for qy in 0..2 {
                for qx in 0..2 {
                    let color_index = (qy << 1) | qx; // 0..3
                    let nes_color = if color_index == 0 {
                        palette[0] as usize
                    } else {
                        let idx = palette_index * 4 + color_index;
                        palette.get(idx).copied().unwrap_or(palette[0]) as usize
                    } & 0x3F;

                    let pixel = state.bgra_palette[nes_color];
                    for py in 0..4 {
                        for px in 0..4 {
                            let screen_x = x0 + qx * 4 + px;
                            let screen_y = y0 + qy * 4 + py;
                            let idx = (screen_y * pitch + screen_x) * 4;
                            if idx + 3 < rgba.len() {
                                rgba[idx] = pixel[0];
                                rgba[idx + 1] = pixel[1];
                                rgba[idx + 2] = pixel[2];
                                rgba[idx + 3] = pixel[3];
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_grayscale_in_place(buf: &mut [u8]) {
    // Coefficients approximate Rec. 709: 0.2126R + 0.7152G + 0.0722B
    // Scaled by 256: (54, 183, 19).
    for px in buf.chunks_exact_mut(4) {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            let b = px[0] as u16;
            let g = px[1] as u16;
            let r = px[2] as u16;
            let y = ((54 * r + 183 * g + 19 * b) >> 8) as u8;
            px[0] = y;
            px[1] = y;
            px[2] = y;
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            let r = px[0] as u16;
            let g = px[1] as u16;
            let b = px[2] as u16;
            let y = ((54 * r + 183 * g + 19 * b) >> 8) as u8;
            px[0] = y;
            px[1] = y;
            px[2] = y;
        }
    }
}
