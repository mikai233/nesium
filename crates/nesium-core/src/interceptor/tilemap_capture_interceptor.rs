use crate::{
    bus::CpuBus, cartridge::header::Mirroring, cpu::Cpu, interceptor::Interceptor,
    memory::ppu as ppu_mem,
};

#[derive(Debug, Clone)]
pub struct DebugTilemapData {
    pub ciram: Vec<u8>,
    pub palette: [u8; 32],
    pub chr: Vec<u8>,
    pub mirroring: Mirroring,
    pub bg_pattern_base: u16,
    pub vram_addr: u16,
    pub fine_x: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilemapCapturePoint {
    Disabled,
    FrameStart,
    VblankStart,
    ScanlineDot { scanline: i16, dot: u16 },
}

impl Default for TilemapCapturePoint {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Debug, Default)]
pub struct TilemapCaptureInterceptor {
    capture_point: TilemapCapturePoint,
    snapshot: Option<DebugTilemapData>,
}

impl TilemapCaptureInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take_snapshot(&mut self) -> Option<DebugTilemapData> {
        self.snapshot.take()
    }

    pub fn set_capture_point(&mut self, point: TilemapCapturePoint) {
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

        let bg_pattern_base = ppu.registers.control.background_pattern_table();
        let vram_addr = ppu.registers.vram.v.raw();
        let fine_x = ppu.registers.vram.x;

        self.snapshot = Some(DebugTilemapData {
            ciram,
            palette,
            chr,
            mirroring,
            bg_pattern_base,
            vram_addr,
            fine_x,
        });
    }
}

impl Interceptor for TilemapCaptureInterceptor {
    fn debug(&self, _cpu: &mut Cpu, _bus: &mut CpuBus) {}

    fn on_ppu_frame_start(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus) {
        if matches!(self.capture_point, TilemapCapturePoint::FrameStart) {
            self.capture(bus);
        }
    }

    fn on_ppu_vblank_start(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus) {
        if matches!(self.capture_point, TilemapCapturePoint::VblankStart) {
            self.capture(bus);
        }
    }

    fn on_ppu_scanline_dot(&mut self, _cpu: &mut Cpu, bus: &mut CpuBus, scanline: i16, dot: u16) {
        if let TilemapCapturePoint::ScanlineDot {
            scanline: target_scanline,
            dot: target_dot,
        } = self.capture_point
        {
            if scanline == target_scanline && dot == target_dot {
                self.capture(bus);
            }
        }
    }
}
