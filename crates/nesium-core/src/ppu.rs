//! NES Picture Processing Unit (PPU) implementation with cycle-level timing.
//!
//! **Quick primer for newcomers**
//! - The PPU draws 262 scanlines per frame. Scanline `-1` is the *prerender*
//!   line, `0..=239` are visible, `240` is post-render, and `241..=260` are
//!   vblank. Each scanline has 341 PPU cycles ("dots").
//! - CPU sees eight registers at `$2000-$2007` (mirrored). Most of the PPU
//!   state lives in tiny internal latches and shift registers; mirroring that
//!   behavior is what makes the code look odd in places.
//! - The hardware treats "background" (tiles) and "sprites" separately. Each
//!   side has 16-bit shifters that push out one pixel per dot while fetch units
//!   refill them every 8 dots.
//! - Some features depend on *which* cycle or scanline you are on (odd-frame
//!   skipped tick, sprite evaluation windows, scroll copies). Those checks
//!   are explicit in `clock()`.
//!
//! **Why some code looks strange**
//! - The odd-frame skip on prerender: real hardware drops cycle 0 on odd
//!   frames when rendering is enabled. We model that by jumping `cycle` from 0
//!   to 1 and bailing early.
//! - The `bus_latch`: untouched register reads return the last value that was
//!   on the data bus (open-bus behavior). We store that in `bus_latch` and feed
//!   it back for "unknown" reads.
//! - OAM reads during rendering: hardware doesn’t expose live primary OAM then;
//!   we return a constant `0xFF` to approximate the internal bus noise.
//! - Sprite overflow is still an approximation; the hardware bug relies on
//!   suppressed secondary-OAM writes in a specific pattern (see TODO in
//!   `SpriteEvalPhase::OverflowScan`).
//! - Palette RAM has mirroring quirks ($3F10 mirrors $3F00, etc.). Those rules
//!   are handled in `palette::PaletteIndex::mirrored_addr` and `PaletteRam`.

pub mod palette;

mod background_pipeline;
mod registers;
mod sprite;
mod sprite_pipeline;
mod sprite_state;

use self::{
    background_pipeline::BgPipeline,
    sprite_pipeline::SpritePipeline,
    sprite_state::{SpriteEvalPhase, SpriteEvalState, SpriteFetchState, SpriteLineBuffers},
};

use core::fmt;

use crate::{
    cartridge::{Cartridge, header::Mirroring},
    memory::ppu::{self as ppu_mem, Register as PpuRegister},
    ppu::{
        palette::PaletteRam,
        registers::{Control, Mask, Registers, Status},
    },
    ram::ppu::{SecondaryOamRam, Vram},
};

/// Minimal PPU timing/debug snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NmiDebugState {
    pub nmi_output: bool,
    pub nmi_pending: bool,
    pub scanline: i16,
    pub cycle: u16,
    pub frame: u64,
}
pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
const CYCLES_PER_SCANLINE: u16 = 341;
const SCANLINES_PER_FRAME: i16 = 262; // -1 (prerender) + 0..239 visible + post + vblank (241..260)

/// Captures the position of the first sprite-0 hit in the current frame (debug).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sprite0HitPos {
    pub scanline: i16,
    pub cycle: u16,
}

/// Debug info captured on the first sprite-0 hit of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sprite0HitDebug {
    pub pos: Sprite0HitPos,
    pub oam: [u8; 4],
}

/// Entry points for the CPU PPU register mirror.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ppu {
    /// Collection of CPU visible registers and their helper latches.
    registers: Registers,
    /// Internal VRAM backing store for nametables and pattern tables.
    vram: Vram,
    /// Dedicated palette RAM. Addresses between `$3F00` and `$3FFF` map here.
    palette_ram: PaletteRam,
    /// Current dot (0..=340) within the active scanline.
    cycle: u16,
    /// Current scanline. `-1` is the prerender line, `0..239` are visible.
    scanline: i16,
    /// Total number of frames produced so far.
    frame: u64,
    /// Tracks whether the current frame is odd. Required for the skipped tick logic.
    odd_frame: bool,
    /// Background pixel pipeline (pattern and attribute shifters).
    bg_pipeline: BgPipeline,
    /// Sprite pixel pipeline for the current scanline.
    sprite_pipeline: SpritePipeline,
    /// Latched NMI request (true when the PPU wants to fire NMI).
    nmi_pending: bool,
    /// Current level of the NMI output line (true = asserted).
    nmi_output: bool,
    /// First sprite-0 hit debug info in the current frame (debug).
    sprite0_hit_pos: Option<Sprite0HitDebug>,
    /// Last value on the (simulated) PPU data bus for open-bus behavior.
    bus_latch: u8,
    /// Secondary OAM used during sprite evaluation for the current scanline.
    secondary_oam: SecondaryOamRam,
    /// Sprite evaluation state (cycle-accurate structure).
    sprite_eval: SpriteEvalState,
    /// Sprite fetch state for dots 257..=320.
    sprite_fetch: SpriteFetchState,
    /// Buffered secondary-OAM sprite bytes/patterns for the next scanline.
    sprite_line_next: SpriteLineBuffers,
    /// Background + sprite rendering target for the current frame.
    framebuffer: Box<[u8; SCREEN_WIDTH * SCREEN_HEIGHT]>,
}

/// Temporary view that lets the PPU reach the cartridge CHR space without storing a raw pointer.
///
/// The bus creates one of these per PPU call, so lifetimes remain explicit and borrow-checked.
#[derive(Default)]
pub struct PatternBus<'a> {
    cartridge: Option<&'a mut Cartridge>,
}

impl<'a> PatternBus<'a> {
    pub fn new(cartridge: Option<&'a mut Cartridge>) -> Self {
        Self { cartridge }
    }

    pub fn none() -> Self {
        Self { cartridge: None }
    }

    pub fn from_cartridge(cartridge: &'a mut Cartridge) -> Self {
        Self {
            cartridge: Some(cartridge),
        }
    }

    fn read(&mut self, addr: u16) -> Option<u8> {
        self.cartridge
            .as_deref_mut()
            .map(|cart| cart.ppu_read(addr))
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_write(addr, value);
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.cartridge
            .as_deref()
            .map(|cart| cart.mirroring())
            .unwrap_or(Mirroring::Horizontal)
    }
}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ppu")
            .field("registers", &self.registers)
            .field("cycle", &self.cycle)
            .field("scanline", &self.scanline)
            .field("frame", &self.frame)
            .field("odd_frame", &self.odd_frame)
            .finish()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    /// Creates a new PPU instance with cleared VRAM and default register values.
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            vram: Vram::new(),
            palette_ram: PaletteRam::new(),
            cycle: 0,
            scanline: -1,
            frame: 0,
            odd_frame: false,
            bg_pipeline: BgPipeline::new(),
            sprite_pipeline: SpritePipeline::new(),
            nmi_pending: false,
            nmi_output: false,
            sprite0_hit_pos: None,
            bus_latch: 0,
            secondary_oam: SecondaryOamRam::new(),
            sprite_eval: SpriteEvalState::default(),
            sprite_fetch: SpriteFetchState::default(),
            sprite_line_next: SpriteLineBuffers::new(),
            framebuffer: Box::new([0; SCREEN_WIDTH * SCREEN_HEIGHT]),
        }
    }

    /// Preloads CHR ROM/RAM into VRAM when a cartridge is inserted.
    ///
    /// Pattern table accesses still flow through the mapper via [`PatternBus`];
    /// this helper just seeds VRAM for setups that poke pattern RAM directly.
    pub(crate) fn attach_cartridge(&mut self, cartridge: &Cartridge) {
        if let Some(chr) = cartridge.mapper().chr_rom() {
            self.load_chr(chr);
        }
    }

    /// Restores the device to its power-on state.
    ///
    /// Wipes VRAM/palette, resets scroll/state latches, and clears pixel
    /// pipelines so the next frame starts from a clean slate (mirrors hardware
    /// cold boot).
    pub fn reset(&mut self) {
        self.registers.reset();
        self.vram.fill(0);
        self.palette_ram.fill(0);
        self.cycle = 0;
        self.scanline = -1;
        self.frame = 0;
        self.odd_frame = false;
        self.bg_pipeline.clear();
        self.sprite_pipeline.clear();
        self.nmi_pending = false;
        self.nmi_output = false;
        self.sprite0_hit_pos = None;
        // PPU power-on: clear VBlank flag so the first BIT $2002 loop waits
        // for the true VBlank edge instead of seeing a stale high.
        self.registers
            .status
            .remove(registers::Status::VERTICAL_BLANK);
        self.bus_latch = 0;
        self.secondary_oam.fill(0);
        self.sprite_eval = SpriteEvalState::default();
        self.sprite_fetch = SpriteFetchState::default();
        self.sprite_line_next.clear();
        self.clear_framebuffer();
    }

    /// Current NMI output level: true when VBLANK is set and NMI is enabled.
    pub fn nmi_output(&self) -> bool {
        self.nmi_output
    }

    /// Returns an immutable view of the current framebuffer.
    ///
    /// Each entry is a palette index (0..=63) which can be resolved using
    /// the palette RAM and a host-side color palette.
    pub fn framebuffer(&self) -> &[u8] {
        &*self.framebuffer
    }

    /// Current frame counter (increments when scanline wraps from 260 to -1).
    pub fn frame_count(&self) -> u64 {
        self.frame
    }

    /// Debug info about NMI output/pending and position.
    pub(crate) fn debug_nmi_state(&self) -> NmiDebugState {
        NmiDebugState {
            nmi_output: self.nmi_output,
            nmi_pending: self.nmi_pending,
            scanline: self.scanline,
            cycle: self.cycle,
            frame: self.frame,
        }
    }

    /// First sprite-0 hit position for the current frame (if any).
    pub(crate) fn sprite0_hit_pos(&self) -> Option<Sprite0HitDebug> {
        self.sprite0_hit_pos
    }

    /// Clears the framebuffer to palette index 0.
    fn clear_framebuffer(&mut self) {
        self.framebuffer.fill(0);
    }

    /// Copies CHR ROM/RAM contents into the pattern table window (`$0000-$1FFF`).
    ///
    /// This is a temporary bridge until full PPU<->mapper wiring is in place.
    pub fn load_chr(&mut self, chr: &[u8]) {
        let len = chr.len().min(0x2000);
        self.vram[..len].copy_from_slice(&chr[..len]);
    }

    /// Handles CPU writes to the mirrored PPU register space (`$2000-$3FFF`).
    ///
    /// Mirrors open-bus semantics by latching the last value written; the
    /// hardware leaves that value on the data bus.
    pub fn cpu_write(&mut self, addr: u16, value: u8, pattern: &mut PatternBus<'_>) {
        self.bus_latch = value;
        match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Control => {
                let prev_output = self.nmi_output;
                let old_ctrl = self.registers.control.bits();
                let old_nmi_en = self.registers.control.nmi_enabled();

                self.registers.write_control(value);

                let new_ctrl = self.registers.control.bits();
                let new_nmi_en = self.registers.control.nmi_enabled();

                #[cfg(debug_assertions)]
                {
                    if old_ctrl != new_ctrl {
                        eprintln!(
                            "[PPU] PPUCTRL write {:02X}->{:02X} (NMI_EN {}->{}) at frame {}, scanline {}, cycle {}",
                            old_ctrl,
                            new_ctrl,
                            old_nmi_en,
                            new_nmi_en,
                            self.frame,
                            self.scanline,
                            self.cycle,
                        );
                    }
                }

                self.update_nmi_output(prev_output);
            }
            PpuRegister::Mask => self.registers.mask = Mask::from_bits_retain(value),
            PpuRegister::Status => {} // read-only
            PpuRegister::OamAddr => self.registers.oam_addr = value,
            PpuRegister::OamData => self.write_oam_data(value),
            PpuRegister::Scroll => self.registers.vram.write_scroll(value),
            PpuRegister::Addr => self.registers.vram.write_addr(value),
            PpuRegister::Data => self.write_vram_data(value, pattern),
        }
    }

    /// Handles CPU reads from the mirrored PPU register space (`$2000-$3FFF`).
    ///
    /// Unhandled reads return the last bus value to approximate open-bus
    /// behavior.
    pub fn cpu_read(&mut self, addr: u16, pattern: &mut PatternBus<'_>) -> u8 {
        let value = match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Status => self.read_status(),
            PpuRegister::OamData => self.read_oam_data(),
            PpuRegister::Data => self.read_vram_data(pattern),
            _ => self.bus_latch,
        };
        self.bus_latch = value;
        value
    }

    /// Advances the PPU by a single dot, keeping cycle and frame counters up to date.
    ///
    /// This is the main timing entry: it performs background/sprite pipeline
    /// work, runs fetch windows, and renders pixels on visible scanlines. Call
    /// three times per CPU tick for NTSC timing.
    pub fn clock(&mut self, pattern: &mut PatternBus<'_>) {
        let rendering_enabled = self.registers.mask.rendering_enabled();
        let prev_nmi_output = self.nmi_output;

        // NOTE: We clear VBlank and drop NMI output at dot 1 of prerender.
        // Dot 0 should NOT clear VBlank on normal frames (avoids early NMI fall).
        // For odd-frame skip, we clear explicitly in the skip path below.

        // Odd-frame cycle skip: on prerender line, when rendering is enabled,
        // the PPU omits cycle 0 on odd frames.
        if self.scanline == -1 && self.cycle == 0 && self.odd_frame && rendering_enabled {
            // Odd-frame prerender skip: hardware still clears VBlank/flags at dot 1.
            // Since we skip dot 1 processing, clear here.
            self.registers.status.remove(Status::VERTICAL_BLANK);
            self.registers
                .status
                .remove(Status::SPRITE_OVERFLOW | Status::SPRITE_ZERO_HIT);
            self.sprite0_hit_pos = None;
            self.nmi_output = false;
            self.nmi_pending = false;
            self.cycle = 1;
            // Skip the rest of match processing for this dot.
            self.update_nmi_output(prev_nmi_output);
            self.advance_cycle();
            return;
        }

        // Load sprite shifters for the new scanline before rendering begins.
        if self.cycle == 1 && (0..=239).contains(&self.scanline) {
            self.sprite_pipeline.load_scanline(
                self.sprite_eval.count,
                self.sprite_eval.sprite0_in_range_next,
                self.sprite_line_next.attr_slice(),
                self.sprite_line_next.x_slice(),
                self.sprite_line_next.pattern_low_slice(),
                self.sprite_line_next.pattern_high_slice(),
            );
        }

        // If rendering is disabled, keep pipelines idle to avoid stale data.
        if !rendering_enabled {
            self.bg_pipeline.clear();
            self.sprite_pipeline.clear();
            self.sprite_line_next.clear();
        }

        match (self.scanline, self.cycle) {
            // ----------------------
            // Pre-render scanline (-1)
            // ----------------------
            // Dot 0 has no special side effects (prerender clears happen at dot 1).
            (_, 0) => {}

            (-1, 1) => {
                // Clear vblank/sprite flags at dot 1 of prerender.
                self.registers.status.remove(Status::VERTICAL_BLANK);
                self.registers
                    .status
                    .remove(Status::SPRITE_OVERFLOW | Status::SPRITE_ZERO_HIT);
                // Drop per-frame sprite0 debug info when vblank ends.
                self.sprite0_hit_pos = None;
                // Debug latch is per-VBlank; drop it when VBlank ends.
                self.nmi_pending = false;
            }

            // Background pipeline ticks during prerender dots 1..=256 when rendering is enabled.
            (-1, 1..=256) => {
                if rendering_enabled {
                    // Shift background shifters each dot.
                    let _ = self.bg_pipeline.sample_and_shift(self.registers.vram.x);
                    // Fetch/reload background data at tile boundaries.
                    self.fetch_background_data(pattern);
                    // Sprite evaluation for scanline 0 happens on prerender as well.
                    self.sprite_pipeline_eval_tick();

                    if self.cycle == 256 {
                        self.increment_scroll_y();
                    }
                }
            }

            // Dot 257: copy horizontal scroll bits from t -> v.
            (-1, 257) => {
                if rendering_enabled {
                    self.copy_horizontal_scroll();
                }
            }

            // Dots 258..=320: sprite pattern fetches for scanline 0.
            // During dots 280..=304, vertical scroll bits are also copied from t -> v.
            (-1, 258..=320) => {
                if rendering_enabled {
                    if (280..=304).contains(&self.cycle) {
                        self.copy_vertical_scroll();
                    }
                    self.sprite_pipeline_fetch_tick(pattern);
                }
            }

            // Dots 321..=336: prefetch first two background tiles for scanline 0.
            (-1, 321..=336) => {
                if rendering_enabled {
                    let _ = self.bg_pipeline.sample_and_shift(self.registers.vram.x);
                    self.fetch_background_data(pattern);
                }
            }

            // Dots 337..=340: idle/dummy fetches on prerender.
            (-1, 337..=340) => {}

            // ----------------------
            // Visible scanlines (0..=239)
            // ----------------------
            (0..=239, 1..=256) => {
                // Render current pixel.
                self.render_pixel();

                if rendering_enabled {
                    // Fetch background data for this dot.
                    self.fetch_background_data(pattern);
                    // Sprite evaluation for the *next* scanline runs during dots 1..=256.
                    self.sprite_pipeline_eval_tick();

                    if self.cycle == 256 {
                        self.increment_scroll_y();
                    }
                }
            }

            // Dot 257: copy horizontal scroll bits for next scanline.
            (0..=239, 257) => {
                if rendering_enabled {
                    self.copy_horizontal_scroll();
                }
            }

            // Dots 258..=320: sprite pattern fetches for the next scanline.
            (0..=239, 258..=320) => {
                if rendering_enabled {
                    self.sprite_pipeline_fetch_tick(pattern);
                }
            }

            // Dots 321..=336: prefetch first two background tiles for next scanline.
            (0..=239, 321..=336) => {
                if rendering_enabled {
                    let _ = self.bg_pipeline.sample_and_shift(self.registers.vram.x);
                    self.fetch_background_data(pattern);
                }
            }

            // Dots 337..=340: dummy nametable fetches (no visible effect).
            (0..=239, 337..=340) => {}

            // ----------------------
            // Post-render scanline (240)
            // ----------------------
            (240, _) => {}

            // ----------------------
            // VBlank scanlines (241..=260)
            // ----------------------
            (241, 1) => {
                // Enter vblank at scanline 241, dot 1.
                // NMI is edge-triggered from the NMI line (VBLANK && NMI_ENABLE)
                // in `update_nmi_output`; no separate latch here.
                self.registers.status.insert(Status::VERTICAL_BLANK);
            }
            (241..=260, _) => {}

            // Any other scanline value indicates a bug in the timing logic.
            _ => unreachable!("PPU scanline {} out of range", self.scanline),
        }

        self.update_nmi_output(prev_nmi_output);
        self.advance_cycle();
    }

    /// Debug helper: overrides PPU position counters (scanline/cycle/frame).
    /// Intended for trace alignment only.
    pub(crate) fn debug_set_position(&mut self, scanline: i16, cycle: u16, frame: u64) {
        self.scanline = scanline;
        self.cycle = cycle;
        self.frame = frame;
        self.odd_frame = frame % 2 == 1;
    }

    /// Advances to the next dot / scanline / frame.
    fn advance_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle >= CYCLES_PER_SCANLINE {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline > 260 {
                self.scanline = -1;
                self.frame = self.frame.wrapping_add(1);
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    /// Recomputes the NMI output line based on VBlank and control register,
    /// latching a pending NMI on rising edges.
    fn update_nmi_output(&mut self, prev_output: bool) {
        let new_output = self.registers.status.contains(Status::VERTICAL_BLANK)
            && self.registers.control.nmi_enabled();
        self.nmi_output = new_output;

        if self.nmi_output && !prev_output {
            self.nmi_pending = true;
        }
    }

    /// Renders a single pixel into the framebuffer based on the current
    /// background and sprite pipeline state.
    ///
    /// Samples shifters, applies left-edge masks, resolves priority, flags
    /// sprite-0 hits, then looks up the final palette color.
    fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        if x >= SCREEN_WIDTH || y >= SCREEN_HEIGHT {
            return;
        }
        let idx = y * SCREEN_WIDTH + x;

        let mask = self.registers.mask;
        let fine_x = self.registers.vram.x;

        // Background pixel sample.
        let (mut bg_palette, mut bg_color) = self.bg_pipeline.sample_and_shift(fine_x);
        let bg_visible = mask.contains(Mask::SHOW_BACKGROUND)
            && (x >= 8 || mask.contains(Mask::SHOW_BACKGROUND_LEFT));
        if !bg_visible {
            bg_color = 0;
            bg_palette = 0;
        }

        // Sprite pixel sample.
        let mut sprite_pixel = self.sprite_pipeline.sample_and_shift();
        let sprite_visible =
            mask.contains(Mask::SHOW_SPRITES) && (x >= 8 || mask.contains(Mask::SHOW_SPRITES_LEFT));
        if !sprite_visible {
            sprite_pixel.color = 0;
        }

        let sprite_opaque = sprite_pixel.color != 0;
        let bg_opaque = bg_color != 0;

        // Resolve priority between background and sprite.
        let mut final_palette = 0u8;
        let mut final_color = 0u8;
        let mut from_sprite = false;
        match (bg_opaque, sprite_opaque) {
            (false, false) => {}
            (false, true) => {
                final_palette = sprite_pixel.palette;
                final_color = sprite_pixel.color;
                from_sprite = true;
            }
            (true, false) => {
                final_palette = bg_palette;
                final_color = bg_color;
            }
            (true, true) => {
                if sprite_pixel.priority_behind_bg {
                    final_palette = bg_palette;
                    final_color = bg_color;
                } else {
                    final_palette = sprite_pixel.palette;
                    final_color = sprite_pixel.color;
                    from_sprite = true;
                }
            }
        }

        // Sprite 0 hit occurs when both layers are opaque and sprite 0 contributes.
        if bg_opaque
            && sprite_opaque
            && sprite_pixel.is_sprite0
            && sprite_visible
            && bg_visible
            && self.cycle != 256
        {
            self.registers.status.insert(Status::SPRITE_ZERO_HIT);
            if self.sprite0_hit_pos.is_none() {
                let oam0 = [
                    self.registers.oam[0],
                    self.registers.oam[1],
                    self.registers.oam[2],
                    self.registers.oam[3],
                ];
                self.sprite0_hit_pos = Some(Sprite0HitDebug {
                    pos: Sprite0HitPos {
                        scanline: self.scanline,
                        cycle: self.cycle,
                    },
                    oam: oam0,
                });
                #[cfg(debug_assertions)]
                eprintln!(
                    "[PPU][sprite0] hit at s={}, c={} oam0=[y:{:02X} tile:{:02X} attr:{:02X} x:{:02X}]",
                    self.scanline, self.cycle, oam0[0], oam0[1], oam0[2], oam0[3]
                );
            }
        }

        // Resolve palette RAM address (color 0 always uses universal background).
        let palette_addr = if final_color == 0 {
            ppu_mem::PALETTE_BASE
        } else if from_sprite {
            ppu_mem::PALETTE_BASE + 0x10 + (final_palette as u16) * 4 + (final_color as u16)
        } else {
            ppu_mem::PALETTE_BASE + (final_palette as u16) * 4 + (final_color as u16)
        };
        let color_index = self.palette_ram.read(palette_addr);
        self.framebuffer[idx] = color_index;
    }

    /// Performs background tile/attribute fetches and reloads shifters every 8 dots.
    ///
    /// Mirrors the PPU’s interleaved nametable/attribute/pattern fetch pattern
    /// on prerender and visible scanlines.
    fn fetch_background_data(&mut self, pattern: &mut PatternBus<'_>) {
        // Only run while background rendering is enabled.
        if !self.registers.mask.contains(Mask::SHOW_BACKGROUND) {
            return;
        }
        // Run during visible scanlines and prerender, when the background fetch pipeline is active.
        if !((0..=239).contains(&self.scanline) || self.scanline == -1) {
            return;
        }

        // Fetch and reload shifters every 8 dots (tile boundary) during fetch windows.
        let in_fetch_window = (1..=256).contains(&self.cycle) || (321..=336).contains(&self.cycle);
        if in_fetch_window && (self.cycle - 1) % 8 == 0 {
            self.load_background_tile(pattern);
            self.increment_scroll_x();
        }
    }

    /// Sprite pipeline tick for dots 1..=256.
    ///
    /// Dots 1..=64 clear secondary OAM; dots 65..=256 scan primary OAM to
    /// select up to 8 sprites for the *next* scanline.
    fn sprite_pipeline_eval_tick(&mut self) {
        if !self.registers.mask.contains(Mask::SHOW_SPRITES) {
            return;
        }

        match self.cycle {
            // Dots 1..=64: clear secondary OAM (32 bytes, 2 dots per byte).
            1..=64 => {
                if self.cycle % 2 == 1 {
                    let byte_index = ((self.cycle - 1) / 2) as usize;
                    if byte_index < 32 {
                        self.secondary_oam[byte_index] = 0xFF;
                    }
                }
            }

            // Dots 65..=256: sprite evaluation for next scanline.
            65..=256 => {
                self.evaluate_sprites_for_dot();
            }

            _ => {}
        }
    }

    /// Sprite pipeline tick for dots 257..=320.
    ///
    /// Fetches sprite metadata/pattern bytes for the next scanline. Each sprite
    /// gets an 8-dot slot.
    fn sprite_pipeline_fetch_tick(&mut self, pattern: &mut PatternBus<'_>) {
        if !self.registers.mask.contains(Mask::SHOW_SPRITES) {
            return;
        }

        if (257..=320).contains(&self.cycle) {
            self.fetch_sprites_for_dot(pattern);
        }
    }

    /// Per-dot sprite evaluation step (65..=256).
    fn evaluate_sprites_for_dot(&mut self) {
        // Dot 65 is the first evaluation dot: reset per-scanline latches/state.
        if self.cycle == 65 {
            self.sprite_eval = SpriteEvalState::default();
            self.sprite_eval.phase = SpriteEvalPhase::ScanY;
        }

        // Hardware evaluation runs at ~2 dots per byte. Only advance state on the second dot.
        if ((self.cycle - 65) & 1) == 0 {
            return;
        }

        if !(65..=256).contains(&self.cycle) {
            return;
        }

        if self.sprite_eval.n >= 64 {
            return;
        }

        let next_scanline = self.scanline.wrapping_add(1) as i16;
        let sprite_height: i16 = if self.registers.control.contains(Control::SPRITE_SIZE_16) {
            16
        } else {
            8
        };

        let base = (self.sprite_eval.n as usize) * 4;
        let y = self.registers.oam[base] as i16;

        match self.sprite_eval.phase {
            SpriteEvalPhase::ScanY => {
                // Read byte 0 (Y) and test range.
                let in_range = next_scanline >= y && next_scanline < y + sprite_height;

                if in_range {
                    if self.sprite_eval.count < 8 {
                        // Start copying this sprite into secondary OAM.
                        self.sprite_eval.copying = true;
                        self.sprite_eval.phase = SpriteEvalPhase::CopyRest;

                        // Copy Y.
                        if self.sprite_eval.sec_idx < 32 {
                            self.secondary_oam[self.sprite_eval.sec_idx as usize] =
                                self.registers.oam[base];
                            self.sprite_eval.sec_idx += 1;
                        }

                        if self.sprite_eval.n == 0 {
                            self.sprite_eval.sprite0_in_range_next = true;
                        }

                        self.sprite_eval.m = 1; // next byte to copy
                    } else {
                        // Enter overflow scan phase after 8 sprites.
                        self.sprite_eval.phase = SpriteEvalPhase::OverflowScan;
                        self.sprite_eval.m = 0;
                        // In real HW, overflow is set only if another in-range sprite is found later.
                    }
                } else {
                    // Not in range; advance to next sprite.
                    self.sprite_eval.n = self.sprite_eval.n.wrapping_add(1);
                    self.sprite_eval.m = 0;
                }
            }

            SpriteEvalPhase::CopyRest => {
                // Copy bytes 1..=3 (tile, attr, x).
                if self.sprite_eval.copying {
                    if self.sprite_eval.sec_idx < 32 {
                        let byte = self.registers.oam[base + self.sprite_eval.m as usize];
                        self.secondary_oam[self.sprite_eval.sec_idx as usize] = byte;
                        self.sprite_eval.sec_idx += 1;
                    }

                    self.sprite_eval.m += 1;

                    if self.sprite_eval.m >= 4 {
                        // Finished copying one sprite.
                        self.sprite_eval.copying = false;
                        self.sprite_eval.m = 0;
                        self.sprite_eval.count += 1;
                        self.sprite_eval.n = self.sprite_eval.n.wrapping_add(1);
                        self.sprite_eval.phase = SpriteEvalPhase::ScanY;
                    }
                } else {
                    // Safety fallback.
                    self.sprite_eval.phase = SpriteEvalPhase::ScanY;
                    self.sprite_eval.m = 0;
                    self.sprite_eval.n = self.sprite_eval.n.wrapping_add(1);
                }
            }

            SpriteEvalPhase::OverflowScan => {
                // TODO: HW overflow bug is approximated; emulate secondary OAM write suppression pattern for full accuracy.
                // Buggy overflow scan approximation:
                // Hardware increments m every dot, and only checks Y when m==0.
                if self.sprite_eval.m == 0 {
                    let in_range = next_scanline >= y && next_scanline < y + sprite_height;
                    if in_range {
                        self.sprite_eval.overflow_next = true;
                        self.registers.status.insert(Status::SPRITE_OVERFLOW);
                    }
                }

                // Advance m with the hardware's buggy pattern.
                self.sprite_eval.m = (self.sprite_eval.m + 1) & 0b11;
                if self.sprite_eval.m == 0 {
                    self.sprite_eval.n = self.sprite_eval.n.wrapping_add(1);
                }
            }
        }
    }

    /// Per-dot sprite fetch step (257..=320).
    fn fetch_sprites_for_dot(&mut self, pattern: &mut PatternBus<'_>) {
        // First fetch dot for the window is cycle 258 (clock arm starts at 258).
        if self.cycle == 258 {
            self.sprite_fetch = SpriteFetchState::default();
        }

        if !(258..=320).contains(&self.cycle) {
            return;
        }

        let i = self.sprite_fetch.i as usize;
        if i >= 8 {
            return;
        }

        // Each sprite gets 8 dots in this region.
        // sub = 0..7 within a sprite slot.
        let sub = self.sprite_fetch.sub;

        // Read sprite bytes from secondary OAM.
        let base = i * 4;
        let y = self.secondary_oam[base];
        let tile = self.secondary_oam[base + 1];
        let attr = self.secondary_oam[base + 2];
        let x = self.secondary_oam[base + 3];

        // Latch raw sprite bytes early in the slot.
        if sub == 0 {
            self.sprite_line_next.set_meta(i, y, tile, attr, x);
        }

        // Compute which row of the sprite to fetch for the next scanline.
        let next_scanline = self.scanline.wrapping_add(1) as i16;
        let sprite_height: i16 = if (self.registers.control.bits() & 0x20) != 0 {
            16
        } else {
            8
        }; // PPUCTRL bit 5
        let mut row = next_scanline - (y as i16);
        if row < 0 {
            row = 0;
        }

        // Vertical flip affects row selection.
        let flip_v = (attr & 0x80) != 0;
        if flip_v {
            row = (sprite_height - 1) - row;
        }

        // Determine pattern table base and tile index for 8x8 vs 8x16.
        let (pattern_base, tile_index) = if sprite_height == 16 {
            // For 8x16, bit 0 selects table, and tile index is even.
            let base = if (tile & 0x01) != 0 {
                ppu_mem::PATTERN_TABLE_1
            } else {
                ppu_mem::PATTERN_TABLE_0
            };
            let top_tile = tile & 0xFE;
            let tile_idx = if row < 8 {
                top_tile
            } else {
                top_tile.wrapping_add(1)
            };
            let r = (row & 7) as u16;
            (base, (tile_idx, r))
        } else {
            // For 8x8, PPUCTRL bit 3 selects sprite pattern table.
            let base = if (self.registers.control.bits() & 0x08) != 0 {
                ppu_mem::PATTERN_TABLE_1
            } else {
                ppu_mem::PATTERN_TABLE_0
            };
            let r = (row & 7) as u16;
            (base, (tile, r))
        };

        let (tile_idx, fine_y) = tile_index;
        let addr = pattern_base + (tile_idx as u16) * 16 + fine_y;

        // sub 4/6 are the low/high plane fetches in a classic 8-dot slot.
        if sub == 4 {
            let pattern_low = self.read_vram(pattern, addr);
            self.sprite_line_next.set_pattern_low(i, pattern_low);
        }
        if sub == 6 {
            let pattern_high = self.read_vram(pattern, addr + 8);
            self.sprite_line_next.set_pattern_high(i, pattern_high);
        }

        // Advance within the 8-dot sprite slot.
        self.sprite_fetch.sub += 1;
        if self.sprite_fetch.sub >= 8 {
            self.sprite_fetch.sub = 0;
            self.sprite_fetch.i += 1;
        }
    }

    fn read_status(&mut self) -> u8 {
        let prev_output = self.nmi_output;
        let status = self.registers.status.bits();
        self.registers.status.remove(Status::VERTICAL_BLANK);
        self.registers.vram.reset_latch();
        self.update_nmi_output(prev_output);
        status
    }

    fn write_oam_data(&mut self, value: u8) {
        let idx = self.registers.oam_addr as usize;
        if idx < ppu_mem::OAM_RAM_SIZE {
            self.registers.oam[idx] = value;
            self.registers.oam_addr = self.registers.oam_addr.wrapping_add(1);
        }
    }

    fn read_oam_data(&self) -> u8 {
        let rendering = self.registers.mask.rendering_enabled();
        let during_render =
            rendering && ((0..=239).contains(&self.scanline) || self.scanline == -1);

        if during_render {
            // During rendering, reads return the contents of the internal (stale) OAM bus.
            // Approximated here as 0xFF to avoid exposing primary OAM.
            0xFF
        } else {
            let idx = self.registers.oam_addr as usize;
            if idx < ppu_mem::OAM_RAM_SIZE {
                self.registers.oam[idx]
            } else {
                0
            }
        }
    }

    fn write_vram_data(&mut self, value: u8, pattern: &mut PatternBus<'_>) {
        let addr = self.registers.vram.v.raw() & ppu_mem::VRAM_MIRROR_MASK;
        self.write_vram(pattern, addr, value);
        let increment = self.registers.control.vram_increment();
        self.registers.vram.v.increment(increment);
    }

    fn read_vram_data(&mut self, pattern: &mut PatternBus<'_>) -> u8 {
        let addr = self.registers.vram.v.raw() & ppu_mem::VRAM_MIRROR_MASK;
        let data = self.read_vram(pattern, addr);
        let buffered = self.registers.vram_buffer;
        self.registers.vram_buffer = data;
        let increment = self.registers.control.vram_increment();
        self.registers.vram.v.increment(increment);

        if addr >= ppu_mem::PALETTE_BASE {
            data
        } else {
            buffered
        }
    }

    fn write_vram(&mut self, pattern: &mut PatternBus<'_>, addr: u16, value: u8) {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;
        let addr = self.mirror_vram_addr(addr, pattern);
        if addr >= ppu_mem::PALETTE_BASE {
            self.palette_ram.write(addr, value);
        } else if addr < 0x2000 {
            if !pattern.write(addr, value) {
                self.vram[addr as usize] = value;
            }
        } else {
            self.vram[addr as usize] = value;
        }
    }

    fn read_vram(&mut self, pattern: &mut PatternBus<'_>, addr: u16) -> u8 {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;
        let addr = self.mirror_vram_addr(addr, pattern);
        if addr >= ppu_mem::PALETTE_BASE {
            self.palette_ram.read(addr)
        } else if addr < 0x2000 {
            pattern
                .read(addr)
                .unwrap_or_else(|| self.vram[addr as usize])
        } else {
            self.vram[addr as usize]
        }
    }

    /// Applies nametable mirroring rules for addresses in `$2000-$3EFF`.
    fn mirror_vram_addr(&self, addr: u16, pattern: &PatternBus<'_>) -> u16 {
        if addr < ppu_mem::NAMETABLE_BASE || addr >= ppu_mem::PALETTE_BASE {
            return addr;
        }

        // $3000-$3EFF mirrors $2000-$2EFF.
        let mirrored = if addr >= 0x3000 { addr - 0x1000 } else { addr };
        let relative = mirrored - ppu_mem::NAMETABLE_BASE;
        let table = ((relative / ppu_mem::NAMETABLE_SIZE) & 0b11) as u8;
        let offset = relative % ppu_mem::NAMETABLE_SIZE;

        let target_table = match pattern.mirroring() {
            // Vertical mirroring: $2000/$2800 share, $2400/$2C00 share (table & 1).
            Mirroring::Vertical => table & 0b01,
            // Horizontal mirroring: $2000/$2400 share, $2800/$2C00 share (table >> 1).
            Mirroring::Horizontal => (table >> 1) & 0b01,
            Mirroring::FourScreen => table,
            Mirroring::SingleScreenLower => 0,
            Mirroring::SingleScreenUpper => 1,
        };

        ppu_mem::NAMETABLE_BASE + (target_table as u16 * ppu_mem::NAMETABLE_SIZE) + offset
    }

    /// Loads the current tile/attribute data into the background shifters.
    fn load_background_tile(&mut self, pattern: &mut PatternBus<'_>) {
        let v = self.registers.vram.v;
        let base_nt = ppu_mem::NAMETABLE_BASE + (v.nametable() as u16 * ppu_mem::NAMETABLE_SIZE);
        let tile_index_addr = base_nt + (v.coarse_y() as u16 * 32) + (v.coarse_x() as u16);
        let tile_index = self.read_vram(pattern, tile_index_addr);

        let fine_y = v.fine_y() as u16;
        let pattern_base = if self.registers.control.contains(Control::BACKGROUND_TABLE) {
            ppu_mem::PATTERN_TABLE_1
        } else {
            ppu_mem::PATTERN_TABLE_0
        };
        let pattern_addr = pattern_base + (tile_index as u16 * 16) + fine_y;
        let tile_pattern = [
            self.read_vram(pattern, pattern_addr),
            self.read_vram(pattern, pattern_addr + 8),
        ];

        let attr_addr =
            base_nt + 0x03C0 + (v.coarse_y() as u16 / 4) * 8 + (v.coarse_x() as u16 / 4);
        let attr_byte = self.read_vram(pattern, attr_addr);
        let quadrant_shift = ((v.coarse_y() & 0b10) << 1) | (v.coarse_x() & 0b10);
        let palette_index = (attr_byte >> quadrant_shift) & 0b11;

        self.bg_pipeline.reload(tile_pattern, palette_index);
    }

    /// Increments the coarse X scroll component in `v`, wrapping nametable horizontally.
    fn increment_scroll_x(&mut self) {
        let cx = self.registers.vram.v.coarse_x();
        if cx == 31 {
            self.registers.vram.v.set_coarse_x(0);
            let nt = self.registers.vram.v.nametable() ^ 0b01;
            self.registers.vram.v.set_nametable(nt);
        } else {
            self.registers.vram.v.set_coarse_x(cx + 1);
        }
    }

    /// Increments the vertical scroll components in `v`, including fine Y.
    fn increment_scroll_y(&mut self) {
        let fine_y = self.registers.vram.v.fine_y();
        if fine_y < 7 {
            self.registers.vram.v.set_fine_y(fine_y + 1);
            return;
        }

        // Fine Y rolls over, advance coarse Y.
        self.registers.vram.v.set_fine_y(0);
        let mut coarse_y = self.registers.vram.v.coarse_y();
        if coarse_y == 29 {
            coarse_y = 0;
            let nt = self.registers.vram.v.nametable() ^ 0b10;
            self.registers.vram.v.set_nametable(nt);
        } else if coarse_y == 31 {
            // Skip attribute memory gap lines.
            coarse_y = 0;
        } else {
            coarse_y = coarse_y.wrapping_add(1);
        }
        self.registers.vram.v.set_coarse_y(coarse_y);
    }

    /// Copies horizontal scroll bits from `t` into `v` (coarse X + nametable X).
    fn copy_horizontal_scroll(&mut self) {
        let t = self.registers.vram.t;
        self.registers.vram.v.set_coarse_x(t.coarse_x());
        let nt = self.registers.vram.v.nametable();
        self.registers
            .vram
            .v
            .set_nametable((nt & 0b10) | (t.nametable() & 0b01));
    }

    /// Copies vertical scroll bits from `t` into `v` (fine Y + coarse Y + nametable Y).
    fn copy_vertical_scroll(&mut self) {
        let t = self.registers.vram.t;
        self.registers.vram.v.set_fine_y(t.fine_y());
        self.registers.vram.v.set_coarse_y(t.coarse_y());
        let nt = self.registers.vram.v.nametable();
        self.registers
            .vram
            .v
            .set_nametable((nt & 0b01) | (t.nametable() & 0b10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_register_helpers() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        ppu.cpu_write(PpuRegister::Control.addr(), 0b1000_0100, &mut pattern);
        assert!(ppu.registers.control.nmi_enabled());
        assert_eq!(ppu.registers.control.vram_increment(), 32);
        assert_eq!(
            ppu.registers.control.base_nametable_addr(),
            ppu_mem::NAMETABLE_BASE
        );
    }

    #[test]
    fn buffered_ppu_data_read() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        // Point to $2000 and write a value.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20, &mut pattern);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut pattern);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x12, &mut pattern);

        // Reset VRAM address to read back.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20, &mut pattern);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut pattern);

        let first = ppu.cpu_read(PpuRegister::Data.addr(), &mut pattern);
        let second = ppu.cpu_read(PpuRegister::Data.addr(), &mut pattern);
        assert_eq!(first, 0x00, "First read should return buffered value");
        assert_eq!(second, 0x12, "Second read should contain VRAM data");
    }

    #[test]
    fn palette_reads_bypass_buffer() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F, &mut pattern);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut pattern);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x99, &mut pattern);

        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F, &mut pattern);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut pattern);

        let value = ppu.cpu_read(PpuRegister::Data.addr(), &mut pattern);
        assert_eq!(value, 0x99);
    }

    #[test]
    fn status_read_resets_scroll_latch() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x12, &mut pattern); // horizontal
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x34, &mut pattern); // vertical
        assert_eq!(ppu.registers.vram.t.coarse_x(), 0x12 >> 3);
        assert_eq!(ppu.registers.vram.x, 0x12 & 0x07);
        assert_eq!(ppu.registers.vram.t.coarse_y(), 0x34 >> 3);
        assert_eq!(ppu.registers.vram.t.fine_y(), 0x34 & 0x07);

        // Reading status should clear the write toggle so the next write targets horizontal.
        let _ = ppu.cpu_read(PpuRegister::Status.addr(), &mut pattern);
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x56, &mut pattern);
        assert_eq!(ppu.registers.vram.t.coarse_x(), 0x56 >> 3);
        assert_eq!(ppu.registers.vram.t.coarse_y(), 0x34 >> 3);
    }

    #[test]
    fn oam_data_auto_increments() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        ppu.cpu_write(PpuRegister::OamAddr.addr(), 0x02, &mut pattern);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xAA, &mut pattern);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xBB, &mut pattern);
        assert_eq!(ppu.registers.oam[2], 0xAA);
        assert_eq!(ppu.registers.oam[3], 0xBB);
    }

    #[test]
    fn vblank_flag_is_managed_by_clock() {
        let mut ppu = Ppu::new();
        let mut pattern = PatternBus::default();
        // Run until scanline 241, cycle 1 (accounting for prerender line).
        let target_cycles = (242i32 * CYCLES_PER_SCANLINE as i32 + 2) as usize;
        for _ in 0..target_cycles {
            ppu.clock(&mut pattern);
        }
        assert!(ppu.registers.status.contains(Status::VERTICAL_BLANK));

        // Continue to the prerender line, then run dot 1 where VBL is cleared.
        while !(ppu.scanline == -1 && ppu.cycle == 1) {
            ppu.clock(&mut pattern);
        }
        // Dot 1 of prerender clears VBL/sprite flags (mirrors hardware timing).
        ppu.clock(&mut pattern);
        assert!(!ppu.registers.status.contains(Status::VERTICAL_BLANK));
    }
}
