use std::fmt::Display;

use crate::{
    Nes,
    apu::Apu,
    audio::mixer::MixerState,
    bus::{PendingDma, savestate::OpenBusState},
    cartridge::{
        Cartridge,
        mapper::{Mapper1, Mapper2, Mapper3, Mapper4, mapper_downcast_mut, mapper_downcast_ref},
    },
    controller::ControllerPorts,
    cpu::{Cpu, IrqSource, Status as CpuStatus},
    ppu::{
        Control, Mask, PendingVramIncrement, Ppu, SpriteLineBuffers, Status,
        savestate::{
            BgPipelineState, PendingVramIncrementState, PpuOpenBusState, SpriteEvalState,
            SpriteFetchState, SpriteLineBuffersState, SpritePipelineState,
        },
    },
    state::{SaveState, Snapshot, SnapshotMeta},
};

#[cfg(feature = "savestate-serde")]
use serde::{Deserialize, Serialize};

/// Errors raised when capturing/restoring a full NES save state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NesSaveStateError {
    NoCartridge,
    UnsupportedMapper { mapper_id: u16 },
    CorruptState(&'static str),
}

impl Display for NesSaveStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NesSaveStateError::NoCartridge => write!(f, "no cartridge loaded"),
            NesSaveStateError::UnsupportedMapper { mapper_id } => {
                write!(f, "unsupported mapper ID: {}", mapper_id)
            }
            NesSaveStateError::CorruptState(msg) => write!(f, "corrupt state: {}", msg),
        }
    }
}

impl std::error::Error for NesSaveStateError {}

/// Serializable snapshot of the CPU core.
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    /// Raw status register bits (NV-BDIZC).
    pub p: u8,
    pub pc: u16,
    pub opcode_in_flight: Option<u8>,
    pub step: u8,
    pub tmp: u8,
    pub effective_addr: u16,
    pub irq_latch: u8,
    pub prev_irq_active: bool,
    pub irq_active: bool,
    pub irq_enable_mask: u8,
    pub prev_nmi_level: bool,
    pub nmi_latch: bool,
    pub prev_nmi_latch: bool,
    pub dma: CpuDmaState,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuDmaState {
    pub halt_needed: bool,
    pub dummy_read_needed: bool,
    pub dmc_active: bool,
    pub dmc_abort_pending: bool,
    pub dmc_addr: u16,
    pub is_dmc_read: bool,
    pub oam_active: bool,
    pub oam_page: u8,
    pub oam_cycle_counter: u16,
    pub oam_latch: u8,
}

/// Serializable snapshot of the PPU core (excludes the framebuffer planes / callbacks).
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PpuState {
    pub control: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_addr: u8,
    pub oam: Vec<u8>,
    pub vram_v: u16,
    pub vram_t: u16,
    pub vram_x: u8,
    pub vram_w: bool,
    pub vram_buffer: u8,
    pub pending_vram_addr: u16,
    pub pending_vram_delay: u8,
    pub ciram: Vec<u8>,
    pub palette_ram: Vec<u8>,
    pub cycle: u16,
    pub scanline: i16,
    pub frame: u32,
    pub master_clock: u64,
    pub bg_pipeline: BgPipelineState,
    pub sprite_pipeline: SpritePipelineState,
    pub nmi_level: bool,
    pub prevent_vblank_flag: bool,
    pub open_bus: PpuOpenBusState,
    pub ignore_vram_read: u8,
    pub oam_copybuffer: u8,
    pub pending_vram_increment: PendingVramIncrementState,
    pub secondary_oam: Vec<u8>,
    pub sprite_eval: SpriteEvalState,
    pub sprite_fetch: SpriteFetchState,
    pub sprite_line_next: SpriteLineBuffersState,
    pub render_enabled: bool,
    pub prev_render_enabled: bool,
    pub oam_addr_disable_glitch_pending: bool,
    pub corrupt_oam_row: [bool; 32],
    pub state_update_pending: bool,
}

/// Save-state snapshot of the cartridge and mapper (ROM data not included).
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartridgeState {
    pub mapper_id: u16,
    pub submapper: u8,
    pub prg_ram: Option<Vec<u8>>,
    pub prg_work_ram: Option<Vec<u8>>,
    pub chr_ram: Option<Vec<u8>>,
    pub chr_battery_ram: Option<Vec<u8>>,
    pub mapper_ram: Option<Vec<u8>>,
    pub mapper: MapperState,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapperState {
    Mapper0,
    Mapper1(crate::cartridge::mapper::mapper1::Mapper1State),
    Mapper2(crate::cartridge::mapper::mapper2::Mapper2State),
    Mapper3(crate::cartridge::mapper::mapper3::Mapper3State),
    Mapper4(crate::cartridge::mapper::mapper4::Mapper4State),
}

/// Full deterministic emulator snapshot (suitable for save/load and rewind).
///
/// Notes:
/// - This does not include the ROM image. Callers must ensure the same ROM is loaded.
/// - PPU framebuffer configuration/planes are preserved; only PPU internal state is restored.
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NesState {
    pub cpu: CpuState,
    pub ppu: PpuState,
    pub apu: Apu,
    pub ram: Vec<u8>,
    pub cartridge: CartridgeState,
    pub controllers: ControllerPorts,
    pub last_frame: u32,
    pub dot_counter: u64,
    pub master_clock: u64,
    pub ppu_offset: u8,
    pub clock_start_count: u8,
    pub clock_end_count: u8,
    pub pending_dma: PendingDma,
    pub open_bus: OpenBusState,
    pub cycles: u64,
    pub mixer: MixerState,
}

/// Delta snapshots are not yet optimized; we currently store full snapshots for rewind.
pub type NesDeltaState = NesState;

/// Convenience alias for a full snapshot including metadata.
pub type NesSnapshot = Snapshot<NesState, SnapshotMeta>;

impl Nes {
    pub fn save_snapshot(&self, meta: SnapshotMeta) -> Result<NesSnapshot, NesSaveStateError> {
        <Self as SaveState>::save(self, meta)
    }

    pub fn load_snapshot(&mut self, snapshot: &NesSnapshot) -> Result<(), NesSaveStateError> {
        <Self as SaveState>::load(self, snapshot)
    }
}

impl SaveState for Nes {
    type State = NesState;
    type Error = NesSaveStateError;
    type Meta = SnapshotMeta;

    fn save(&self, mut meta: Self::Meta) -> Result<Snapshot<Self::State, Self::Meta>, Self::Error> {
        if let Some(cart) = self.cartridge.as_ref() {
            meta.format_version = Self::FORMAT_VERSION;
            let state = NesState {
                cpu: cpu_to_state(&self.cpu),
                ppu: ppu_to_state(&self.ppu),
                apu: self.apu.clone(),
                ram: self.ram.as_slice().to_vec(),
                cartridge: cartridge_to_state(cart)?,
                controllers: self.controllers.clone(),
                last_frame: self.last_frame,
                dot_counter: self.dot_counter,
                master_clock: self.master_clock,
                ppu_offset: self.ppu_offset,
                clock_start_count: self.clock_start_count,
                clock_end_count: self.clock_end_count,
                pending_dma: self.pending_dma,
                open_bus: OpenBusState::from_open_bus(self.open_bus),
                cycles: self.cycles,
                mixer: self.mixer.save_state(),
            };
            Ok(Snapshot { meta, data: state })
        } else {
            Err(NesSaveStateError::NoCartridge)
        }
    }

    fn load(&mut self, snapshot: &Snapshot<Self::State, Self::Meta>) -> Result<(), Self::Error> {
        if self.cartridge.is_none() {
            return Err(NesSaveStateError::NoCartridge);
        }

        // Validate mapper compatibility if metadata is present.
        if let Some((expected_mapper, _)) = snapshot.meta.mapper {
            let current_mapper = self.cartridge.as_ref().unwrap().header().mapper();
            if expected_mapper != current_mapper {
                return Err(NesSaveStateError::CorruptState(
                    "mapper mismatch between snapshot and loaded ROM",
                ));
            }
        }

        let state = &snapshot.data;

        state_to_cpu(&mut self.cpu, &state.cpu);
        state_to_ppu(&mut self.ppu, &state.ppu)?;
        self.apu = state.apu.clone();
        if self.ram.as_slice().len() != state.ram.len() {
            return Err(NesSaveStateError::CorruptState("ram size mismatch"));
        }
        self.ram.as_mut_slice().copy_from_slice(&state.ram);

        if let Some(cart) = self.cartridge.as_mut() {
            apply_cartridge_state(cart, &state.cartridge)?;
        }

        self.controllers = state.controllers.clone();
        self.last_frame = state.last_frame;
        self.dot_counter = state.dot_counter;
        self.master_clock = state.master_clock;
        self.ppu_offset = state.ppu_offset;
        self.clock_start_count = state.clock_start_count;
        self.clock_end_count = state.clock_end_count;
        self.pending_dma = state.pending_dma;
        state.open_bus.apply_to(&mut self.open_bus);
        self.cycles = state.cycles;

        self.mixer.load_state(state.mixer.clone());
        self.sound_bus.reset();
        self.mixer_frame_buffer.clear();
        Ok(())
    }
}

fn ppu_to_state(ppu: &Ppu) -> PpuState {
    let regs = &ppu.registers;
    PpuState {
        control: regs.control.bits(),
        mask: regs.mask.bits(),
        status: regs.status.bits(),
        oam_addr: regs.oam_addr,
        oam: regs.oam.as_slice().to_vec(),
        vram_v: regs.vram.v.raw(),
        vram_t: regs.vram.t.raw(),
        vram_x: regs.vram.x,
        vram_w: regs.vram.w,
        vram_buffer: regs.vram_buffer,
        pending_vram_addr: ppu.pending_vram_addr.raw(),
        pending_vram_delay: ppu.pending_vram_delay,
        ciram: ppu.ciram.as_slice().to_vec(),
        palette_ram: ppu.palette_ram.as_slice().to_vec(),
        cycle: ppu.cycle,
        scanline: ppu.scanline,
        frame: ppu.frame,
        master_clock: ppu.master_clock,
        bg_pipeline: ppu.bg_pipeline.save_state(),
        sprite_pipeline: ppu.sprite_pipeline.save_state(),
        nmi_level: ppu.nmi_level,
        prevent_vblank_flag: ppu.prevent_vblank_flag,
        open_bus: ppu.open_bus.save_state(),
        ignore_vram_read: ppu.ignore_vram_read,
        oam_copybuffer: ppu.oam_copybuffer,
        pending_vram_increment: match ppu.pending_vram_increment {
            PendingVramIncrement::None => PendingVramIncrementState::none(),
            PendingVramIncrement::By1 => PendingVramIncrementState::by1(),
            PendingVramIncrement::By32 => PendingVramIncrementState::by32(),
        },
        secondary_oam: ppu.secondary_oam.as_slice().to_vec(),
        sprite_eval: SpriteEvalState {
            sprite_addr_h: ppu.sprite_eval.sprite_addr_h,
            sprite_addr_l: ppu.sprite_eval.sprite_addr_l,
            secondary_oam_addr: ppu.sprite_eval.secondary_oam_addr,
            sprite_in_range: ppu.sprite_eval.sprite_in_range,
            oam_copy_done: ppu.sprite_eval.oam_copy_done,
            overflow_bug_counter: ppu.sprite_eval.overflow_bug_counter,
            sprite0_in_range_next: ppu.sprite_eval.sprite0_in_range_next,
            count: ppu.sprite_eval.count,
        },
        sprite_fetch: SpriteFetchState {
            i: ppu.sprite_fetch.i,
            sub: ppu.sprite_fetch.sub,
        },
        sprite_line_next: sprite_line_buffers_to_state(&ppu.sprite_line_next),
        render_enabled: ppu.render_enabled,
        prev_render_enabled: ppu.prev_render_enabled,
        oam_addr_disable_glitch_pending: ppu.oam_addr_disable_glitch_pending,
        corrupt_oam_row: ppu.corrupt_oam_row,
        state_update_pending: ppu.state_update_pending,
    }
}

fn state_to_ppu(ppu: &mut Ppu, state: &PpuState) -> Result<(), NesSaveStateError> {
    ppu.registers.control = Control::from_bits_retain(state.control);
    ppu.registers.mask = Mask::from_bits_retain(state.mask);
    ppu.registers.status = Status::from_bits_retain(state.status);
    ppu.registers.oam_addr = state.oam_addr;
    if ppu.registers.oam.as_slice().len() != state.oam.len() {
        return Err(NesSaveStateError::CorruptState("ppu oam size mismatch"));
    }
    ppu.registers.oam.as_mut_slice().copy_from_slice(&state.oam);
    ppu.registers.vram.v.set_raw(state.vram_v);
    ppu.registers.vram.t.set_raw(state.vram_t);
    ppu.registers.vram.x = state.vram_x;
    ppu.registers.vram.w = state.vram_w;
    ppu.registers.vram_buffer = state.vram_buffer;

    ppu.pending_vram_addr.set_raw(state.pending_vram_addr);
    ppu.pending_vram_delay = state.pending_vram_delay;

    if ppu.ciram.as_slice().len() != state.ciram.len() {
        return Err(NesSaveStateError::CorruptState("ppu ciram size mismatch"));
    }
    ppu.ciram.as_mut_slice().copy_from_slice(&state.ciram);
    if ppu.palette_ram.as_slice().len() != state.palette_ram.len() {
        return Err(NesSaveStateError::CorruptState(
            "ppu palette ram size mismatch",
        ));
    }
    ppu.palette_ram
        .as_mut_slice()
        .copy_from_slice(&state.palette_ram);

    ppu.cycle = state.cycle;
    ppu.scanline = state.scanline;
    ppu.frame = state.frame;
    ppu.master_clock = state.master_clock;
    ppu.bg_pipeline.load_state(state.bg_pipeline);
    ppu.sprite_pipeline
        .load_state(state.sprite_pipeline.clone());
    ppu.nmi_level = state.nmi_level;
    ppu.prevent_vblank_flag = state.prevent_vblank_flag;
    ppu.open_bus.load_state(state.open_bus);
    ppu.ignore_vram_read = state.ignore_vram_read;
    ppu.oam_copybuffer = state.oam_copybuffer;
    ppu.pending_vram_increment = match state.pending_vram_increment.0 {
        1 => PendingVramIncrement::By1,
        32 => PendingVramIncrement::By32,
        _ => PendingVramIncrement::None,
    };

    if ppu.secondary_oam.as_slice().len() != state.secondary_oam.len() {
        return Err(NesSaveStateError::CorruptState(
            "ppu secondary oam size mismatch",
        ));
    }
    ppu.secondary_oam
        .as_mut_slice()
        .copy_from_slice(&state.secondary_oam);

    ppu.sprite_eval.sprite_addr_h = state.sprite_eval.sprite_addr_h;
    ppu.sprite_eval.sprite_addr_l = state.sprite_eval.sprite_addr_l;
    ppu.sprite_eval.secondary_oam_addr = state.sprite_eval.secondary_oam_addr;
    ppu.sprite_eval.sprite_in_range = state.sprite_eval.sprite_in_range;
    ppu.sprite_eval.oam_copy_done = state.sprite_eval.oam_copy_done;
    ppu.sprite_eval.overflow_bug_counter = state.sprite_eval.overflow_bug_counter;
    ppu.sprite_eval.sprite0_in_range_next = state.sprite_eval.sprite0_in_range_next;
    ppu.sprite_eval.count = state.sprite_eval.count;

    ppu.sprite_fetch.i = state.sprite_fetch.i;
    ppu.sprite_fetch.sub = state.sprite_fetch.sub;

    sprite_line_buffers_from_state(&mut ppu.sprite_line_next, &state.sprite_line_next);

    ppu.render_enabled = state.render_enabled;
    ppu.prev_render_enabled = state.prev_render_enabled;
    ppu.oam_addr_disable_glitch_pending = state.oam_addr_disable_glitch_pending;
    ppu.corrupt_oam_row = state.corrupt_oam_row;
    ppu.state_update_pending = state.state_update_pending;
    Ok(())
}

fn sprite_line_buffers_to_state(buffers: &SpriteLineBuffers) -> SpriteLineBuffersState {
    let mut state = SpriteLineBuffersState::default();
    state.y.copy_from_slice(buffers.y.as_slice());
    state.tile.copy_from_slice(buffers.tile.as_slice());
    state.attr.copy_from_slice(buffers.attr.as_slice());
    state.x.copy_from_slice(buffers.x.as_slice());
    state
        .pattern_low
        .copy_from_slice(buffers.pattern_low.as_slice());
    state
        .pattern_high
        .copy_from_slice(buffers.pattern_high.as_slice());
    state
}

fn sprite_line_buffers_from_state(buffers: &mut SpriteLineBuffers, state: &SpriteLineBuffersState) {
    buffers.y.as_mut_slice().copy_from_slice(&state.y);
    buffers.tile.as_mut_slice().copy_from_slice(&state.tile);
    buffers.attr.as_mut_slice().copy_from_slice(&state.attr);
    buffers.x.as_mut_slice().copy_from_slice(&state.x);
    buffers
        .pattern_low
        .as_mut_slice()
        .copy_from_slice(&state.pattern_low);
    buffers
        .pattern_high
        .as_mut_slice()
        .copy_from_slice(&state.pattern_high);
}

#[cfg(feature = "savestate-postcard")]
impl NesState {
    pub fn to_postcard_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    pub fn from_postcard_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(feature = "savestate-postcard")]
impl Snapshot<NesState, SnapshotMeta> {
    pub fn to_postcard_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }

    pub fn from_postcard_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

#[cfg(all(test, feature = "savestate-postcard"))]
mod tests {
    use super::*;
    use crate::{cartridge, controller::Button, ppu::buffer::ColorFormat};

    fn dummy_nrom_rom() -> Vec<u8> {
        // iNES header + 16 KiB PRG + 8 KiB CHR.
        let mut rom = Vec::with_capacity(16 + 16 * 1024 + 8 * 1024);
        rom.extend_from_slice(b"NES\x1A");
        rom.push(1); // PRG: 1 * 16 KiB
        rom.push(1); // CHR: 1 * 8 KiB
        rom.push(0); // flags 6 (mapper 0, horizontal mirroring)
        rom.push(0); // flags 7
        rom.extend_from_slice(&[0; 8]); // rest of header
        rom.extend_from_slice(&vec![0u8; 16 * 1024]);
        rom.extend_from_slice(&vec![0u8; 8 * 1024]);
        rom
    }

    #[test]
    fn savestate_postcard_roundtrip() {
        let cart = cartridge::load_cartridge(dummy_nrom_rom()).expect("load dummy cartridge");

        let mut nes = crate::Nes::new(ColorFormat::Rgb555);
        nes.insert_cartridge(cart.clone());

        nes.set_button(0, Button::A, true);
        nes.run_frame(false);
        nes.run_frame(false);

        let meta = SnapshotMeta {
            tick: 123,
            ..SnapshotMeta::default()
        };
        let snap = <crate::Nes as SaveState>::save(&nes, meta).expect("save snapshot");
        let bytes = snap.to_postcard_bytes().expect("encode snapshot");

        let decoded =
            Snapshot::<NesState, SnapshotMeta>::from_postcard_bytes(&bytes).expect("decode");

        let mut nes2 = crate::Nes::new(ColorFormat::Rgb555);
        nes2.insert_cartridge(cart);
        <crate::Nes as SaveState>::load(&mut nes2, &decoded).expect("load snapshot");

        let snap2 =
            <crate::Nes as SaveState>::save(&nes2, decoded.meta.clone()).expect("save again");
        let bytes2 = snap2.to_postcard_bytes().expect("encode again");
        assert_eq!(bytes, bytes2);
    }
}

fn cpu_to_state(cpu: &Cpu) -> CpuState {
    CpuState {
        a: cpu.a,
        x: cpu.x,
        y: cpu.y,
        s: cpu.s,
        p: cpu.p.bits(),
        pc: cpu.pc,
        opcode_in_flight: cpu.opcode_in_flight,
        step: cpu.step,
        tmp: cpu.tmp,
        effective_addr: cpu.effective_addr,
        irq_latch: cpu.irq_latch.bits(),
        prev_irq_active: cpu.prev_irq_active,
        irq_active: cpu.irq_active,
        irq_enable_mask: cpu.irq_enable_mask.bits(),
        prev_nmi_level: cpu.prev_nmi_level,
        nmi_latch: cpu.nmi_latch,
        prev_nmi_latch: cpu.prev_nmi_latch,
        dma: CpuDmaState {
            halt_needed: cpu.dma.halt_needed,
            dummy_read_needed: cpu.dma.dummy_read_needed,
            dmc_active: cpu.dma.dmc_active,
            dmc_abort_pending: cpu.dma.dmc_abort_pending,
            dmc_addr: cpu.dma.dmc_addr,
            is_dmc_read: cpu.dma.is_dmc_read,
            oam_active: cpu.dma.oam_active,
            oam_page: cpu.dma.oam_page,
            oam_cycle_counter: cpu.dma.oam_cycle_counter,
            oam_latch: cpu.dma.oam_latch,
        },
    }
}

fn state_to_cpu(cpu: &mut Cpu, state: &CpuState) {
    cpu.a = state.a;
    cpu.x = state.x;
    cpu.y = state.y;
    cpu.s = state.s;
    cpu.p = CpuStatus::from_bits_retain(state.p);
    cpu.pc = state.pc;
    cpu.opcode_in_flight = state.opcode_in_flight;
    cpu.step = state.step;
    cpu.tmp = state.tmp;
    cpu.effective_addr = state.effective_addr;
    cpu.irq_latch = IrqSource::from_bits_retain(state.irq_latch);
    cpu.prev_irq_active = state.prev_irq_active;
    cpu.irq_active = state.irq_active;
    cpu.irq_enable_mask = IrqSource::from_bits_retain(state.irq_enable_mask);
    cpu.prev_nmi_level = state.prev_nmi_level;
    cpu.nmi_latch = state.nmi_latch;
    cpu.prev_nmi_latch = state.prev_nmi_latch;
    cpu.dma.halt_needed = state.dma.halt_needed;
    cpu.dma.dummy_read_needed = state.dma.dummy_read_needed;
    cpu.dma.dmc_active = state.dma.dmc_active;
    cpu.dma.dmc_abort_pending = state.dma.dmc_abort_pending;
    cpu.dma.dmc_addr = state.dma.dmc_addr;
    cpu.dma.is_dmc_read = state.dma.is_dmc_read;
    cpu.dma.oam_active = state.dma.oam_active;
    cpu.dma.oam_page = state.dma.oam_page;
    cpu.dma.oam_cycle_counter = state.dma.oam_cycle_counter;
    cpu.dma.oam_latch = state.dma.oam_latch;
}

fn cartridge_to_state(cart: &Cartridge) -> Result<CartridgeState, NesSaveStateError> {
    let mapper = cart.mapper();
    let mapper_id = mapper.mapper_id();
    let submapper = cart.header().submapper();

    let mapper_state = match mapper_id {
        0 => MapperState::Mapper0,
        1 => MapperState::Mapper1(
            mapper_downcast_ref::<Mapper1>(mapper)
                .ok_or(NesSaveStateError::UnsupportedMapper { mapper_id })?
                .save_state(),
        ),
        2 => MapperState::Mapper2(
            mapper_downcast_ref::<Mapper2>(mapper)
                .ok_or(NesSaveStateError::UnsupportedMapper { mapper_id })?
                .save_state(),
        ),
        3 => MapperState::Mapper3(
            mapper_downcast_ref::<Mapper3>(mapper)
                .ok_or(NesSaveStateError::UnsupportedMapper { mapper_id })?
                .save_state(),
        ),
        4 => MapperState::Mapper4(
            mapper_downcast_ref::<Mapper4>(mapper)
                .ok_or(NesSaveStateError::UnsupportedMapper { mapper_id })?
                .save_state(),
        ),
        _ => return Err(NesSaveStateError::UnsupportedMapper { mapper_id }),
    };

    Ok(CartridgeState {
        mapper_id,
        submapper,
        prg_ram: mapper.prg_ram().map(|s| s.to_vec()),
        prg_work_ram: mapper.prg_work_ram().map(|s| s.to_vec()),
        chr_ram: mapper.chr_ram().map(|s| s.to_vec()),
        chr_battery_ram: mapper.chr_battery_ram().map(|s| s.to_vec()),
        mapper_ram: mapper.mapper_ram().map(|s| s.to_vec()),
        mapper: mapper_state,
    })
}

fn apply_cartridge_state(
    cart: &mut Cartridge,
    state: &CartridgeState,
) -> Result<(), NesSaveStateError> {
    let header_submapper = cart.header().submapper();
    let current_mapper_id = cart.mapper().mapper_id();
    if current_mapper_id != state.mapper_id {
        return Err(NesSaveStateError::CorruptState("mapper mismatch"));
    }
    if header_submapper != state.submapper {
        return Err(NesSaveStateError::CorruptState("submapper mismatch"));
    }

    let mapper = cart.mapper_mut();
    if let (Some(dst), Some(src)) = (mapper.prg_ram_mut(), state.prg_ram.as_deref()) {
        if dst.len() != src.len() {
            return Err(NesSaveStateError::CorruptState("prg_ram size mismatch"));
        }
        dst.copy_from_slice(src);
    }
    if let (Some(dst), Some(src)) = (mapper.prg_work_ram_mut(), state.prg_work_ram.as_deref()) {
        if dst.len() != src.len() {
            return Err(NesSaveStateError::CorruptState(
                "prg_work_ram size mismatch",
            ));
        }
        dst.copy_from_slice(src);
    }
    if let (Some(dst), Some(src)) = (mapper.chr_ram_mut(), state.chr_ram.as_deref()) {
        if dst.len() != src.len() {
            return Err(NesSaveStateError::CorruptState("chr_ram size mismatch"));
        }
        dst.copy_from_slice(src);
    }
    if let (Some(dst), Some(src)) = (
        mapper.chr_battery_ram_mut(),
        state.chr_battery_ram.as_deref(),
    ) {
        if dst.len() != src.len() {
            return Err(NesSaveStateError::CorruptState(
                "chr_battery_ram size mismatch",
            ));
        }
        dst.copy_from_slice(src);
    }
    if let (Some(dst), Some(src)) = (mapper.mapper_ram_mut(), state.mapper_ram.as_deref()) {
        if dst.len() != src.len() {
            return Err(NesSaveStateError::CorruptState("mapper_ram size mismatch"));
        }
        dst.copy_from_slice(src);
    }

    match &state.mapper {
        MapperState::Mapper0 => Ok(()),
        MapperState::Mapper1(s) => {
            mapper_downcast_mut::<Mapper1>(mapper)
                .ok_or(NesSaveStateError::CorruptState("mapper downcast failed"))?
                .load_state(s);
            Ok(())
        }
        MapperState::Mapper2(s) => {
            mapper_downcast_mut::<Mapper2>(mapper)
                .ok_or(NesSaveStateError::CorruptState("mapper downcast failed"))?
                .load_state(s);
            Ok(())
        }
        MapperState::Mapper3(s) => {
            mapper_downcast_mut::<Mapper3>(mapper)
                .ok_or(NesSaveStateError::CorruptState("mapper downcast failed"))?
                .load_state(s);
            Ok(())
        }
        MapperState::Mapper4(s) => {
            mapper_downcast_mut::<Mapper4>(mapper)
                .ok_or(NesSaveStateError::CorruptState("mapper downcast failed"))?
                .load_state(s);
            Ok(())
        }
    }
}
