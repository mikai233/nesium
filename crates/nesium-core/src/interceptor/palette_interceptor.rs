//! Palette viewer interceptor - captures PPU palette RAM for palette display.

use crate::{bus::CpuBus, cpu::Cpu, interceptor::Interceptor};

pub use crate::interceptor::capture_point::CapturePoint;

/// Snapshot of PPU palette RAM for the Palette Viewer.
#[derive(Debug, Clone)]
pub struct PaletteSnapshot {
    /// Palette RAM - 32 bytes.
    pub palette: [u8; 32],
}

/// Interceptor that captures PPU palette RAM for palette viewing.
#[derive(Debug, Default)]
pub struct PaletteInterceptor {
    capture_point: CapturePoint,
    snapshot: Option<PaletteSnapshot>,
}

impl PaletteInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_snapshot(&mut self) -> Option<PaletteSnapshot> {
        self.snapshot.take()
    }

    pub fn set_capture_point(&mut self, point: CapturePoint) {
        self.capture_point = point;
    }

    fn capture(&mut self, bus: &mut CpuBus) {
        let ppu = &mut *bus.ppu;
        let palette = *ppu.palette_ram.as_slice().try_into().unwrap_or(&[0; 32]);

        self.snapshot = Some(PaletteSnapshot { palette });
    }
}

impl Interceptor for PaletteInterceptor {
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
