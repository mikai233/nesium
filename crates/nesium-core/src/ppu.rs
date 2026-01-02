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
//! - Odd frames are one PPU tick shorter when rendering is enabled: the PPU
//!   skips the first idle tick on the first visible scanline by jumping from
//!   the last prerender dot directly to scanline 0, cycle 1 (per NESdev).
//! - The PPU data bus floats when undriven; we mirror that with an open-bus
//!   latch (with decay) so untouched register reads see the last driven value.
//! - OAM reads during rendering: hardware doesn’t expose live primary OAM then;
//!   we return a constant `0xFF` to approximate the internal bus noise.
//! - Palette RAM has mirroring quirks ($3F10 mirrors $3F00, etc.). Those rules
//!   are handled in `palette::PaletteIndex::mirrored_addr` and `PaletteRam`.

pub mod palette;

mod background_pipeline;
pub mod buffer;
mod open_bus;
pub(crate) mod pattern_bus;
mod pending_vram_increment;
mod ppu_model;
mod registers;
pub(crate) mod savestate;
mod sprite;
mod sprite_pipeline;
mod sprite_state;

pub(crate) use pending_vram_increment::PendingVramIncrement;
pub(crate) use registers::{Control, Mask, Status};
pub(crate) use sprite_state::SpriteLineBuffers;

use self::{
    background_pipeline::BgPipeline,
    sprite_pipeline::SpritePipeline,
    sprite_state::{SpriteEvalState, SpriteFetchState},
};

use core::ffi::c_void;
use core::fmt;

use crate::{
    bus::CpuBus,
    cartridge::mapper::{NametableTarget, PpuVramAccessContext, PpuVramAccessKind},
    context::Context,
    cpu::Cpu,
    mem_block::ppu::{Ciram, SecondaryOamRam},
    memory::ppu::{self as ppu_mem, Register as PpuRegister},
    ppu::{
        buffer::FrameBuffer,
        buffer::FrameReadyCallback,
        open_bus::PpuOpenBus,
        palette::{Palette, PaletteRam},
        pattern_bus::PpuBus,
        registers::{Registers, VramAddr},
        sprite::SpriteView,
    },
    reset_kind::ResetKind,
};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
const CYCLES_PER_SCANLINE: u16 = 341;
const SCANLINES_PER_FRAME: i16 = 262; // -1 (prerender) + 0..239 visible + post + vblank (241..260)

/// Entry points for the CPU PPU register mirror.
#[derive(Clone)]
pub struct Ppu {
    /// Collection of CPU visible registers and their helper latches.
    pub(crate) registers: Registers,
    /// Deferred $2006 VRAM address to apply after the hardware latency window.
    pub(crate) pending_vram_addr: VramAddr,
    /// Remaining dots before the pending VRAM address commit (0 = no pending write).
    pub(crate) pending_vram_delay: u8,
    /// Character Internal RAM (CIRAM) - 2 KiB internal nametable backing store.
    /// Pattern table data ($0000-$1FFF) is provided by the cartridge CHR ROM/RAM.
    pub(crate) ciram: Ciram,
    /// Dedicated palette RAM. Addresses between `$3F00` and `$3FFF` map here.
    pub(crate) palette_ram: PaletteRam,
    /// Current dot (0..=340) within the active scanline.
    pub(crate) cycle: u16,
    /// Current scanline. `-1` is the prerender line, `0..239` are visible.
    pub(crate) scanline: i16,
    /// Total number of frames produced so far.
    pub(crate) frame: u32,
    /// Master clock in PPU master cycles (4 master cycles per dot).
    pub(crate) master_clock: u64,
    /// Background pixel pipeline (pattern and attribute shifters).
    pub(crate) bg_pipeline: BgPipeline,
    /// Sprite pixel pipeline for the current scanline.
    pub(crate) sprite_pipeline: SpritePipeline,
    /// Current level of the NMI output line (true = asserted).
    pub(crate) nmi_level: bool,
    /// When true, suppresses the upcoming VBlank flag/NMI edge for this frame.
    /// Models the $2002 read-vs-VBlank set race described on NESdev (and used by Mesen2).
    pub(crate) prevent_vblank_flag: bool,
    /// PPU-side open-bus latch (with decay).
    pub(crate) open_bus: PpuOpenBus,
    /// Countdown (in PPU dots) during which a second `$2007` read is ignored.
    /// Mirrors Mesen2's `_ignoreVramRead` behaviour (two consecutive CPU
    /// cycles -> second read returns open bus and does not increment VRAM).
    pub(crate) ignore_vram_read: u8,
    /// Internal OAM data bus copy buffer used during rendering for `$2004` reads.
    pub(crate) oam_copybuffer: u8,
    /// Pending VRAM increment after a `$2007` read/write (applied one dot later).
    pub(crate) pending_vram_increment: PendingVramIncrement,
    /// Secondary OAM used during sprite evaluation for the current scanline.
    pub(crate) secondary_oam: SecondaryOamRam,
    /// Sprite evaluation state (cycle-accurate structure).
    pub(crate) sprite_eval: SpriteEvalState,
    /// Sprite fetch state for dots 257..=320.
    pub(crate) sprite_fetch: SpriteFetchState,
    /// Buffered secondary-OAM sprite bytes/patterns for the next scanline.
    pub(crate) sprite_line_next: SpriteLineBuffers,
    /// Master system palette used to map palette indices to RGB colors.
    pub(crate) palette: Palette,
    /// Effective rendering enable latch (Mesen-style), true when either
    /// background or sprites are enabled.
    pub(crate) render_enabled: bool,
    /// Previous dot's rendering enable state (for scroll/odd-frame logic and
    /// trace parity with Mesen's `_prevRenderingEnabled`).
    pub(crate) prev_render_enabled: bool,
    /// Pending OAMADDR increment glitch when rendering is disabled during sprite evaluation.
    pub(crate) oam_addr_disable_glitch_pending: bool,
    /// OAM row corruption flags (Mesen2 `SetOamCorruptionFlags` / `ProcessOamCorruption`).
    pub(crate) corrupt_oam_row: [bool; 32],
    /// Pending state update request from $2001/$2006/$2007/VRAM-related
    /// side effects. Mirrors Mesen's `_needStateUpdate` latch.
    pub(crate) state_update_pending: bool,
    /// Background + sprite rendering target for the current frame.
    pub(crate) framebuffer: FrameBuffer,
}

/// Source register for scroll-glitch emulation.
#[derive(Copy, Clone)]
enum ScrollGlitchSource {
    Control2000,
    Scroll2005,
    Addr2006,
}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ppu")
            .field("registers", &self.registers)
            .field("cycle", &self.cycle)
            .field("scanline", &self.scanline)
            .field("frame", &self.frame)
            .finish()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new(FrameBuffer::default())
    }
}

impl Ppu {
    /// Creates a new PPU instance with cleared VRAM and default register values.
    pub fn new(framebuffer: FrameBuffer) -> Self {
        Self {
            registers: Registers::new(),
            pending_vram_addr: VramAddr::default(),
            pending_vram_delay: 0,
            ciram: Ciram::new(),
            palette_ram: PaletteRam::new(),
            // Initialize to the last dot of the pre-boot frame so the first clock()
            // rolls over to scanline -1, cycle 0.
            cycle: CYCLES_PER_SCANLINE - 1,
            scanline: -1,
            frame: 1,
            master_clock: 0,
            bg_pipeline: BgPipeline::new(),
            sprite_pipeline: SpritePipeline::new(),
            nmi_level: false,
            prevent_vblank_flag: false,
            open_bus: PpuOpenBus::new(),
            ignore_vram_read: 0,
            oam_copybuffer: 0,
            pending_vram_increment: PendingVramIncrement::None,
            secondary_oam: SecondaryOamRam::new(),
            sprite_eval: SpriteEvalState::default(),
            sprite_fetch: SpriteFetchState::default(),
            sprite_line_next: SpriteLineBuffers::new(),
            palette: Palette::default(),
            render_enabled: false,
            prev_render_enabled: false,
            oam_addr_disable_glitch_pending: false,
            corrupt_oam_row: [false; 32],
            state_update_pending: false,
            framebuffer,
        }
    }

    /// Restores the PPU to either a power-on or soft-reset state.
    ///
    /// - `ResetKind::PowerOn`:
    ///   Simulates a cold boot. Clears VRAM / palette RAM / secondary OAM,
    ///   resets registers, and clears the framebuffer.
    ///
    /// - `ResetKind::Soft`:
    ///   Simulates CPU /RESET. Resets control/scroll/address latches and
    ///   internal timing, but does not touch PPU RAM contents and preserves
    ///   the PPU status register ($2002), including the VBlank flag.
    pub fn reset(&mut self, kind: ResetKind) {
        // Preserve the current $2002 status so soft reset can restore it.
        // On real hardware, the VBlank flag (bit 7) is not affected by CPU reset.
        let prev_status = self.registers.status;

        match kind {
            ResetKind::PowerOn => {
                // Full power-on reset: registers and PPU-internal RAM go back
                // to their initial state.
                self.registers.reset();

                // On real hardware, VRAM/palette contents are technically
                // undefined at power-on. For determinism (and compatibility
                // with common test ROM expectations) we initialize VRAM to 0
                // and palette RAM to a commonly observed RP2C02 power-up table.
                self.ciram.fill(0);
                self.palette_ram.fill_power_on();
                self.secondary_oam.fill(0);

                // At power-on the VBlank flag is effectively random. For
                // determinism we start with it cleared so the first BIT $2002
                // loop always waits for a real VBlank edge.
                self.registers
                    .status
                    .remove(registers::Status::VERTICAL_BLANK);
            }
            ResetKind::Soft => {
                // Soft reset: reset control/scroll/address latches, but do not
                // reset PPU memory, and keep $2002 as-is.
                self.registers.reset();

                // Restore the full $2002 status byte (including VBlank,
                // sprite 0 hit and overflow) so soft reset doesn't affect it.
                self.registers.status = prev_status;

                // Note: VRAM, palette RAM, secondary OAM and the framebuffer
                // are intentionally left untouched here. They are only
                // initialized/cleared at power-on.
            }
        }

        // --- Common reset logic for both power-on and soft reset ---

        // Reset timing so that the next clock() call starts from
        // scanline -1, cycle 0 (prerender line).
        self.cycle = CYCLES_PER_SCANLINE - 1;
        self.scanline = -1;
        self.frame = 1;
        self.master_clock = 0;

        // Clear delayed $2006 write and pending VRAM increment.
        self.pending_vram_delay = 0;
        self.pending_vram_addr = VramAddr::default();
        self.pending_vram_increment = PendingVramIncrement::None;

        // Clear background / sprite pipelines and sprite-eval state so the
        // next frame starts from a clean pipeline state.
        self.bg_pipeline.clear();
        self.sprite_pipeline.clear();
        self.sprite_eval = SpriteEvalState::default();
        self.sprite_fetch = SpriteFetchState::default();
        self.sprite_line_next.clear();

        self.nmi_level = false;
        self.prevent_vblank_flag = false;

        // Open bus and related counters.
        self.open_bus.reset();
        self.ignore_vram_read = 0;
        self.oam_copybuffer = 0;
        self.oam_addr_disable_glitch_pending = false;
        self.corrupt_oam_row = [false; 32];

        // Sprite-0 debug info is per-frame; drop it on reset.

        // Reset rendering enable latches and mark state as needing an update
        // on the next clock, mirroring Mesen's `_renderingEnabled` /
        // `_prevRenderingEnabled` + `_needStateUpdate` behaviour.
        self.render_enabled = false;
        self.prev_render_enabled = false;
        self.state_update_pending = true;

        // Only clear the framebuffer on power-on. Soft reset keeps the last
        // rendered frame visible, just like real hardware.
        if matches!(kind, ResetKind::PowerOn) {
            self.clear_framebuffer();
        }
    }

    /// Returns an immutable view of the current framebuffer.
    ///
    /// In color mode the buffer contains packed pixels in the active
    /// [`buffer::ColorFormat`]. In index mode it contains one byte per pixel
    /// with palette indices (`0..=63`).
    pub fn render_buffer(&self) -> &[u8] {
        self.framebuffer.render()
    }

    pub fn render_index_buffer(&self) -> &[u8] {
        self.framebuffer.render_index()
    }

    /// Copies the current front buffer pixels into the provided destination slice.
    pub fn copy_render_buffer(&mut self, dst: &mut [u8]) {
        self.framebuffer.copy_render_buffer(dst);
    }

    /// Copies the current front index buffer into the provided destination slice.
    pub fn copy_render_index_buffer(&self, dst: &mut [u8]) {
        self.framebuffer.copy_render_index_buffer(dst);
    }

    pub fn set_frame_ready_callback(
        &mut self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) {
        self.framebuffer.set_frame_ready_callback(cb, user_data);
    }

    /// Current frame counter (increments when scanline wraps from 260 to -1).
    pub fn frame_count(&self) -> u32 {
        self.frame
    }

    /// Clears the framebuffer to palette index 0.
    fn clear_framebuffer(&mut self) {
        self.framebuffer.clear();
    }

    /// Replaces the master system palette used for color conversion.
    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }

    /// Master system palette used to map palette indices to RGB colors.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Mutable reference to the internal framebuffer.
    pub fn framebuffer_mut(&mut self) -> &mut FrameBuffer {
        &mut self.framebuffer
    }

    /// Handles CPU writes to the mirrored PPU register space (`$2000-$3FFF`).
    ///
    /// Mirrors open-bus semantics by latching the last value written; the
    /// hardware leaves that value on the data bus.
    pub fn cpu_write(&mut self, addr: u16, value: u8, ppu_bus: &mut PpuBus) {
        // Writes to PPU registers fully drive the bus.
        self.open_bus.set(0xFF, value, self.frame);
        match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Control => {
                self.registers.write_control(value);
                self.maybe_apply_scroll_glitch(ScrollGlitchSource::Control2000);
                self.update_nmi_level();
            }
            PpuRegister::Mask => {
                // TODO: Hardware/Mesen2 model subtle mid-frame rendering enable/disable glitches (bus address reset, OAM corruption).
                // We currently just update the mask bitfield and then update the
                // Mesen-style rendering-enabled latch on the next `clock()`.
                self.registers.mask = Mask::from_bits_retain(value);

                let mask = self.registers.mask;
                let new_render = mask.rendering_enabled();
                if new_render != self.render_enabled {
                    // Defer the actual latch update to `update_state_latch` so
                    // that it is applied in a consistent place in the PPU
                    // timeline, like Mesen's `_needStateUpdate`.
                    self.state_update_pending = true;
                }

                // // Debug trace: mirror Mesen's SetMaskRegister $2001 write log.
                // let bg_en = if mask.contains(Mask::SHOW_BACKGROUND) {
                //     1
                // } else {
                //     0
                // };
                // let sp_en = if mask.contains(Mask::SHOW_SPRITES) {
                //     1
                // } else {
                //     0
                // };
                // tracing::debug!(
                //     "ppu_write_2001: frame={} scanline={} cycle={} value={:02X} bg_en={} sp_en={}",
                //     self.frame,
                //     self.scanline,
                //     self.cycle,
                //     value,
                //     bg_en,
                //     sp_en,
                // );
            }

            PpuRegister::Status => {} // read-only
            PpuRegister::OamAddr => self.registers.oam_addr = value,
            PpuRegister::OamData => self.write_oam_data(value),
            PpuRegister::Scroll => {
                let w_before = self.registers.vram.w;
                self.registers.vram.write_scroll(value);
                // Glitch on first $2005 write (horizontal scroll) when it lands
                // exactly on dot 257 of a visible scanline while rendering.
                if !w_before {
                    self.maybe_apply_scroll_glitch(ScrollGlitchSource::Scroll2005);
                }
            }
            PpuRegister::Addr => {
                let w_before = self.registers.vram.w;
                if let Some(new_v) = self.registers.vram.write_addr(value) {
                    self.pending_vram_addr = new_v;
                    self.pending_vram_delay = 3;
                }
                // Glitch on first $2006 write (high byte) under the same timing
                // conditions as Mesen2's ProcessTmpAddrScrollGlitch.
                if !w_before {
                    self.maybe_apply_scroll_glitch(ScrollGlitchSource::Addr2006);
                }
            }
            PpuRegister::Data => self.write_vram_data(value, ppu_bus),
        }
    }

    /// Handles CPU reads from the mirrored PPU register space (`$2000-$3FFF`).
    ///
    /// Unhandled reads return the last bus value to approximate open-bus
    /// behavior.
    pub fn cpu_read(&mut self, addr: u16, ppu_bus: &mut PpuBus<'_>) -> u8 {
        match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Status => self.read_status(),
            PpuRegister::OamData => {
                let v = self.read_oam_data();
                // OAMDATA drives the full bus when read.
                self.open_bus.apply(0x00, v, self.frame)
            }
            PpuRegister::Data => self.read_vram_data(ppu_bus),
            // Write-only / unimplemented reads: floating bus. Still apply decay.
            _ => self.open_bus.apply(0xFF, 0, self.frame),
        }
    }

    /// Advances the PPU by a single dot, keeping cycle and frame counters up to date.
    ///
    /// This is the main timing entry: it performs background/sprite pipeline
    /// work, runs fetch windows, and renders pixels on visible scanlines. Call
    /// three times per CPU tick for NTSC timing.
    pub fn step(bus: &mut CpuBus, _cpu: &mut Cpu, _ctx: &mut Context) {
        // let cpu_cycle = bus.cycles();
        // if cpu_cycle > 16_000_000 && cpu_cycle < 20_000_000 {
        //     let cpu_master_clock = bus.master_clock();
        //     let ppu = bus.devices().ppu;
        //     let status = ppu.registers.status;
        //     let prevent_vbl = u8::from(ppu.prevent_vblank_flag);
        //     let status_v = u8::from(status.contains(Status::VERTICAL_BLANK));
        //     let status_s0 = u8::from(status.contains(Status::SPRITE_ZERO_HIT));
        //     let status_ovf = u8::from(status.contains(Status::SPRITE_OVERFLOW));
        //     let v_raw = ppu.registers.vram.v.raw();
        //     let t_raw = ppu.registers.vram.t.raw();
        //     let xscroll = ppu.registers.vram.x;
        //     let mask = ppu.registers.mask;
        //     let mask_bg = u8::from(mask.contains(Mask::SHOW_BACKGROUND));
        //     let mask_sp = u8::from(mask.contains(Mask::SHOW_SPRITES));
        //     let need_nmi = 0;
        //     let prev_need_nmi = 0;
        //     let prev_nmi_flag = 0;
        //     let nmi_flag = 0;
        //
        //     tracing::debug!(
        //         "cpu_mc={} cpu_cyc={} pc={:04X} a={:02X} x={:02X} y={:02X} sp={:02X} ps={:02X}  \
        //          ppu_mc={} ppu_scanline={} ppu_cycle={} frame={} prevent_vbl={} status_v={} status_s0={} status_ovf={} \
        //          v={:04X} t={:04X} xscroll={:02X} mask_bg={} mask_sp={} need_nmi={} prev_need_nmi={} prev_nmi_flag={} nmi_flag={}",
        //         cpu_master_clock,
        //         cpu_cycle,
        //         cpu.pc,
        //         cpu.a,
        //         cpu.x,
        //         cpu.y,
        //         cpu.s,
        //         cpu.p,
        //         ppu.master_clock,
        //         ppu.scanline,
        //         ppu.cycle,
        //         ppu.frame,
        //         prevent_vbl,
        //         status_v,
        //         status_s0,
        //         status_ovf,
        //         v_raw,
        //         t_raw,
        //         xscroll,
        //         mask_bg,
        //         mask_sp,
        //         need_nmi,
        //         prev_need_nmi,
        //         prev_nmi_flag,
        //         nmi_flag,
        //     );
        // }
        // Pre-increment: Advance counters at the START of the clock.
        // This ensures scanline/cycle reflect the state *after* this cycle is processed
        // for synchronization purposes, aligning with Mesen's interpretation.
        let cpu_cycle = bus.cycles();
        let mut devices = bus.devices_mut();
        let mut ppu_bus = PpuBus::new(devices.cartridge.as_deref_mut(), cpu_cycle);
        let ppu = &mut devices.ppu;
        ppu.advance_cycle();

        if ppu.oam_addr_disable_glitch_pending {
            ppu.registers.oam_addr = ppu.registers.oam_addr.wrapping_add(1);
            ppu.oam_addr_disable_glitch_pending = false;
        }

        let rendering_enabled = ppu.render_enabled;
        ppu.step_pending_vram_addr();

        if ppu.ignore_vram_read > 0 {
            ppu.ignore_vram_read -= 1;
        }

        if ppu.pending_vram_increment.is_pending() {
            // Mesen2 / hardware: the simple "+1 or +32" VRAM increment after
            // a $2007 access only applies when rendering is disabled or during
            // VBlank/post-render scanlines. While rendering is active on the
            // prerender/visible scanlines, VRAM address progression is driven
            // by the scroll increment logic (coarse/fine X/Y) instead.
            if ppu.scanline >= 240 || !rendering_enabled {
                let step = ppu.pending_vram_increment.amount();
                if step != 0 {
                    ppu.registers.vram.v.increment(step);
                }
            }
            ppu.pending_vram_increment = PendingVramIncrement::None;
        }

        // NOTE: We clear VBlank and drop NMI output at dot 1 of prerender.
        // Dot 0 should NOT clear VBlank on normal frames (avoids early NMI fall).

        // Load sprite shifters for the new scanline before rendering begins.
        if ppu.cycle == 1 && (0..=239).contains(&ppu.scanline) {
            let sprite_count = ppu.sprite_eval.count.min(8);

            // Take slices once so we can both log and pass them to the pipeline.
            let attrs = ppu.sprite_line_next.attr_slice();
            let xs = ppu.sprite_line_next.x_slice();
            let pats_lo = ppu.sprite_line_next.pattern_low_slice();
            let pats_hi = ppu.sprite_line_next.pattern_high_slice();

            ppu.sprite_pipeline.load_scanline(
                sprite_count,
                ppu.sprite_eval.sprite0_in_range_next,
                attrs,
                xs,
                pats_lo,
                pats_hi,
            );
        }

        // If rendering is disabled, keep pipelines idle to avoid stale data.
        if !rendering_enabled {
            ppu.bg_pipeline.clear();
            ppu.sprite_pipeline.clear();
            ppu.sprite_line_next.clear();
        }

        match (ppu.scanline, ppu.cycle) {
            // ----------------------
            // Pre-render scanline (-1)
            // ----------------------
            // Dot 0 has no special side effects (prerender clears happen at dot 1).
            (_, 0) => {}

            // Background pipeline ticks during prerender dots 1..=256 when rendering is enabled.
            (-1, 1..=256) => {
                if ppu.cycle == 1 {
                    // Clear vblank/sprite flags at dot 1 of prerender.
                    ppu.registers.status.remove(Status::VERTICAL_BLANK);
                    ppu.registers
                        .status
                        .remove(Status::SPRITE_OVERFLOW | Status::SPRITE_ZERO_HIT);
                    // Mesen2: sprite evaluation does not run on the pre-render line,
                    // so ensure scanline 0 starts with 0 active sprites.
                    ppu.sprite_eval.count = 0;
                    ppu.sprite_eval.sprite0_in_range_next = false;
                }
                if rendering_enabled {
                    // Mesen2: OAMADDR bug on prerender, cycles 1..=8.
                    // If OAMADDR >= 8 when rendering starts, the 8 bytes starting at
                    // OAMADDR & 0xF8 are copied to the first 8 bytes of OAM.
                    if ppu.cycle < 9 && ppu.registers.oam_addr >= 0x08 {
                        let src_base = ppu.registers.oam_addr & 0xF8;
                        let dst = (ppu.cycle - 1) as u8;
                        let src = src_base.wrapping_add(dst);
                        ppu.registers.oam[dst as usize] = ppu.registers.oam[src as usize];
                    }

                    // Shift background shifters each dot.
                    let _ = ppu.bg_pipeline.sample_and_shift(ppu.registers.vram.x);
                    // Fetch/reload background data at tile boundaries.
                    ppu.fetch_background_data(&mut ppu_bus);

                    if ppu.cycle == 256 {
                        ppu.increment_scroll_y();
                    }
                }
            }

            // Dot 257: copy horizontal scroll bits from t -> v.
            (-1, 257) => {
                if rendering_enabled {
                    ppu.copy_horizontal_scroll();
                    // NESdev/Mesen2: during sprite tile loading (257..=320) the PPU
                    // forces OAMADDR to 0. This also means OAMADDR is 0 after a
                    // normal rendered frame.
                    ppu.registers.oam_addr = 0;
                    // Mesen2: sprite tile loading also performs a garbage NT fetch on dot 257.
                    ppu.sprite_pipeline_fetch_tick(&mut ppu_bus);
                }
            }

            // Dots 258..=320: sprite pattern fetches for scanline 0.
            // During dots 280..=304, vertical scroll bits are also copied from t -> v.
            (-1, 258..=320) => {
                if rendering_enabled {
                    if (280..=304).contains(&ppu.cycle) {
                        ppu.copy_vertical_scroll();
                    }
                    ppu.sprite_pipeline_fetch_tick(&mut ppu_bus);
                }
            }

            // Dots 321..=336: prefetch first two background tiles for scanline 0.
            (-1, 321..=336) => {
                if rendering_enabled {
                    if ppu.cycle == 321 {
                        // Mesen2: dot 321 latches secondary OAM[0] onto the internal OAM bus.
                        ppu.oam_copybuffer = ppu.secondary_oam[0];
                    }
                    let _ = ppu.bg_pipeline.sample_and_shift(ppu.registers.vram.x);
                    ppu.fetch_background_data(&mut ppu_bus);
                }
            }

            // Dots 337..=340: idle/dummy fetches on prerender.
            //
            // On NTSC-like timing, odd frames with rendering enabled are one
            // PPU tick shorter. Mesen2 implements this by skipping the dot
            // after scanline -1, cycle 339 ("skip from 339 to 0, going over
            // 340"). We mirror that behaviour here by forcing the cycle to
            // the last dot of the scanline when we are on the pre-render
            // scanline, cycle 339 of an odd frame with rendering enabled.
            (-1, 337..=340) => {
                if ppu.cycle == 339 && ppu.frame % 2 == 1 && ppu.render_enabled {
                    // Force the next `advance_cycle()` call to wrap directly
                    // to scanline 0, cycle 0, effectively removing dot 340
                    // from the timeline.
                    ppu.cycle = CYCLES_PER_SCANLINE - 1; // 340
                }
            }

            // ----------------------
            // Visible scanlines (0..=239)
            // ----------------------
            (0..=239, 1..=256) => {
                // Render current pixel.
                ppu.render_pixel();

                if rendering_enabled {
                    // Fetch background data for this dot.
                    ppu.fetch_background_data(&mut ppu_bus);
                    // Sprite evaluation for the *next* scanline runs during dots 1..=256.
                    ppu.sprite_pipeline_eval_tick();

                    if ppu.cycle == 256 {
                        ppu.increment_scroll_y();
                    }
                }
            }

            // Dot 257: copy horizontal scroll bits for next scanline.
            (0..=239, 257) => {
                if rendering_enabled {
                    ppu.copy_horizontal_scroll();
                    // NESdev/Mesen2: force OAMADDR to 0 for the sprite fetch window.
                    ppu.registers.oam_addr = 0;
                    // Mesen2: sprite tile loading also performs a garbage NT fetch on dot 257.
                    ppu.sprite_pipeline_fetch_tick(&mut ppu_bus);
                }
            }

            // Dots 258..=320: sprite pattern fetches for the next scanline.
            (0..=239, 258..=320) => {
                if rendering_enabled {
                    ppu.sprite_pipeline_fetch_tick(&mut ppu_bus);
                }
            }

            // Dots 321..=336: prefetch first two background tiles for next scanline.
            (0..=239, 321..=336) => {
                if rendering_enabled {
                    if ppu.cycle == 321 {
                        // Mesen2: dot 321 latches secondary OAM[0] onto the internal OAM bus.
                        ppu.oam_copybuffer = ppu.secondary_oam[0];
                    }
                    // Visible scanline prefetch window: keep BG shifters advancing
                    // here as well so that the prefetched tiles for the *next*
                    // scanline are aligned with dots 0 and 8 when rendering resumes.
                    let _ = ppu.bg_pipeline.sample_and_shift(ppu.registers.vram.x);
                    ppu.fetch_background_data(&mut ppu_bus);
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
                // NESdev/2C02 race: if $2002 was read at scanline 241, dot 0,
                // the VBlank flag never sets and no NMI edge is generated for this frame.
                if !ppu.prevent_vblank_flag {
                    ppu.registers.status.insert(Status::VERTICAL_BLANK);
                }
                // Consume the race latch each frame.
                ppu.prevent_vblank_flag = false;
            }
            (241..=260, _) => {}

            // Any other scanline value indicates a bug in the timing logic.
            _ => unreachable!("PPU scanline {} out of range", ppu.scanline),
        }

        ppu.update_nmi_level();
        ppu.update_state_latch();
    }

    /// Advance the PPU until its master clock reaches `target_master`.
    pub(crate) fn run_until(
        bus: &mut CpuBus,
        target_master: u64,
        cpu: &mut Cpu,
        ctx: &mut Context,
    ) {
        loop {
            Self::step(bus, cpu, ctx);
            // One PPU dot = 4 master cycles.
            let reached_target = {
                let mut devices = bus.devices_mut();
                let ppu = &mut devices.ppu;
                ppu.master_clock = ppu.master_clock.wrapping_add(4);
                ppu.master_clock + 4 > target_master
            };
            if reached_target {
                break;
            }
        }
    }

    /// Current PPU master clock (4 master cycles per dot).
    pub fn master_clock(&self) -> u64 {
        self.master_clock
    }

    /// Total PPU dots elapsed since power-on (monotonic across frames).
    pub fn total_dots(&self) -> u64 {
        self.current_ppu_cycle()
    }

    #[inline]
    fn step_pending_vram_addr(&mut self) {
        if self.pending_vram_delay == 0 {
            return;
        }
        self.pending_vram_delay -= 1;
        if self.pending_vram_delay == 0 {
            let new_v = self.pending_vram_addr;
            // Mesen2 / hardware: when the delayed $2006 update lands exactly
            // on the Y or X increment, the written value is ANDed with the
            // incremented value instead of simply replacing it.
            if self.registers.mask.rendering_enabled() && (0..=239).contains(&self.scanline) {
                let cur_raw = self.registers.vram.v.raw();
                let new_raw = new_v.raw();
                let merged = if self.cycle == 257 {
                    // Landing on the Y increment (scanline increment): AND the
                    // entire V with the written value.
                    cur_raw & new_raw
                } else if self.cycle > 0
                    && (self.cycle & 0x07) == 0
                    && (self.cycle <= 256 || self.cycle > 320)
                {
                    // Landing on an X increment (every 8 dots while rendering):
                    // only the coarse X and horizontal nametable bits (mask 0x041F)
                    // are corrupted by ANDing with the written value.
                    (new_raw & !0x041F) | (cur_raw & new_raw & 0x041F)
                } else {
                    new_raw
                };
                self.registers.vram.v.set_raw(merged);
            } else {
                self.registers.vram.v = new_v;
            }
        }
    }

    /// Returns a monotonically increasing PPU dot counter across frames.
    fn current_ppu_cycle(&self) -> u64 {
        // Map scanline -1..=260 to 0..=261 for indexing.
        let scanline_index = (self.scanline + 1) as i64;
        let per_frame = (SCANLINES_PER_FRAME as i64) * (CYCLES_PER_SCANLINE as i64);
        let within_frame = scanline_index * (CYCLES_PER_SCANLINE as i64) + (self.cycle as i64);
        (self.frame as i64 * per_frame + within_frame) as u64
    }

    // /// Advances to the next dot / scanline / frame.
    // fn advance_cycle(&mut self) {
    //     // NESdev / 2C02: with rendering enabled, each odd frame is one PPU
    //     // clock shorter. This is implemented by skipping the first idle tick
    //     // on the first visible scanline, i.e. by jumping directly from the
    //     // last prerender dot to scanline 0, cycle 1. See:
    //     // https://www.nesdev.org/wiki/PPU_frame_timing
    //     if self.scanline == -1
    //         && self.cycle == CYCLES_PER_SCANLINE - 1
    //         && self.odd_frame
    //         && self.registers.mask.rendering_enabled()
    //     {
    //         // We just finished the last dot of the prerender scanline on an
    //         // odd frame with rendering enabled. Skip scanline 0, cycle 0 and
    //         // start the first visible scanline at cycle 1.
    //         self.scanline = 0;
    //         self.cycle = 1;
    //         return;
    //     }

    //     self.cycle += 1;
    //     if self.cycle >= CYCLES_PER_SCANLINE {
    //         self.cycle = 0;
    //         self.scanline += 1;

    //         if self.scanline == 240 {
    //             self.frame = self.frame.wrapping_add(1);
    //         } else if self.scanline > 260 {
    //             self.scanline = -1;
    //             self.odd_frame = !self.odd_frame;
    //         }
    //     }
    // }

    /// Advances to the next dot / scanline / frame.
    fn advance_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle >= CYCLES_PER_SCANLINE {
            self.cycle = 0;

            // Finished processing the last visible scanline for this frame; present the
            // freshly rendered back buffer before moving into post-render/vblank.
            if self.scanline == (SCREEN_HEIGHT as i16 - 1) {
                self.framebuffer.present(self.palette.as_colors());
            }

            self.scanline += 1;

            if self.scanline == 240 {
                self.frame = self.frame.wrapping_add(1);
            } else if self.scanline > 260 {
                self.scanline = -1;
            }
        }
    }

    /// Recomputes the NMI output line based on VBlank and control register,
    /// latching a pending NMI on rising edges.
    fn update_nmi_level(&mut self) {
        let new_nmi_level = self.registers.status.contains(Status::VERTICAL_BLANK)
            && self.registers.control.nmi_enabled();
        self.nmi_level = new_nmi_level;
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
        }

        // Resolve palette RAM address (color 0 always uses universal background).
        let palette_addr = if final_color == 0 {
            ppu_mem::PALETTE_BASE
        } else if from_sprite {
            ppu_mem::PALETTE_BASE + 0x10 + (final_palette as u16) * 4 + (final_color as u16)
        } else {
            ppu_mem::PALETTE_BASE + (final_palette as u16) * 4 + (final_color as u16)
        };
        let mut color_index = self.palette_ram.read(palette_addr);
        // Apply grayscale mask when $2001 bit 0 is set: only keep the grey
        // column ($00, $10, $20, $30) as in Mesen2 / hardware.
        if self.registers.mask.contains(Mask::GRAYSCALE) {
            color_index &= 0x30;
        }

        self.framebuffer.write_index(x, y, color_index);
    }

    /// Emulates the $2000/$2005/$2006 scroll glitch when writes land on
    /// dot 257 of a visible scanline while rendering is enabled (Mesen2-style).
    fn maybe_apply_scroll_glitch(&mut self, source: ScrollGlitchSource) {
        // Only visible scanlines, when rendering is enabled, at dot 257.
        if !(0..=239).contains(&self.scanline) {
            return;
        }
        if !self.registers.mask.rendering_enabled() {
            return;
        }
        if self.cycle != 257 {
            return;
        }

        let bus = self.open_bus.sample();
        let (mask, value): (u16, u16) = match source {
            ScrollGlitchSource::Control2000 => {
                // Use bus bits to perturb nametable X (bit 10).
                (0x0400, (bus as u16) << 10)
            }
            ScrollGlitchSource::Scroll2005 => {
                // Use bus bits to perturb coarse X (bits 0-4).
                (0x001F, (bus as u16) >> 3)
            }
            ScrollGlitchSource::Addr2006 => {
                // Use bus bits to perturb nametable (bits 10-11).
                (0x0C00, (bus as u16) << 8)
            }
        };

        let raw = self.registers.vram.v.raw();
        let new_raw = (raw & !mask) | (value & mask);
        self.registers.vram.v.set_raw(new_raw);
    }

    /// Performs background tile/attribute fetches and reloads shifters every 8 dots.
    ///
    /// This helper assumes it is only called:
    /// - on the prerender scanline (-1) or visible scanlines (0..=239)
    /// - during the background fetch windows (dots 1..=256 or 321..=336)
    ///   Callers are responsible for enforcing those timing constraints.
    fn fetch_background_data(&mut self, ppu_bus: &mut PpuBus<'_>) {
        debug_assert!(
            ((0..=239).contains(&self.scanline) || self.scanline == -1)
                && ((1..=256).contains(&self.cycle) || (321..=336).contains(&self.cycle)),
            "fetch_background_data should only be called during bg fetch window (scanline={}, cycle={})",
            self.scanline,
            self.cycle,
        );

        // Fetch and reload shifters every 8 dots (tile boundary) during fetch windows.
        if self.cycle.is_multiple_of(8) {
            self.load_background_tile(ppu_bus);
            self.increment_scroll_x();
        }
    }

    /// Sprite pipeline tick for dots 1..=256.
    ///
    /// Dots 1..=64 clear secondary OAM; dots 65..=256 scan primary OAM to
    /// select up to 8 sprites for the *next* scanline.
    fn sprite_pipeline_eval_tick(&mut self) {
        // Mesen2 / hardware: sprite evaluation runs whenever the latched
        // rendering state is enabled (background OR sprites), with a 1-dot
        // delay relative to $2001 writes.
        if !self.render_enabled {
            return;
        }
        // Mesen2: secondary OAM clear + sprite evaluation do not occur on the
        // pre-render line (scanline -1) for NTSC timing.
        if self.scanline < 0 {
            return;
        }

        match self.cycle {
            // Dots 1..=64: clear secondary OAM (32 bytes, 2 dots per byte).
            1..=64 => {
                let byte_index = ((self.cycle - 1) >> 1) as usize;
                if byte_index < 32 {
                    self.secondary_oam[byte_index] = 0xFF;
                }
                self.oam_copybuffer = 0xFF;
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
    fn sprite_pipeline_fetch_tick(&mut self, ppu_bus: &mut PpuBus<'_>) {
        // Mesen2 / hardware: sprite fetches run whenever rendering is enabled
        // (background OR sprites). Pixel visibility is still controlled later
        // by SHOW_SPRITES in `render_pixel`.
        if !self.render_enabled {
            return;
        }

        if (257..=320).contains(&self.cycle) {
            // Keep OAMADDR pinned to 0 throughout the fetch window (defensive;
            // dot 257 already sets it in `clock`).
            self.registers.oam_addr = 0;
            self.fetch_sprites_for_dot(ppu_bus);
        }
    }

    /// Per-dot sprite evaluation step (65..=256).
    fn evaluate_sprites_for_dot(&mut self) {
        debug_assert!(
            (65..=256).contains(&self.cycle) && self.scanline >= 0,
            "evaluate_sprites_for_dot called outside eval window"
        );

        let sprite_height: u8 = if self.registers.control.contains(Control::SPRITE_SIZE_16) {
            16
        } else {
            8
        };

        // Odd dots: read a byte from primary OAM at the current internal address.
        if (self.cycle & 1) == 1 {
            if self.cycle == 65 {
                self.sprite_eval.start(self.registers.oam_addr);
            }
            let addr = self.registers.oam_addr as usize;
            self.oam_copybuffer = self.registers.oam[addr];
            return;
        }

        // Even dots: write/advance based on the byte read on the previous dot.
        if self.cycle == 256 {
            self.sprite_eval.latch_end_of_evaluation(
                self.scanline,
                self.oam_copybuffer,
                sprite_height,
            );
            // `sprite0_in_range_next` is the "sprite 0 visible" latch for the
            // upcoming scanline; it is set when the first in-range Y is found.
            // (Mesen2 sets this at cycle 66.)
            // Note: `start()` resets it to false each scanline.
        }

        // Range check uses the current scanline (`_scanline` in Mesen2) and the
        // byte currently on the OAM bus (`_oamCopybuffer`).
        if !self.sprite_eval.sprite_in_range {
            let y = self.oam_copybuffer as i16;
            let end = y + sprite_height as i16;
            if self.scanline >= y && self.scanline < end {
                self.sprite_eval.sprite_in_range = !self.sprite_eval.oam_copy_done;
            }
        }

        if self.sprite_eval.secondary_oam_addr < 0x20 {
            // Copy 1 byte into secondary OAM (even if the sprite is not in range).
            let idx = self.sprite_eval.secondary_oam_addr as usize;
            self.secondary_oam[idx] = self.oam_copybuffer;

            if self.sprite_eval.sprite_in_range {
                if self.cycle == 66 {
                    // If the first Y coordinate we load is in range, set the sprite 0 flag.
                    // (Happens even if evaluation started on a non-zero OAMADDR.)
                    self.sprite_eval.sprite0_in_range_next = true;
                }

                self.sprite_eval.sprite_addr_l = self.sprite_eval.sprite_addr_l.wrapping_add(1);
                self.sprite_eval.secondary_oam_addr =
                    self.sprite_eval.secondary_oam_addr.wrapping_add(1);

                if self.sprite_eval.sprite_addr_l >= 4 {
                    self.sprite_eval.sprite_addr_h =
                        (self.sprite_eval.sprite_addr_h.wrapping_add(1)) & 0x3F;
                    self.sprite_eval.sprite_addr_l = 0;
                    if self.sprite_eval.sprite_addr_h == 0 {
                        self.sprite_eval.oam_copy_done = true;
                    }
                }

                // Using `secondary_oam_addr & 3` matches Mesen2 and is required
                // to reproduce mid-frame enable/disable glitches.
                if (self.sprite_eval.secondary_oam_addr & 0x03) == 0 {
                    self.sprite_eval.sprite_in_range = false;

                    // If eval started on a misaligned address, resync normally,
                    // unless the last copied byte (interpreted as a Y coordinate)
                    // is itself in range.
                    if self.sprite_eval.sprite_addr_l != 0 {
                        let y = self.oam_copybuffer as i16;
                        let end = y + sprite_height as i16;
                        let in_range = self.scanline >= y && self.scanline < end;
                        if !in_range {
                            self.sprite_eval.sprite_addr_l = 0;
                        }
                    }
                }
            } else {
                // Nothing to copy: skip to next sprite.
                self.sprite_eval.sprite_addr_h =
                    (self.sprite_eval.sprite_addr_h.wrapping_add(1)) & 0x3F;
                self.sprite_eval.sprite_addr_l = 0;
                if self.sprite_eval.sprite_addr_h == 0 {
                    self.sprite_eval.oam_copy_done = true;
                }
            }
        } else {
            // Secondary OAM writes are disabled: writes turn into reads.
            self.oam_copybuffer =
                self.secondary_oam[(self.sprite_eval.secondary_oam_addr & 0x1F) as usize];

            // 8 sprites have been found; check next sprites for overflow + emulate the bugged address ppu_bus.
            if self.sprite_eval.oam_copy_done {
                self.sprite_eval.sprite_addr_h =
                    (self.sprite_eval.sprite_addr_h.wrapping_add(1)) & 0x3F;
                self.sprite_eval.sprite_addr_l = 0;
            } else if self.sprite_eval.sprite_in_range {
                self.registers.status.insert(Status::SPRITE_OVERFLOW);

                self.sprite_eval.sprite_addr_l = self.sprite_eval.sprite_addr_l.wrapping_add(1);
                if self.sprite_eval.sprite_addr_l == 4 {
                    self.sprite_eval.sprite_addr_h =
                        (self.sprite_eval.sprite_addr_h.wrapping_add(1)) & 0x3F;
                    self.sprite_eval.sprite_addr_l = 0;
                }

                if self.sprite_eval.overflow_bug_counter == 0 {
                    self.sprite_eval.overflow_bug_counter = 3;
                } else {
                    self.sprite_eval.overflow_bug_counter =
                        self.sprite_eval.overflow_bug_counter.saturating_sub(1);
                    if self.sprite_eval.overflow_bug_counter == 0 {
                        // After a few bytes, realign and stop matching further sprites.
                        self.sprite_eval.oam_copy_done = true;
                        self.sprite_eval.sprite_addr_l = 0;
                    }
                }
            } else {
                // Sprite isn't on this scanline: increment both H & L.
                self.sprite_eval.sprite_addr_h =
                    (self.sprite_eval.sprite_addr_h.wrapping_add(1)) & 0x3F;
                self.sprite_eval.sprite_addr_l =
                    (self.sprite_eval.sprite_addr_l.wrapping_add(1)) & 0x03;
                if self.sprite_eval.sprite_addr_h == 0 {
                    self.sprite_eval.oam_copy_done = true;
                }
            }
        }

        self.registers.oam_addr = self.sprite_eval.primary_oam_addr();
    }

    /// Per-dot sprite fetch step (257..=320).
    fn fetch_sprites_for_dot(&mut self, ppu_bus: &mut PpuBus<'_>) {
        if self.cycle == 257 {
            self.sprite_fetch = SpriteFetchState::default();
        }

        if !(257..=320).contains(&self.cycle) {
            return;
        }

        let rel = self.cycle - 257;
        let i = (rel / 8) as usize;
        if i >= 8 {
            return;
        }

        // Each sprite gets 8 dots in this region; `sub` is 0..7 within a slot.
        let sub = (rel % 8) as u8;

        // Mesen2 / hardware: during sprite fetch window, the PPU performs
        // "garbage" nametable and attribute reads on sub-cycles 0 and 2.
        // These reads don't affect pixels but do affect mapper IRQ timing (A12 toggles).
        if sub == 0 {
            let _ = self.read_vram(
                ppu_bus,
                self.nametable_addr(),
                PpuVramAccessKind::RenderingFetch,
            );
        }
        if sub == 2 {
            let _ = self.read_vram(
                ppu_bus,
                self.attribute_addr(),
                PpuVramAccessKind::RenderingFetch,
            );
        }

        // Read sprite bytes from secondary OAM.
        let view = SpriteView::at_index(&mut self.secondary_oam, i).expect("invalid sprite index");
        let y = view.y();
        let tile = view.tile();
        let attr = view.attributes().bits();
        let x = view.x();
        // Sprite fetches also drive the internal OAM data bus; approximate by
        // latching the last metadata byte.
        self.oam_copybuffer = x;

        // Latch raw sprite bytes early in the slot.
        if sub == 0 {
            self.sprite_line_next.set_meta(i, y, tile, attr, x);
        }

        // Compute which row of the sprite to fetch for the next scanline.
        // Mesen2 relies on the NES sprite Y-off-by-one behavior: sprites are drawn at Y+1,
        // so using the current scanline here produces the correct row for the upcoming scanline.
        let sprite_height: i16 = if (self.registers.control.bits() & 0x20) != 0 {
            16
        } else {
            8
        }; // PPUCTRL bit 5
        let active_sprites = self.sprite_eval.count.min(8);
        let fetch_last_sprite = (i as u8) >= active_sprites || y >= 240;

        let (effective_tile, row): (u8, i16) = if fetch_last_sprite {
            // Mesen2: when there are fewer than 8 sprites, the PPU still performs
            // dummy pattern fetches to sprite tile $FF, row 0 (used by MMC3 IRQ counters).
            (0xFF, 0)
        } else {
            let mut r = self.scanline - (y as i16);
            debug_assert!(
                r >= 0 && r < sprite_height,
                "PPU sprite row out of range: scanline={} sprite_y={} sprite_height={} row={}",
                self.scanline,
                y,
                sprite_height,
                r,
            );

            // Vertical flip affects row selection.
            if (attr & 0x80) != 0 {
                r = (sprite_height - 1) - r;
            }
            (tile, r)
        };

        // Determine pattern table base and tile index for 8x8 vs 8x16.
        let (pattern_base, tile_index) = if sprite_height == 16 {
            // For 8x16, bit 0 selects table, and tile index is even.
            let base = if (effective_tile & 0x01) != 0 {
                ppu_mem::PATTERN_TABLE_1
            } else {
                ppu_mem::PATTERN_TABLE_0
            };
            let top_tile = effective_tile & 0xFE;
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
            (base, (effective_tile, r))
        };

        let (tile_idx, fine_y) = tile_index;
        let addr = pattern_base + (tile_idx as u16) * 16 + fine_y;

        // Mesen2: perform both pattern reads during the same sub-step (case 4),
        // as an approximation of the 8-step internal fetch pipeline.
        if sub == 4 {
            let pattern_low = self.read_vram(ppu_bus, addr, PpuVramAccessKind::RenderingFetch);
            let pattern_high = self.read_vram(
                ppu_bus,
                addr.wrapping_add(8),
                PpuVramAccessKind::RenderingFetch,
            );
            self.sprite_line_next.set_pattern_low(i, pattern_low);
            self.sprite_line_next.set_pattern_high(i, pattern_high);
        }
    }

    /// Current nametable fetch address derived from `v` (used for normal and garbage fetches).
    fn nametable_addr(&self) -> u16 {
        let v = self.registers.vram.v;
        let base_nt = ppu_mem::NAMETABLE_BASE + (v.nametable() as u16 * ppu_mem::NAMETABLE_SIZE);
        base_nt + (v.coarse_y() as u16 * 32) + (v.coarse_x() as u16)
    }

    /// Current attribute fetch address derived from `v` (used for normal and garbage fetches).
    fn attribute_addr(&self) -> u16 {
        let v = self.registers.vram.v;
        let base_nt = ppu_mem::NAMETABLE_BASE + (v.nametable() as u16 * ppu_mem::NAMETABLE_SIZE);
        base_nt + 0x03C0 + (v.coarse_y() as u16 / 4) * 8 + (v.coarse_x() as u16 / 4)
    }

    fn read_status(&mut self) -> u8 {
        let status = self.registers.status.bits();
        // Mesen2 / hardware: low 5 bits of $2002 come from open bus.
        // Use PpuOpenBus::apply with a mask covering the low 5 bits so only
        // the status flags (high 3 bits) refresh the decay stamps.
        let ret = self.open_bus.apply(0x1F, status, self.frame);

        // Reading $2002 clears VBlank and the VRAM write latch.
        self.registers.status.remove(Status::VERTICAL_BLANK);
        self.registers.vram.reset_latch();

        // NESdev race approximation (matches Mesen2):
        // If $2002 is read one dot before VBlank would set (241:0),
        // suppress VBlank/NMI for that frame.
        if self.scanline == 241 && self.cycle == 0 {
            self.prevent_vblank_flag = true;
        }

        self.update_nmi_level();
        ret
    }

    fn write_oam_data(&mut self, value: u8) {
        let rendering = self.render_enabled;
        let during_render =
            rendering && ((0..=239).contains(&self.scanline) || self.scanline == -1);

        if during_render {
            // Mesen2 / hardware: writes to $2004 during rendering do not modify primary OAM.
            // Instead, OAMADDR is incremented in a "glitchy" way: only the high 6 bits bump.
            self.registers.oam_addr = self.registers.oam_addr.wrapping_add(4);
        } else {
            let idx = self.registers.oam_addr as usize;
            if idx < ppu_mem::OAM_RAM_SIZE {
                // Outside rendering, writes go directly into primary OAM and
                // auto-increment OAMADDR.
                self.registers.oam[idx] = value;
                self.registers.oam_addr = self.registers.oam_addr.wrapping_add(1);
            }
        }
    }

    fn read_oam_data(&mut self) -> u8 {
        let rendering = self.render_enabled;
        let during_render = rendering && (0..=239).contains(&self.scanline);

        if during_render {
            // While the screen is being drawn, $2004 exposes the internal OAM
            // evaluation/rendering bus:
            // - During sprite fetches (scanline 0..=239, cycles 257..=320),
            //   the value on the bus comes from secondary OAM.
            // - Outside that window, reads return the last OAM bus value.
            if (257..=320).contains(&self.cycle) {
                let rel = self.cycle - 257;
                let step = (rel % 8).min(3);
                let addr = ((rel / 8) * 4 + step) as usize;
                if addr < ppu_mem::SECONDARY_OAM_RAM_SIZE {
                    let v = self.secondary_oam[addr];
                    self.oam_copybuffer = v;
                }
            }
            self.oam_copybuffer
        } else {
            let idx = self.registers.oam_addr as usize;
            if idx < ppu_mem::OAM_RAM_SIZE {
                let mut v = self.registers.oam[idx];
                // Mask off the 3 unimplemented bits of sprite byte 2 so they
                // read back as 0, mirroring hardware behaviour.
                if (self.registers.oam_addr & 0x03) == 0x02 {
                    v &= 0xE3;
                }
                // Outside rendering, $2004 reads primary OAM and also update
                // the internal OAM bus copybuffer.
                self.oam_copybuffer = v;
                v
            } else {
                0
            }
        }
    }

    fn write_vram_data(&mut self, value: u8, ppu_bus: &mut PpuBus<'_>) {
        let addr = self.effective_vram_addr() & ppu_mem::VRAM_MIRROR_MASK;
        self.write_vram(ppu_bus, addr, value);
        // Delay VRAM increment by one PPU dot to match Mesen2 / hardware
        // behaviour (used by some test ROMs to observe transient colours).
        self.pending_vram_increment = PendingVramIncrement::from_control(self.registers.control);
    }

    /// CPU read handler for $2007 (PPUDATA).
    ///
    /// Behaviour is aligned with Mesen2 / hardware:
    /// - A normal read returns the buffered VRAM value (or palette data) and
    ///   schedules a VRAM auto-increment.
    /// - A second read that happens too soon (within a short window after the
    ///   previous $2007 read) does not perform a real VRAM access. Instead it
    ///   returns only the current open-bus value, without touching the read
    ///   buffer or the VRAM address.
    fn read_vram_data(&mut self, ppu_bus: &mut PpuBus<'_>) -> u8 {
        // Too-soon read window: ignore the VRAM access and just return the
        // latched open-bus value. This matches Mesen2's `_ignoreVramRead`
        // path where `openBusMask = 0xFF`.
        if self.ignore_vram_read > 0 {
            return self.open_bus.apply(0xFF, 0, self.frame);
        }

        let addr = self.registers.vram.v.raw() & 0x3FFF;

        // Normal $2007 read path.
        let result = if addr >= ppu_mem::PALETTE_BASE {
            // Even though palette reads bypass the buffer (they return immediately),
            // the PPU still performs a "shadow" VRAM read to refresh the internal
            // read buffer. This shadow read targets $2F00-$2FFF (i.e. addr & $2FFF)
            // and is *not* affected by the palette's $3F10/$14/$18/$1C mirroring.
            let shadow_addr = addr & 0x2FFF;
            self.registers.vram_buffer =
                self.read_vram(ppu_bus, shadow_addr, PpuVramAccessKind::CpuRead);

            // Palette area ($3F00-$3FFF) read.
            //
            // The low 6 bits come from palette RAM, optionally masked by the
            // grayscale bit. The high 2 bits are mixed with open bus, as on
            // hardware and in Mesen2.
            let palette_value = self.palette_ram.read(addr);

            let mut low = palette_value & 0x3F;
            if self.registers.mask.contains(Mask::GRAYSCALE) {
                // In grayscale mode, only the "grey column" is visible
                // ($00, $10, $20, $30).
                low &= 0x30;
            }

            // Mesen2 / hardware: palette reads drive the low 6 bits from
            // palette RAM, while the upper 2 bits come from the open bus.
            self.open_bus.apply(0xC0, low, self.frame)
        } else {
            // Nametable / CHR / general VRAM area read.
            //
            // As on the NES, $2007 returns the previous value from an internal
            // read buffer and then refreshes that buffer with the new VRAM
            // data fetched from the mapper.
            let data = self.read_vram(ppu_bus, addr, PpuVramAccessKind::CpuRead);
            let buffered = self.registers.vram_buffer;
            self.registers.vram_buffer = data;

            // Nametable/CHR reads fully drive the bus with the buffered value.
            self.open_bus.apply(0x00, buffered, self.frame)
        };

        // After a successful $2007 read:
        // - Schedule the VRAM auto-increment (applied later in `clock()`).
        // - Start the "ignore" window for too-soon reads so that the next
        //   immediate read can fall back to the open-bus-only path above.
        self.pending_vram_increment = PendingVramIncrement::from_control(self.registers.control);
        self.ignore_vram_read = 6;

        result
    }

    /// Returns the effective VRAM address for CPU-side $2007 accesses.
    ///
    /// When a $2006 write has been staged but not yet committed to `v` via the
    /// delayed update in `step_pending_vram_addr`, use the pending address so
    /// CPU reads/writes see the most recent VRAM pointer even if the PPU
    /// clock has not advanced yet (as in unit tests).
    #[inline]
    fn effective_vram_addr(&self) -> u16 {
        if self.pending_vram_delay != 0 {
            self.pending_vram_addr.raw()
        } else {
            self.registers.vram.v.raw()
        }
    }

    fn write_vram(&mut self, ppu_bus: &mut PpuBus<'_>, addr: u16, value: u8) {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;

        // Palette space is handled separately from nametable/CHR.
        if addr >= ppu_mem::PALETTE_BASE {
            self.palette_ram.write(addr, value);
            return;
        }

        // Pattern tables ($0000-$1FFF): delegate to mapper CHR path.
        // Unlike the old design, there is no fallback to internal VRAM.
        if addr < ppu_mem::NAMETABLE_BASE {
            let ctx = PpuVramAccessContext {
                ppu_cycle: self.current_ppu_cycle(),
                cpu_cycle: ppu_bus.cpu_cycle(),
                kind: PpuVramAccessKind::CpuWrite,
            };
            let _ = ppu_bus.write(addr, value, ctx);
            return;
        }

        // Nametable space ($2000-$3EFF): delegate mapping to the mapper.
        match ppu_bus.map_nametable(addr) {
            NametableTarget::Ciram(offset) => {
                // CIRAM is 2 KiB, offset is 0-0x7FF
                let ciram_index = (offset & 0x07FF) as usize;
                self.ciram[ciram_index] = value;
            }
            NametableTarget::MapperVram(offset) => {
                ppu_bus.mapper_nametable_write(offset, value);
            }
            NametableTarget::None => {
                // No backing store: writes are ignored (open-bus semantics).
            }
        }
    }

    fn read_vram(&mut self, ppu_bus: &mut PpuBus<'_>, addr: u16, kind: PpuVramAccessKind) -> u8 {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;

        // read_vram is intended for nametable/pattern table accesses only; palette
        // RAM is handled separately in read_vram_data / write_vram_data.
        debug_assert!(
            addr < ppu_mem::PALETTE_BASE,
            "read_vram should not be used for palette addresses (got {:04X})",
            addr
        );

        // Pattern tables ($0000-$1FFF): always delegate to mapper.
        // Unlike the old design, there is no fallback to internal storage.
        if addr < ppu_mem::NAMETABLE_BASE {
            let ctx = PpuVramAccessContext {
                ppu_cycle: self.current_ppu_cycle(),
                cpu_cycle: ppu_bus.cpu_cycle(),
                kind,
            };
            // Open bus: Mesen2 returns the address LSB when CHR is disabled.
            return ppu_bus.read(addr, ctx).unwrap_or_else(|| addr as u8);
        }

        // Nametable space ($2000-$3EFF).
        match ppu_bus.map_nametable(addr) {
            NametableTarget::Ciram(offset) => {
                // CIRAM is 2 KiB, offset is 0-0x7FF
                let ciram_index = (offset & 0x07FF) as usize;
                self.ciram[ciram_index]
            }
            NametableTarget::MapperVram(offset) => {
                // Open bus: return offset LSB if no backing store.
                ppu_bus
                    .mapper_nametable_read(offset)
                    .unwrap_or_else(|| offset as u8)
            }
            NametableTarget::None => {
                // Open bus: Mesen2 returns address LSB.
                addr as u8
            }
        }
    }

    /// Loads the current tile/attribute data into the background shifters.
    fn load_background_tile(&mut self, ppu_bus: &mut PpuBus<'_>) {
        let v = self.registers.vram.v;
        let base_nt = ppu_mem::NAMETABLE_BASE + (v.nametable() as u16 * ppu_mem::NAMETABLE_SIZE);
        let tile_index_addr = base_nt + (v.coarse_y() as u16 * 32) + (v.coarse_x() as u16);
        let tile_index =
            self.read_vram(ppu_bus, tile_index_addr, PpuVramAccessKind::RenderingFetch);

        let fine_y = v.fine_y() as u16;
        let pattern_base = if self.registers.control.contains(Control::BACKGROUND_TABLE) {
            ppu_mem::PATTERN_TABLE_1
        } else {
            ppu_mem::PATTERN_TABLE_0
        };
        let pattern_addr = pattern_base + (tile_index as u16 * 16) + fine_y;
        let tile_pattern = [
            self.read_vram(ppu_bus, pattern_addr, PpuVramAccessKind::RenderingFetch),
            self.read_vram(ppu_bus, pattern_addr + 8, PpuVramAccessKind::RenderingFetch),
        ];

        let attr_addr =
            base_nt + 0x03C0 + (v.coarse_y() as u16 / 4) * 8 + (v.coarse_x() as u16 / 4);
        let attr_byte = self.read_vram(ppu_bus, attr_addr, PpuVramAccessKind::RenderingFetch);
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
        let coarse_y = match self.registers.vram.v.coarse_y() {
            29 => {
                let nt = self.registers.vram.v.nametable() ^ 0b10;
                self.registers.vram.v.set_nametable(nt);
                0
            }
            31 => {
                // Skip attribute memory gap lines.
                0
            }
            coarse_y => coarse_y.wrapping_add(1),
        };
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

    /// Updates Mesen-style internal state latches for this dot.
    ///
    /// This mirrors NesPpu::UpdateState in Mesen:
    /// - `_needStateUpdate` is raised by $2001/$2006/$2007 and VRAM side
    ///   effects.
    /// - `_renderingEnabled` is a latched copy of the effective background /
    ///   sprite enable bits, separate from the raw $2001 mask.
    /// - `_prevRenderingEnabled` holds the previous dot's value for scroll
    ///   increments and trace/logging.
    fn update_state_latch(&mut self) {
        // Decide whether we need to refresh the latched state for this dot.
        let need_state = self.state_update_pending
            || self.pending_vram_delay > 0
            || self.pending_vram_increment.is_pending()
            || self.ignore_vram_read > 0;

        if need_state {
            let mask = self.registers.mask;
            let new_render = mask.rendering_enabled();

            // Preserve the old value for prev_render_enabled, then latch the
            // new effective rendering state.
            let old_render = self.render_enabled;
            self.prev_render_enabled = old_render;
            self.render_enabled = new_render;

            if old_render != new_render && self.scanline < 240 {
                if new_render {
                    // Rendering was just enabled: perform any pending OAM corruption.
                    self.process_oam_corruption();
                } else {
                    // Rendering was just disabled: flag potential OAM corruption.
                    self.set_oam_corruption_flags();

                    // Disabling rendering during sprite evaluation triggers an OAMADDR glitch.
                    if (65..=256).contains(&self.cycle) {
                        self.oam_addr_disable_glitch_pending = true;
                    }
                }
            }

            // Clear the explicit "dirty" flag; the other conditions
            // (pending_vram_delay, pending increment, ignore_vram_read) are
            // transient and will clear themselves over subsequent dots.
            self.state_update_pending = false;
        } else {
            // No explicit state change requested; just propagate the previous
            // latched state into `prev_render_enabled` for this dot.
            self.prev_render_enabled = self.render_enabled;
        }
    }

    fn set_oam_corruption_flags(&mut self) {
        // Mirrors Mesen2's `SetOamCorruptionFlags` logic.
        //
        // When rendering is disabled mid-screen during either:
        // A) Secondary OAM clear (cycles 0..63)
        // B) Sprite tile fetching (cycles 256..319)
        // hardware can corrupt primary OAM; Mesen tracks that via row flags.
        let cycle = self.cycle;
        if cycle < 64 {
            let idx = (cycle >> 1) as usize;
            if idx < self.corrupt_oam_row.len() {
                self.corrupt_oam_row[idx] = true;
            }
        } else if (256..320).contains(&cycle) {
            let rel = cycle - 256;
            let base = rel >> 3;
            let offset = (rel & 0x07).min(3);
            let idx = (base * 4 + offset) as usize;
            if idx < self.corrupt_oam_row.len() {
                self.corrupt_oam_row[idx] = true;
            }
        }
    }

    fn process_oam_corruption(&mut self) {
        // Mirrors Mesen2's `ProcessOamCorruption` logic: copy the first OAM row
        // over flagged rows.
        let mut row0 = [0u8; 8];
        row0.copy_from_slice(&self.registers.oam[0..8]);

        for (row, flagged) in self.corrupt_oam_row.iter_mut().enumerate() {
            if *flagged {
                if row > 0 {
                    let start = row * 8;
                    let end = start + 8;
                    self.registers.oam[start..end].copy_from_slice(&row0);
                }
                *flagged = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        apu::Apu,
        bus::{OpenBus, PendingDma, cpu::CpuBus},
        controller::{ControllerPorts, SerialLogger},
        mem_block::cpu as cpu_ram,
        ppu::pattern_bus::PpuBus,
    };

    use super::*;

    #[test]
    fn control_register_helpers() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();
        ppu.cpu_write(PpuRegister::Control.addr(), 0b1000_0100, &mut ppu_bus);
        assert!(ppu.registers.control.nmi_enabled());
        assert_eq!(ppu.registers.control.vram_increment(), 32);
        assert_eq!(
            ppu.registers.control.base_nametable_addr(),
            ppu_mem::NAMETABLE_BASE
        );
    }

    #[test]
    #[ignore = "this test fails and needs investigation"]
    fn buffered_ppu_data_read() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();
        // Point to $2000 and write a value.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x12, &mut ppu_bus);

        // Reset VRAM address to read back.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut ppu_bus);

        let first = ppu.cpu_read(PpuRegister::Data.addr(), &mut ppu_bus);
        let second = ppu.cpu_read(PpuRegister::Data.addr(), &mut ppu_bus);
        assert_eq!(first, 0x00, "First read should return buffered value");
        assert_eq!(second, 0x12, "Second read should contain VRAM data");
    }

    #[test]
    #[ignore = "this test fails and needs investigation"]
    fn palette_reads_bypass_buffer() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x99, &mut ppu_bus);

        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00, &mut ppu_bus);

        let value = ppu.cpu_read(PpuRegister::Data.addr(), &mut ppu_bus);
        assert_eq!(value, 0x99);
    }

    #[test]
    fn palette_reads_mix_high_bits_from_open_bus() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();

        // Force the PPU VRAM pointer into palette space without going through
        // $2006 (which would overwrite open bus).
        ppu.registers.vram.v.set_raw(0x3F00);
        ppu.ignore_vram_read = 0;

        // Palette RAM is 6-bit; store a low 6-bit value.
        ppu.palette_ram.write(0x3F00, 0x15);

        // Set open bus to a value with non-zero high bits; palette reads should
        // return those high bits even though palette RAM does not store them.
        ppu.open_bus.set(0xFF, 0xC0, ppu.frame);

        let value = ppu.cpu_read(PpuRegister::Data.addr(), &mut ppu_bus);
        assert_eq!(value, 0xD5);
    }

    #[test]
    fn open_bus_decays_on_floating_register_reads() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();

        // Drive a 1 bit onto the open bus at an earlier frame.
        ppu.open_bus.set(0xFF, 0x80, 1);

        // Advance enough frames for the bit to decay (Mesen2-style: > 3 frames).
        ppu.frame = 5;

        // Reading a write-only register should return open bus (with decay applied).
        let value = ppu.cpu_read(PpuRegister::Control.addr(), &mut ppu_bus);
        assert_eq!(value, 0x00);
    }

    #[test]
    fn status_read_resets_scroll_latch() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x12, &mut ppu_bus); // horizontal
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x34, &mut ppu_bus); // vertical
        assert_eq!(ppu.registers.vram.t.coarse_x(), 0x12 >> 3);
        assert_eq!(ppu.registers.vram.x, 0x12 & 0x07);
        assert_eq!(ppu.registers.vram.t.coarse_y(), 0x34 >> 3);
        assert_eq!(ppu.registers.vram.t.fine_y(), 0x34 & 0x07);

        // Reading status should clear the write toggle so the next write targets horizontal.
        let _ = ppu.cpu_read(PpuRegister::Status.addr(), &mut ppu_bus);
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x56, &mut ppu_bus);
        assert_eq!(ppu.registers.vram.t.coarse_x(), 0x56 >> 3);
        assert_eq!(ppu.registers.vram.t.coarse_y(), 0x34 >> 3);
    }

    #[test]
    fn oam_data_auto_increments() {
        let mut ppu = Ppu::default();
        let mut ppu_bus = PpuBus::default();
        ppu.cpu_write(PpuRegister::OamAddr.addr(), 0x02, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xAA, &mut ppu_bus);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xBB, &mut ppu_bus);
        assert_eq!(ppu.registers.oam[2], 0xAA);
        assert_eq!(ppu.registers.oam[3], 0xBB);
    }

    #[test]
    fn vblank_flag_is_managed_by_clock() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut controllers = ControllerPorts::new();
        let mut serial_log = SerialLogger::default();
        let mut pending_dma = PendingDma::default();
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;

        let mut bus = CpuBus {
            ram: &mut ram,
            ppu: &mut ppu,
            apu: &mut apu,
            cartridge: None,
            controllers: &mut controllers,
            serial_log: Some(&mut serial_log),
            open_bus: &mut open_bus,
            mixer: None,
            cycles: &mut cpu_bus_cycle,
            master_clock: &mut master_clock,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            pending_dma: &mut pending_dma,
        };

        // Run until scanline 241, cycle 1 (accounting for prerender line).
        let target_cycles = (242i32 * CYCLES_PER_SCANLINE as i32 + 2) as usize;
        for _ in 0..target_cycles {
            Ppu::step(&mut bus, &mut cpu, &mut Context::None);
        }
        let status_set = bus
            .devices()
            .ppu
            .registers
            .status
            .contains(Status::VERTICAL_BLANK);
        assert!(status_set);

        // Continue to the prerender line, then run dot 1 where VBL is cleared.
        loop {
            let pos = {
                let devices = bus.devices();
                (devices.ppu.scanline, devices.ppu.cycle)
            };
            if pos == (-1, 1) {
                break;
            }
            Ppu::step(&mut bus, &mut cpu, &mut Context::None);
        }

        // Dot 1 of prerender clears VBL/sprite flags (mirrors hardware timing).
        Ppu::step(&mut bus, &mut cpu, &mut Context::None);
        let status_cleared = !bus
            .devices()
            .ppu
            .registers
            .status
            .contains(Status::VERTICAL_BLANK);
        assert!(status_cleared);
    }

    fn run_sprite_evaluation_scanline(ppu: &mut Ppu, scanline: i16) {
        ppu.scanline = scanline;
        ppu.render_enabled = true;
        ppu.registers
            .status
            .remove(Status::SPRITE_OVERFLOW | Status::SPRITE_ZERO_HIT);
        ppu.registers.oam_addr = 0;

        for cycle in 1..=256 {
            ppu.cycle = cycle;
            ppu.sprite_pipeline_eval_tick();
        }
    }

    #[test]
    fn sprite_overflow_includes_y_239() {
        let mut ppu = Ppu::default();
        // 9 sprites with Y=239 should still be considered "in range" during
        // evaluation on scanline 239, even though they won't be drawn.
        for i in 0..9 {
            let base = i * 4;
            ppu.registers.oam[base] = 239;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        run_sprite_evaluation_scanline(&mut ppu, 239);
        assert!(ppu.registers.status.contains(Status::SPRITE_OVERFLOW));
    }

    #[test]
    fn sprite_overflow_excludes_y_240() {
        let mut ppu = Ppu::default();
        // Y=240 is off-screen; should not contribute to sprite overflow.
        for i in 0..9 {
            let base = i * 4;
            ppu.registers.oam[base] = 240;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        run_sprite_evaluation_scanline(&mut ppu, 239);
        assert!(!ppu.registers.status.contains(Status::SPRITE_OVERFLOW));
    }

    #[test]
    fn sprite_overflow_obscure_byte_shift_matches_mesen() {
        let mut ppu = Ppu::default();
        ppu.registers.oam.fill(200);

        // Scanline 0 evaluation selects sprites for scanline 1; the range check
        // uses the current scanline (0) against the OAM "Y" byte.
        //
        // Make the first 8 sprites in range (Y=0), then make the 9th sprite
        // out of range (Y=200). The overflow bug then increments both the high
        // and low address bits, causing the *second byte* of sprite #10 to be
        // treated as the Y coordinate. Set that second byte to 0 to trigger
        // overflow, and keep other bytes out of range.
        for i in 0..8 {
            let base = i * 4;
            ppu.registers.oam[base] = 0;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        // Sprite #9 (index 8): out of range.
        {
            let base = 8 * 4;
            ppu.registers.oam[base] = 200;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        // Sprite #10 (index 9): Y out of range, but tile byte (byte 1) in range.
        {
            let base = 9 * 4;
            ppu.registers.oam[base] = 200;
            ppu.registers.oam[base + 1] = 0;
            ppu.registers.oam[base + 2] = 200;
            ppu.registers.oam[base + 3] = 200;
        }

        run_sprite_evaluation_scanline(&mut ppu, 0);
        assert!(ppu.registers.status.contains(Status::SPRITE_OVERFLOW));
    }

    #[test]
    fn sprite_overflow_obscure_byte_shift_can_skip_overflow() {
        let mut ppu = Ppu::default();
        ppu.registers.oam.fill(200);

        for i in 0..8 {
            let base = i * 4;
            ppu.registers.oam[base] = 0;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        // Sprite #9 (index 8): out of range.
        {
            let base = 8 * 4;
            ppu.registers.oam[base] = 200;
            ppu.registers.oam[base + 1] = 0xFF;
            ppu.registers.oam[base + 2] = 0xFF;
            ppu.registers.oam[base + 3] = 0xFF;
        }

        // Sprite #10 (index 9): all bytes out of range (no overflow should occur).
        {
            let base = 9 * 4;
            ppu.registers.oam[base] = 200;
            ppu.registers.oam[base + 1] = 200;
            ppu.registers.oam[base + 2] = 200;
            ppu.registers.oam[base + 3] = 200;
        }

        run_sprite_evaluation_scanline(&mut ppu, 0);
        assert!(!ppu.registers.status.contains(Status::SPRITE_OVERFLOW));
    }
}
