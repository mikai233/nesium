//! Background worker thread for asynchronous EventSender processing.
//!
//! This module offloads heavy event processing work (rendering, texture updates,
//! stream pushes) from the NES emulation thread to a dedicated worker thread.

use std::sync::OnceLock;
use std::thread;

use crossbeam_channel::{Receiver, Sender, bounded};
use nesium_core::cartridge::header::Mirroring;

use crate::api::events::{
    SpriteInfo, SpriteSnapshot, TileSnapshot, TilemapMirroring, TilemapSnapshot,
};
use crate::frb_generated::StreamSink;

/// Task payload for the event worker.
pub enum EventTask {
    /// Render tilemap to aux texture and optionally stream snapshot.
    Tilemap {
        state: Box<nesium_runtime::TilemapState>,
        sink: Option<StreamSink<TilemapSnapshot>>,
    },
    /// Update Tile aux texture and stream snapshot.
    Tile {
        state: Box<nesium_runtime::TileState>,
        sink: StreamSink<TileSnapshot>,
    },
    /// Update sprite aux textures and stream snapshot.
    Sprite {
        state: Box<nesium_runtime::SpriteState>,
        sink: StreamSink<SpriteSnapshot>,
    },
    /// Shutdown the worker thread.
    Shutdown,
}

/// Handle to submit tasks to the event worker.
pub struct EventWorkerHandle {
    sender: Sender<EventTask>,
}

impl EventWorkerHandle {
    /// Submits a task to the worker. Returns false if the worker is disconnected.
    pub fn send(&self, task: EventTask) -> bool {
        self.sender.send(task).is_ok()
    }
}

static EVENT_WORKER: OnceLock<EventWorkerHandle> = OnceLock::new();

/// Returns the global event worker handle, spawning the worker thread if needed.
pub fn event_worker() -> &'static EventWorkerHandle {
    EVENT_WORKER.get_or_init(|| {
        // Small buffer provides backpressure: if worker falls behind by 4 frames,
        // NES thread will block. In practice, the worker should easily keep up.
        let (tx, rx) = bounded::<EventTask>(4);
        thread::Builder::new()
            .name("event-worker".into())
            .spawn(move || worker_loop(rx))
            .expect("failed to spawn event worker");
        EventWorkerHandle { sender: tx }
    })
}

fn worker_loop(rx: Receiver<EventTask>) {
    for task in rx {
        match task {
            EventTask::Tilemap { state, sink } => {
                process_tilemap(&state, sink.as_ref());
            }
            EventTask::Tile { state, sink } => {
                process_tile_viewer(&state, &sink);
            }
            EventTask::Sprite { state, sink } => {
                process_sprite(&state, &sink);
            }
            EventTask::Shutdown => break,
        }
    }
}

/// Processes tilemap: renders to aux texture and optionally streams snapshot.
fn process_tilemap(
    state: &nesium_runtime::TilemapState,
    sink: Option<&StreamSink<TilemapSnapshot>>,
) {
    // Render to auxiliary texture (uses DashMap, no global lock contention)
    crate::senders::tilemap::render_tilemap_to_aux(state);

    // Stream snapshot to Flutter if sink is provided
    if let Some(sink) = sink {
        let mirroring = match state.mirroring {
            Mirroring::Horizontal => TilemapMirroring::Horizontal,
            Mirroring::Vertical => TilemapMirroring::Vertical,
            Mirroring::FourScreen => TilemapMirroring::FourScreen,
            Mirroring::SingleScreenLower => TilemapMirroring::SingleScreenLower,
            Mirroring::SingleScreenUpper => TilemapMirroring::SingleScreenUpper,
            Mirroring::MapperControlled => TilemapMirroring::MapperControlled,
        };

        // Convert BGRA to RGBA for Flutter
        let mut rgba_palette = Vec::with_capacity(64 * 4);
        for px in state.bgra_palette.iter() {
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                rgba_palette.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            }
            #[cfg(not(any(target_os = "macos", target_os = "ios")))]
            {
                rgba_palette.extend_from_slice(px);
            }
        }

        let snapshot = TilemapSnapshot {
            ciram: state.ciram.clone(),
            palette: state.palette.to_vec(),
            chr: state.chr.clone(),
            mirroring,
            bg_pattern_base: state.bg_pattern_base,
            rgba_palette,
            vram_addr: state.vram_addr,
            temp_addr: state.temp_addr,
            fine_x: state.fine_x,
        };

        let _ = sink.add(snapshot);
    }
}

/// Processes Tile Viewer: renders tiles to aux texture and streams snapshot.
fn process_tile_viewer(state: &nesium_runtime::TileState, sink: &StreamSink<TileSnapshot>) {
    // Render tiles from source_bytes (this was previously done in NES thread!)
    let rgba = render_tile_view_rgba(
        &state.source_bytes,
        state.column_count,
        state.row_count,
        state.layout,
        state.background,
        state.selected_palette,
        state.use_grayscale_palette,
        &state.palette,
        &state.bgra_palette,
    );

    // Update auxiliary texture with rendered data
    crate::aux_texture::aux_update(crate::senders::tile::TILE_VIEWER_TEXTURE_ID, &rgba);

    // Convert BGRA to RGBA for Flutter
    let mut rgba_palette = Vec::with_capacity(64 * 4);
    for px in state.bgra_palette.iter() {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            rgba_palette.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            rgba_palette.extend_from_slice(px);
        }
    }

    let snapshot = TileSnapshot {
        palette: state.palette.to_vec(),
        rgba_palette,
        selected_palette: state.selected_palette,
        width: state.width,
        height: state.height,
        source: state.source as u8,
        source_size: state.source_size,
        start_address: state.start_address,
        column_count: state.column_count,
        row_count: state.row_count,
        layout: state.layout as u8,
        background: state.background as u8,
        use_grayscale_palette: state.use_grayscale_palette,
        bg_pattern_base: state.bg_pattern_base,
        sprite_pattern_base: state.sprite_pattern_base,
        large_sprites: state.large_sprites,
    };

    let _ = sink.add(snapshot);
}

fn process_sprite(state: &nesium_runtime::SpriteState, sink: &StreamSink<SpriteSnapshot>) {
    // Build sprite info from raw OAM data and render textures
    let (sprites, thumbnails_rgba, screen_rgba) = build_and_render_sprites(state);

    // Update auxiliary textures
    crate::aux_texture::aux_update(crate::senders::sprite::SPRITE_TEXTURE_ID, &thumbnails_rgba);
    crate::aux_texture::aux_update(
        crate::senders::sprite::SPRITE_SCREEN_TEXTURE_ID,
        &screen_rgba,
    );

    // Convert sprites to FRB-compatible format
    let sprites: Vec<SpriteInfo> = sprites
        .iter()
        .map(|s| SpriteInfo {
            index: s.index,
            x: s.x,
            y: s.y,
            tile_index: s.tile_index,
            palette: s.palette,
            flip_h: s.flip_h,
            flip_v: s.flip_v,
            behind_bg: s.behind_bg,
            visible: s.visible,
        })
        .collect();

    // Convert BGRA to RGBA for Flutter
    let mut rgba_palette = Vec::with_capacity(64 * 4);
    for px in state.bgra_palette.iter() {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            rgba_palette.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            rgba_palette.extend_from_slice(px);
        }
    }

    let snapshot = SpriteSnapshot {
        sprites,
        thumbnail_width: state.thumbnail_width,
        thumbnail_height: state.thumbnail_height,
        large_sprites: state.large_sprites,
        pattern_base: state.pattern_base,
        rgba_palette,
    };

    let _ = sink.add(snapshot);
}

/// Builds sprite info from raw OAM data and renders thumbnails + screen preview.
/// This is the expensive logic that was moved from runner.rs.
fn build_and_render_sprites(
    state: &nesium_runtime::SpriteState,
) -> (Vec<nesium_runtime::SpriteInfo>, Vec<u8>, Vec<u8>) {
    let large_sprites = state.large_sprites;
    let sprite_height: u8 = if large_sprites { 16 } else { 8 };
    let pattern_base = state.pattern_base;
    let oam = &state.oam;
    let chr = &state.chr;
    let palette_ram = &state.palette;
    let bgra_palette = &state.bgra_palette;

    // Background color (transparent)
    let bg_pixel = [0u8, 0, 0, 0];

    let mut sprites = Vec::with_capacity(64);

    // Parse 64 sprites from OAM (4 bytes each)
    for i in 0..64 {
        let base = i * 4;
        if base + 3 >= oam.len() {
            break;
        }

        let y = oam[base];
        let tile_index = oam[base + 1];
        let attr = oam[base + 2];
        let x = oam[base + 3];

        let palette = attr & 0x03;
        let behind_bg = (attr & 0x20) != 0;
        let flip_h = (attr & 0x40) != 0;
        let flip_v = (attr & 0x80) != 0;
        let visible = y < 239;

        sprites.push(nesium_runtime::SpriteInfo {
            index: i as u8,
            x,
            y,
            tile_index,
            palette,
            flip_h,
            flip_v,
            behind_bg,
            visible,
        });
    }

    // Render thumbnails: 64 sprites × 8×(8 or 16) each, in an 8×8 grid
    let thumb_w = 8 * 8; // 64 pixels wide
    let thumb_h = sprite_height as usize * 8; // 64 or 128 pixels tall
    let mut thumbnails_rgba = vec![0u8; thumb_w * thumb_h * 4];

    for (i, sprite) in sprites.iter().enumerate() {
        let dest_x = (i % 8) * 8;
        let dest_y = (i / 8) * sprite_height as usize;
        render_single_sprite(
            &mut thumbnails_rgba,
            thumb_w,
            thumb_h,
            dest_x as isize,
            dest_y as isize,
            sprite,
            sprite_height as usize,
            large_sprites,
            pattern_base,
            chr,
            palette_ram,
            bgra_palette,
            &bg_pixel,
        );
    }

    // Render screen preview: 256×256 with sprites at their positions
    let screen_w = 256;
    let screen_h = 256;
    let mut screen_rgba = vec![0u8; screen_w * screen_h * 4];

    // Draw sprites in reverse order (sprite 0 on top)
    for sprite in sprites.iter().rev() {
        let dest_x = sprite.x as isize;
        let dest_y = (sprite.y as isize).wrapping_add(1); // Y offset by 1
        render_single_sprite(
            &mut screen_rgba,
            screen_w,
            screen_h,
            dest_x,
            dest_y,
            sprite,
            sprite_height as usize,
            large_sprites,
            pattern_base,
            chr,
            palette_ram,
            bgra_palette,
            &bg_pixel,
        );
    }

    (sprites, thumbnails_rgba, screen_rgba)
}

/// Renders a single sprite to the destination buffer.
#[allow(clippy::too_many_arguments)]
fn render_single_sprite(
    dst: &mut [u8],
    dst_w: usize,
    dst_h: usize,
    dest_x: isize,
    dest_y: isize,
    sprite: &nesium_runtime::SpriteInfo,
    sprite_h: usize,
    large_sprites: bool,
    pattern_base: u16,
    chr: &[u8],
    palette_ram: &[u8; 32],
    bgra_palette: &[[u8; 4]; 64],
    _bg_pixel: &[u8; 4],
) {
    for y_out in 0..sprite_h {
        let y_src = if sprite.flip_v {
            sprite_h.saturating_sub(1).saturating_sub(y_out)
        } else {
            y_out
        };

        let (tile_select, row_in_tile) = if large_sprites {
            (y_src / 8, y_src % 8)
        } else {
            (0, y_src)
        };

        let tile_pattern_base = if large_sprites {
            if (sprite.tile_index & 0x01) != 0 {
                0x1000usize
            } else {
                0x0000usize
            }
        } else {
            pattern_base as usize
        };

        let tile_idx: u16 = if large_sprites {
            let base_tile = (sprite.tile_index & 0xFE) as u16;
            base_tile.wrapping_add(tile_select as u16)
        } else {
            sprite.tile_index as u16
        };

        let tile_addr = tile_pattern_base + (tile_idx as usize) * 16;
        let lo = chr.get(tile_addr + row_in_tile).copied().unwrap_or(0);
        let hi = chr.get(tile_addr + row_in_tile + 8).copied().unwrap_or(0);

        for x_out in 0..8usize {
            let bit = if sprite.flip_h { x_out } else { 7 - x_out };
            let lo_bit = (lo >> bit) & 1;
            let hi_bit = (hi >> bit) & 1;
            let color_idx = ((hi_bit << 1) | lo_bit) as usize;

            if color_idx == 0 {
                continue; // Transparent
            }

            let palette_offset = 0x10 + (sprite.palette as usize) * 4 + color_idx;
            let nes_color = palette_ram.get(palette_offset).copied().unwrap_or(0) as usize;
            let pixel_color = bgra_palette[nes_color & 0x3F];

            let px = dest_x + x_out as isize;
            let py = dest_y + y_out as isize;
            if px < 0 || py < 0 {
                continue;
            }
            let px = px as usize;
            let py = py as usize;
            if px >= dst_w || py >= dst_h {
                continue;
            }

            let di = (py * dst_w + px) * 4;
            if di + 3 < dst.len() {
                dst[di] = pixel_color[0];
                dst[di + 1] = pixel_color[1];
                dst[di + 2] = pixel_color[2];
                dst[di + 3] = pixel_color[3];
            }
        }
    }
}

// ============================================================================
// Tile rendering (moved from runner.rs to offload from NES thread)
// ============================================================================

use nesium_runtime::{TileViewerBackground, TileViewerLayout};

fn render_tile_view_rgba(
    source_bytes: &[u8],
    column_count: u16,
    row_count: u16,
    layout: TileViewerLayout,
    background: TileViewerBackground,
    selected_palette: u8,
    use_grayscale_palette: bool,
    palette_ram: &[u8; 32],
    bgra_palette: &[[u8; 4]; 64],
) -> Vec<u8> {
    let width = column_count as usize * 8;
    let height = row_count as usize * 8;
    let mut rgba = vec![0u8; width.saturating_mul(height).saturating_mul(4)];

    let bytes_per_tile = 16usize;
    let palette_index = (selected_palette as usize).min(7);
    let pal_base = if palette_index < 4 {
        palette_index * 4
    } else {
        0x10 + (palette_index - 4) * 4
    };

    for ty in 0..row_count as usize {
        for tx in 0..column_count as usize {
            let (mx, my) = tile_viewer_from_layout(layout, tx, ty, column_count as usize);
            let tile_index = my.saturating_mul(column_count as usize).saturating_add(mx);
            let base = tile_index.saturating_mul(bytes_per_tile);
            if base + 15 >= source_bytes.len() {
                continue;
            }

            for py in 0..8usize {
                let plane0 = source_bytes[base + py];
                let plane1 = source_bytes[base + py + 8];
                for px in 0..8usize {
                    let bit = 7 - px;
                    let lo = (plane0 >> bit) & 1;
                    let hi = (plane1 >> bit) & 1;
                    let color_index = ((hi << 1) | lo) as usize;

                    let pixel = if color_index == 0 {
                        match background {
                            TileViewerBackground::Default => {
                                let nes = (palette_ram[0] & 0x3F) as usize;
                                bgra_palette.get(nes).copied().unwrap_or([0, 0, 0, 0xFF])
                            }
                            TileViewerBackground::Transparent => [0, 0, 0, 0],
                            TileViewerBackground::PaletteColor => {
                                let idx = pal_base.min(palette_ram.len().saturating_sub(1));
                                let nes = (palette_ram[idx] & 0x3F) as usize;
                                bgra_palette.get(nes).copied().unwrap_or([0, 0, 0, 0xFF])
                            }
                            TileViewerBackground::Black => [0, 0, 0, 0xFF],
                            TileViewerBackground::White => [0xFF, 0xFF, 0xFF, 0xFF],
                            TileViewerBackground::Magenta => [0xFF, 0, 0xFF, 0xFF],
                        }
                    } else {
                        let idx = pal_base + color_index;
                        let nes = if idx < palette_ram.len() {
                            (palette_ram[idx] & 0x3F) as usize
                        } else {
                            0
                        };
                        bgra_palette.get(nes).copied().unwrap_or([0, 0, 0, 0xFF])
                    };

                    let sx = tx * 8 + px;
                    let sy = ty * 8 + py;
                    let di = (sy * width + sx) * 4;
                    if di + 3 < rgba.len() {
                        rgba[di] = pixel[0];
                        rgba[di + 1] = pixel[1];
                        rgba[di + 2] = pixel[2];
                        rgba[di + 3] = pixel[3];
                    }
                }
            }
        }
    }

    if use_grayscale_palette {
        apply_grayscale_in_place(&mut rgba);
    }

    rgba
}

/// Maps (tx, ty) to (mx, my) based on the tile viewer layout.
fn tile_viewer_from_layout(
    layout: TileViewerLayout,
    tx: usize,
    ty: usize,
    column_count: usize,
) -> (usize, usize) {
    match layout {
        TileViewerLayout::Normal => (tx, ty),
        TileViewerLayout::SingleLine8x16 => {
            // 8×16 sprites: two consecutive tiles vertically
            let pair = tx / 2;
            let offset = tx % 2;
            let mx = pair + (ty / 2) * (column_count / 2);
            let my = offset + (ty % 2) * 2;
            let linear = my * column_count + mx;
            (linear % column_count, linear / column_count)
        }
        TileViewerLayout::SingleLine16x16 => {
            // 16×16 sprites: four tiles in a 2×2 block
            let block_x = tx / 2;
            let block_y = ty / 2;
            let offset_x = tx % 2;
            let offset_y = ty % 2;
            let block_index = block_y * (column_count / 2) + block_x;
            let tile_in_block = offset_y * 2 + offset_x;
            let linear = block_index * 4 + tile_in_block;
            (linear % column_count, linear / column_count)
        }
    }
}

/// Applies grayscale conversion in place.
fn apply_grayscale_in_place(rgba: &mut [u8]) {
    for chunk in rgba.chunks_exact_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
        chunk[0] = gray;
        chunk[1] = gray;
        chunk[2] = gray;
    }
}
