use std::any::Any;

use nesium_runtime::{DebugState, Event, NotificationEvent, RuntimeEventSender};

use crate::api::events::{DebugStateNotification, RuntimeNotification, RuntimeNotificationKind};
use crate::frb_generated::StreamSink;

/// Sender that forwards RuntimeEvent to Flutter as RuntimeNotification.
pub struct FlutterRuntimeEventSender {
    pub(crate) sink: StreamSink<RuntimeNotification>,
}

impl FlutterRuntimeEventSender {
    pub fn new(sink: StreamSink<RuntimeNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for FlutterRuntimeEventSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(notification) = any.downcast::<NotificationEvent>() {
            let notification = match *notification {
                NotificationEvent::AudioInitFailed { error } => RuntimeNotification {
                    kind: RuntimeNotificationKind::AudioInitFailed,
                    error: Some(error),
                },
            };
            let _ = self.sink.add(notification);
            return true;
        }
        true
    }
}

/// Sender that forwards DebugState events to Flutter.
pub struct FlutterDebugEventSender {
    pub(crate) sink: StreamSink<DebugStateNotification>,
}

impl FlutterDebugEventSender {
    pub fn new(sink: StreamSink<DebugStateNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for FlutterDebugEventSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<DebugState>() {
            let notification = DebugStateNotification {
                cpu_pc: state.cpu.pc,
                cpu_a: state.cpu.a,
                cpu_x: state.cpu.x,
                cpu_y: state.cpu.y,
                cpu_sp: state.cpu.sp,
                cpu_status: state.cpu.status,
                cpu_cycle: state.cpu.cycle,
                ppu_scanline: state.ppu.scanline,
                ppu_cycle: state.ppu.cycle,
                ppu_frame: state.ppu.frame,
                ppu_ctrl: state.ppu.ctrl,
                ppu_mask: state.ppu.mask,
                ppu_status: state.ppu.status,
            };
            let _ = self.sink.add(notification);
        }
        true
    }
}

/// Auxiliary texture ID for Tilemap Viewer.
pub const TILEMAP_TEXTURE_ID: u32 = 1;

/// Sender that renders TilemapState directly to auxiliary texture (no stream).
pub struct TilemapTextureSender;

impl RuntimeEventSender for TilemapTextureSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn std::any::Any> = event;
        if let Ok(state) = any.downcast::<nesium_runtime::TilemapState>() {
            render_tilemap_to_aux(&state);
        }
        true
    }
}

/// Renders tilemap state to RGBA and updates the auxiliary texture.
fn render_tilemap_to_aux(state: &nesium_runtime::TilemapState) {
    // Output: 512x480 (2x2 nametables: 256x240 each)
    const WIDTH: usize = 512;
    const HEIGHT: usize = 480;

    let mut rgba = vec![0u8; WIDTH * HEIGHT * 4];

    // Apply nametable mirroring
    let nt0 = mirror_nametable_addr(0x2000, state.mirroring);
    let nt1 = mirror_nametable_addr(0x2400, state.mirroring);
    let nt2 = mirror_nametable_addr(0x2800, state.mirroring);
    let nt3 = mirror_nametable_addr(0x2C00, state.mirroring);

    // Render 4 nametables with mirroring applied
    render_nametable(state, nt0, 0, 0, &mut rgba, WIDTH); // Top-Left ($2000)
    render_nametable(state, nt1, 256, 0, &mut rgba, WIDTH); // Top-Right ($2400)
    render_nametable(state, nt2, 0, 240, &mut rgba, WIDTH); // Bottom-Left ($2800)
    render_nametable(state, nt3, 256, 240, &mut rgba, WIDTH); // Bottom-Right ($2C00)

    crate::aux_texture::aux_update(TILEMAP_TEXTURE_ID, &rgba);
}

/// Maps a nametable address ($2000-$2FFF) to its physical location based on mirroring mode.
/// Mirroring modes: 0=Horizontal, 1=Vertical, 2=FourScreen, 3=SingleScreenLower, 4=SingleScreenUpper
fn mirror_nametable_addr(addr: usize, mirroring: u8) -> usize {
    // Convert $2000-$2FFF range to nametable index (0-3)
    let nt_index = (addr >> 10) & 0x03; // 0=$2000, 1=$2400, 2=$2800, 3=$2C00

    let physical_nt = match mirroring {
        0 => {
            // Horizontal: $2000=$2400, $2800=$2C00
            // (0,1 -> 0), (2,3 -> 1) then mapped to $2000, $2800
            match nt_index {
                0 | 1 => 0,
                _ => 2,
            }
        }
        1 => {
            // Vertical: $2000=$2800, $2400=$2C00
            // (0,2 -> 0), (1,3 -> 1) then mapped to $2000, $2400
            match nt_index {
                0 | 2 => 0,
                _ => 1,
            }
        }
        2 => {
            // FourScreen: all 4 are independent
            nt_index
        }
        3 => {
            // SingleScreenLower: all map to $2000
            0
        }
        4 => {
            // SingleScreenUpper: all map to $2400
            1
        }
        _ => nt_index, // Mapper controlled or unknown - use as-is
    };

    0x2000 + (physical_nt * 0x400)
}

/// Renders a single 256x240 nametable into the RGBA buffer at (offset_x, offset_y).
fn render_nametable(
    state: &nesium_runtime::TilemapState,
    nt_offset: usize,
    offset_x: usize,
    offset_y: usize,
    rgba: &mut [u8],
    pitch: usize,
) {
    let vram = &state.vram;
    let chr = &state.chr;
    let palette = &state.palette;

    // Nametable is 32x30 tiles, each 8x8 pixels
    for tile_y in 0..30 {
        for tile_x in 0..32 {
            let nt_addr = nt_offset + tile_y * 32 + tile_x;
            let tile_index = if nt_addr < vram.len() {
                vram[nt_addr] as usize
            } else {
                0
            };

            // Attribute table is at offset 0x3C0 within each nametable
            let attr_addr = nt_offset + 0x3C0 + (tile_y / 4) * 8 + (tile_x / 4);
            let attr_byte = if attr_addr < vram.len() {
                vram[attr_addr]
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

                    let bgra = state.bgra_palette[nes_color];

                    let screen_x = offset_x + tile_x * 8 + px;
                    let screen_y = offset_y + tile_y * 8 + py;
                    let idx = (screen_y * pitch + screen_x) * 4;

                    if idx + 3 < rgba.len() {
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        {
                            rgba[idx] = bgra[0]; // B
                            rgba[idx + 1] = bgra[1]; // G
                            rgba[idx + 2] = bgra[2]; // R
                        }
                        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
                        {
                            rgba[idx] = bgra[2]; // R
                            rgba[idx + 1] = bgra[1]; // G
                            rgba[idx + 2] = bgra[0]; // B
                        }
                        rgba[idx + 3] = bgra[3]; // A
                    }
                }
            }
        }
    }
}
