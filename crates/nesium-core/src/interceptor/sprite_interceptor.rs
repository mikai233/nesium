//! Sprite viewer interceptor - captures OAM and sprite pattern data.

use crate::{bus::CpuBus, cpu::Cpu, interceptor::Interceptor, memory::ppu as ppu_mem};

pub use crate::interceptor::capture_point::CapturePoint;

/// Snapshot of PPU state for the Sprite Viewer.
#[derive(Debug, Clone)]
pub struct SpriteSnapshot {
    /// OAM (Object Attribute Memory) - 256 bytes, 64 sprites × 4 bytes each.
    pub oam: Vec<u8>,
    /// CHR data (pattern tables) - 8 KB.
    pub chr: Vec<u8>,
    /// Palette RAM - 32 bytes.
    pub palette: [u8; 32],
    /// Whether 8x16 sprite mode is active (PPU CTRL bit 5).
    pub large_sprites: bool,
    /// Sprite pattern table base ($0000 or $1000, only for 8x8 mode).
    pub sprite_pattern_base: u16,
}

/// Interceptor that captures OAM and sprite pattern data for sprite viewing.
#[derive(Debug, Default)]
pub struct SpriteInterceptor {
    capture_point: CapturePoint,
    snapshot: Option<SpriteSnapshot>,
}

impl SpriteInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_snapshot(&mut self) -> Option<SpriteSnapshot> {
        self.snapshot.take()
    }

    pub fn set_capture_point(&mut self, point: CapturePoint) {
        self.capture_point = point;
    }

    fn capture(&mut self, bus: &mut CpuBus) {
        let ppu = &mut *bus.ppu;
        let oam = ppu.registers.oam.as_slice().to_vec();
        let palette = *ppu.palette_ram.as_slice().try_into().unwrap_or(&[0; 32]);

        let mut chr = vec![0u8; ppu_mem::CHR_SIZE];
        if let Some(cart) = bus.cartridge.as_deref() {
            for (offset, byte) in chr.iter_mut().enumerate() {
                *byte = cart.chr_read(offset as u16);
            }
        }

        self.snapshot = Some(SpriteSnapshot {
            oam,
            chr,
            palette,
            large_sprites: ppu.registers.control.use_8x16_sprites(),
            sprite_pattern_base: ppu.registers.control.sprite_pattern_table(),
        });
    }
}

impl Interceptor for SpriteInterceptor {
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
