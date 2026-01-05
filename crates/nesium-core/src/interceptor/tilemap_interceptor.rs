//! Tilemap viewer interceptor - captures PPU state for tilemap/nametable display.

use crate::{
    bus::CpuBus, cartridge::header::Mirroring, cpu::Cpu, interceptor::Interceptor,
    memory::ppu as ppu_mem,
};

pub use crate::interceptor::capture_point::CapturePoint;

/// Snapshot of PPU state for the Tilemap Viewer.
#[derive(Debug, Clone)]
pub struct TilemapSnapshot {
    /// CIRAM (nametable RAM) - 2 KB.
    pub ciram: Vec<u8>,
    /// Palette RAM - 32 bytes.
    pub palette: [u8; 32],
    /// CHR data (pattern tables) - 8 KB.
    pub chr: Vec<u8>,
    /// Current nametable mirroring mode.
    pub mirroring: Mirroring,
    /// Background pattern table base ($0000 or $1000).
    pub bg_pattern_base: u16,
    /// Current VRAM address (V register).
    pub vram_addr: u16,
    /// Temporary VRAM address (T register).
    pub temp_addr: u16,
    /// Fine X scroll (0-7).
    pub fine_x: u8,
}

/// Interceptor that captures PPU state for tilemap viewing.
#[derive(Debug, Default)]
pub struct TilemapInterceptor {
    capture_point: CapturePoint,
    snapshot: Option<TilemapSnapshot>,
}

impl TilemapInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_snapshot(&mut self) -> Option<TilemapSnapshot> {
        self.snapshot.take()
    }

    pub fn set_capture_point(&mut self, point: CapturePoint) {
        self.capture_point = point;
    }

    fn capture(&mut self, bus: &mut CpuBus) {
        let ppu = &mut *bus.ppu;
        let ciram = ppu.ciram.to_vec();
        let palette = *ppu.palette_ram.as_slice().try_into().unwrap_or(&[0; 32]);

        let mut chr = vec![0u8; ppu_mem::CHR_SIZE];
        if let Some(cart) = bus.cartridge.as_deref() {
            for (offset, byte) in chr.iter_mut().enumerate() {
                *byte = cart.chr_read(offset as u16);
            }
        }

        let mirroring = bus
            .cartridge
            .as_deref()
            .map(|cart| cart.mirroring())
            .unwrap_or(Mirroring::Horizontal);

        self.snapshot = Some(TilemapSnapshot {
            ciram,
            palette,
            chr,
            mirroring,
            bg_pattern_base: ppu.registers.control.background_pattern_table(),
            vram_addr: ppu.registers.vram.v.raw(),
            temp_addr: ppu.registers.vram.t.raw(),
            fine_x: ppu.registers.vram.x,
        });
    }
}

impl Interceptor for TilemapInterceptor {
    fn debug(&self, _cpu: &mut Cpu, _bus: &mut CpuBus) {}

    fn on_ppu_frame_start(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus) {
        if self.capture_point.should_capture_on_frame_start() {
            self.capture(bus);
        }
    }

    fn on_ppu_vblank_start(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus) {
        if self.capture_point.should_capture_on_vblank_start() {
            self.capture(bus);
        }
    }

    fn on_ppu_scanline_dot(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus, scanline: i16, dot: u16) {
        if self
            .capture_point
            .should_capture_on_scanline_dot(scanline, dot)
        {
            self.capture(bus);
        }
    }
}
