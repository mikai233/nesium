//! Cartridge mapper registry, traits, and shared helpers.
//!
//! This module wires together the concrete mapper implementations, defines the
//! core [`Mapper`] trait they implement, and exposes a few small helpers for
//! PRG RAM allocation and trainer placement that are reused across mappers.

use std::{any::Any, borrow::Cow, fmt::Debug};

use dyn_clone::DynClone;

use crate::{apu::ExpansionAudio, reset_kind::ResetKind};

pub mod chr_storage;
pub mod mapper0;
pub mod mapper1;
pub mod mapper10;
pub mod mapper11;
pub mod mapper119;
pub mod mapper13;
pub mod mapper19;
pub mod mapper2;
pub mod mapper21;
pub mod mapper228;
pub mod mapper23;
pub mod mapper25;
pub mod mapper26;
pub mod mapper3;
pub mod mapper34;
pub mod mapper4;
pub mod mapper5;
pub mod mapper6;
pub mod mapper66;
pub mod mapper7;
pub mod mapper71;
pub mod mapper78;
pub mod mapper8;
pub mod mapper85;
pub mod mapper9;
pub mod mapper90;
pub mod provider;

pub use chr_storage::{ChrStorage, select_chr_storage};
pub use mapper0::Mapper0;
pub use mapper1::Mapper1;
pub use mapper2::Mapper2;
pub use mapper3::Mapper3;
pub use mapper4::Mapper4;
pub use mapper5::Mapper5;
pub use mapper6::Mapper6;
pub use mapper7::Mapper7;
pub use mapper8::Mapper8;
pub use mapper9::Mapper9;
pub use mapper10::Mapper10;
pub use mapper11::Mapper11;
pub use mapper13::Mapper13;
pub use mapper19::Mapper19;
pub use mapper21::Mapper21;
pub use mapper23::Mapper23;
pub use mapper25::Mapper25;
pub use mapper26::Mapper26;
pub use mapper34::Mapper34;
pub use mapper66::Mapper66;
pub use mapper71::Mapper71;
pub use mapper78::Mapper78;
pub use mapper85::Mapper85;
pub use mapper90::Mapper90;
pub use mapper119::Mapper119;
pub use mapper228::Mapper228;
pub use provider::Provider;

use crate::{
    cartridge::{
        TRAINER_SIZE, TrainerBytes,
        header::{Header, Mirroring, RomFormat},
    },
    memory::cpu as cpu_mem,
};

/// CPU address at which the optional 512 byte trainer is mapped into PRG RAM.
const TRAINER_BASE_ADDR: u16 = 0x7000;
/// Offset of the trainer region within the PRG RAM window.
const TRAINER_RAM_OFFSET: usize = (TRAINER_BASE_ADDR - cpu_mem::PRG_RAM_START) as usize;

/// Categorises the source of a PPU VRAM access.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PpuVramAccessKind {
    /// PPU background/sprite pipelines performing pattern/nametable fetches.
    RenderingFetch,
    /// CPU-driven VRAM read via `$2007`.
    CpuRead,
    /// CPU-driven VRAM write via `$2007`.
    CpuWrite,
    /// Any other source (e.g. debugger, test harness).
    Other,
}

/// Operation classification aligned with Mesen2 `MemoryOperationType`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MapperMemoryOperation {
    Read,
    Write,
    ExecOpcode,
    ExecOperand,
    DmaRead,
    DmaWrite,
    DummyRead,
    DummyWrite,
    PpuRenderingRead,
    Idle,
}

impl PpuVramAccessKind {
    pub const fn operation(self) -> MapperMemoryOperation {
        match self {
            PpuVramAccessKind::RenderingFetch => MapperMemoryOperation::PpuRenderingRead,
            PpuVramAccessKind::CpuRead => MapperMemoryOperation::Read,
            PpuVramAccessKind::CpuWrite => MapperMemoryOperation::Write,
            PpuVramAccessKind::Other => MapperMemoryOperation::Idle,
        }
    }
}

/// Fine-grained source of a PPU address-bus event.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PpuVramAccessSource {
    /// PPU rendering pipeline fetches (background/sprite).
    RenderingFetch,
    /// CPU write to `$2006` (VRAM address latch update).
    CpuAddrWrite,
    /// CPU read from `$2007`.
    CpuDataRead,
    /// CPU write to `$2007`.
    CpuDataWrite,
    /// Delayed `$2007` auto-increment bus update.
    CpuDataIncrement,
    /// Any other source.
    Other,
}

/// Rendering pipeline target for PPU fetches.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PpuRenderFetchTarget {
    Background,
    Sprite,
    Other,
}

/// Rendering fetch phase for PPU memory reads.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PpuRenderFetchType {
    Nametable,
    Attribute,
    PatternLow,
    PatternHigh,
    Other,
}

/// Extra rendering metadata attached to rendering VRAM accesses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PpuRenderFetchInfo {
    pub target: PpuRenderFetchTarget,
    pub fetch: PpuRenderFetchType,
    pub tile_x: Option<u8>,
    pub tile_y: Option<u8>,
    pub sprite_index: Option<u8>,
}

/// Rich timing/context information for a PPU VRAM access.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PpuVramAccessContext {
    /// Internal PPU dot counter (monotonic across frames).
    pub ppu_cycle: u64,
    /// Global CPU bus cycle counter (used for M2-based gating).
    pub cpu_cycle: u64,
    /// High-level classification of this VRAM access.
    pub kind: PpuVramAccessKind,
    /// Fine-grained source of this access.
    pub source: PpuVramAccessSource,
    /// PPU master clock (4 master clocks per dot).
    pub ppu_master_clock: u64,
    /// Current scanline (`-1..=260` on NTSC).
    pub ppu_scanline: i16,
    /// Current dot (`0..=340`).
    pub ppu_dot: u16,
    /// Optional rendering-fetch metadata (background/sprite + fetch phase).
    pub render_fetch: Option<PpuRenderFetchInfo>,
}

bitflags::bitflags! {
    /// Declares which event hooks a mapper wants to receive.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MapperHookMask: u8 {
        const NONE = 0;
        /// Receive explicit CPU bus access events (read/write/DMA).
        const CPU_BUS_ACCESS = 1 << 0;
        /// Receive PPU address-bus change events.
        const PPU_BUS_ADDRESS = 1 << 1;
        /// Receive final PPU VRAM read-value override callback.
        const PPU_READ_OVERRIDE = 1 << 2;
    }
}

/// CPU bus access type observed by the mapper hook system.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CpuBusAccessKind {
    Read,
    Write,
    DmaRead,
    DmaWrite,
    ExecOpcode,
    ExecOperand,
    DummyRead,
    DummyWrite,
    Idle,
}

impl CpuBusAccessKind {
    pub const fn operation(self) -> MapperMemoryOperation {
        match self {
            CpuBusAccessKind::Read => MapperMemoryOperation::Read,
            CpuBusAccessKind::Write => MapperMemoryOperation::Write,
            CpuBusAccessKind::DmaRead => MapperMemoryOperation::DmaRead,
            CpuBusAccessKind::DmaWrite => MapperMemoryOperation::DmaWrite,
            CpuBusAccessKind::ExecOpcode => MapperMemoryOperation::ExecOpcode,
            CpuBusAccessKind::ExecOperand => MapperMemoryOperation::ExecOperand,
            CpuBusAccessKind::DummyRead => MapperMemoryOperation::DummyRead,
            CpuBusAccessKind::DummyWrite => MapperMemoryOperation::DummyWrite,
            CpuBusAccessKind::Idle => MapperMemoryOperation::Idle,
        }
    }
}

/// Unified mapper event stream used by the new bus notification path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MapperEvent {
    /// CPU bus access event with cycle timing.
    CpuBusAccess {
        kind: CpuBusAccessKind,
        addr: u16,
        value: u8,
        cpu_cycle: u64,
        master_clock: u64,
    },
    /// PPU address-bus update event.
    PpuBusAddress {
        addr: u16,
        ctx: PpuVramAccessContext,
    },
}

/// Target backing store for a PPU nametable address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NametableTarget {
    /// Use PPU CIRAM (internal 2 KiB VRAM). `u16` is CIRAM offset.
    Ciram(u16),
    /// Use mapper-controlled VRAM/ROM. `u16` is mapper-local offset.
    MapperVram(u16),
    /// No device drives the bus (open bus).
    None,
}

/// Core mapper interface implemented by all cartridge boards.
///
/// Boards that expose extra sound channels can additionally implement
/// [`ExpansionAudio`] and opt into the optional `as_expansion_audio`/`mut`
/// hooks below so the core can treat expansion audio generically.
pub trait Mapper: Debug + Send + DynClone + Any + 'static {
    /// Returns the CPU-visible byte for `addr`, or `None` when the bus should
    /// float (open-bus behavior) because the addressed resource is disabled or
    /// absent.
    fn cpu_read(&self, addr: u16) -> Option<u8>;

    /// CPU write with cycle count for timing-sensitive mappers.
    fn cpu_write(&mut self, addr: u16, data: u8, cpu_cycle: u64);

    /// Declares which unified mapper events this mapper consumes.
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::NONE
    }

    /// Unified mapper event callback.
    fn on_mapper_event(&mut self, _event: MapperEvent) {}

    /// Final value filter for PPU VRAM reads (`$0000-$3EFF`).
    ///
    /// This is intended for boards that need to override returned PPU read
    /// data depending on rendering phase/state (e.g. MMC5/JY/Rainbow-style
    /// behaviour). The default implementation keeps `value` unchanged.
    fn ppu_read_override(&mut self, _addr: u16, _ctx: PpuVramAccessContext, value: u8) -> u8 {
        value
    }

    /// Returns this mapper as an expansion audio source, when supported.
    ///
    /// The default implementation returns `None`, meaning the board does not
    /// provide any extra audio channels beyond the core APU.
    fn as_expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        None
    }

    /// Mutable variant of [`as_expansion_audio`](Self::as_expansion_audio).
    ///
    /// Mappers that implement [`ExpansionAudio`] typically return
    /// `Some(self)` here; boards without expansion audio keep the
    /// default `None` implementation.
    fn as_expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        None
    }

    /// Applies either a power-on style reset (cold boot) or a soft reset,
    /// depending on `kind`. Default implementation does nothing.
    fn reset(&mut self, _kind: ResetKind) {}

    /// PPU-side read for CHR/VRAM space (`$0000-$1FFF`).
    ///
    /// Returns `Some(byte)` when the mapper drives the PPU data bus, or `None`
    /// when the bus should float (open-bus) and the core PPU logic should
    /// reuse the previous bus value.
    fn ppu_read(&self, addr: u16) -> Option<u8>;

    fn ppu_write(&mut self, addr: u16, data: u8);

    /// Convenience method for CHR pattern table reads (`$0000-$1FFF`).
    ///
    /// Unlike [`ppu_read`], this always returns a byte (no open-bus Option).
    /// Mappers should return actual CHR data; the default falls back to 0.
    fn chr_read(&self, addr: u16) -> u8 {
        self.ppu_read(addr).unwrap_or(0)
    }

    /// Convenience method for CHR pattern table writes (`$0000-$1FFF`).
    ///
    /// Only affects mappers with CHR RAM. CHR ROM mappers should ignore writes.
    fn chr_write(&mut self, addr: u16, data: u8) {
        self.ppu_write(addr, data);
    }

    /// Maps a PPU nametable address (`$2000-$2FFF`) to its backing storage.
    ///
    /// The default implementation uses the mapper's [`Mirroring`] mode and
    /// standard CIRAM mapping to resolve the final CIRAM offset.
    fn map_nametable(&self, addr: u16) -> NametableTarget {
        let base = addr & 0x0FFF;
        let offset = match self.mirroring() {
            // Horizontal mirroring: $2000/$2400 share, $2800/$2C00 share.
            Mirroring::Horizontal => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 1 => within,
                    _ => 0x0400 | within,
                }
            }
            // Vertical mirroring: $2000/$2800 share, $2400/$2C00 share.
            Mirroring::Vertical => {
                let nt = (base >> 10) & 3;
                let within = base & 0x03FF;
                match nt {
                    0 | 2 => within,
                    _ => 0x0400 | within,
                }
            }
            // Four-screen: treat all four nametables as distinct regions.
            Mirroring::FourScreen => base & 0x0FFF,
            // Single-screen lower/upper select one of the two CIRAM pages.
            Mirroring::SingleScreenLower => base & 0x03FF,
            Mirroring::SingleScreenUpper => 0x0400 | (base & 0x03FF),
            // Mapper-controlled: delegate full nametable mapping to mapper VRAM/ROM.
            Mirroring::MapperControlled => base,
        };
        NametableTarget::Ciram(offset)
    }

    /// Called when [`map_nametable`] returns [`NametableTarget::MapperVram`]
    /// for nametable reads.
    fn mapper_nametable_read(&self, _offset: u16) -> u8 {
        0
    }

    /// Called when [`map_nametable`] returns [`NametableTarget::MapperVram`]
    /// for nametable writes.
    fn mapper_nametable_write(&mut self, _offset: u16, _value: u8) {}

    /// Returns `true` when the mapper asserts the CPU IRQ line.
    fn irq_pending(&self) -> bool {
        false
    }

    /// Optional introspection hook for PRG ROM contents.
    fn prg_rom(&self) -> Option<&[u8]> {
        None
    }

    /// Optional introspection hook for unified PRG RAM contents.
    ///
    /// New code should prefer the more granular `prg_save_ram` / `prg_work_ram`
    /// helpers below when it needs to distinguish battery-backed vs volatile
    /// work RAM. This method remains for backwards compatibility.
    fn prg_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Optional mutable access to PRG RAM contents.
    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// PRG save RAM (battery-backed), if present.
    fn prg_save_ram(&self) -> Option<&[u8]> {
        self.prg_ram()
    }

    /// Mutable view of PRG save RAM (battery-backed), if present.
    fn prg_save_ram_mut(&mut self) -> Option<&mut [u8]> {
        self.prg_ram_mut()
    }

    /// PRG work RAM (non battery-backed), if present.
    fn prg_work_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Mutable view of PRG work RAM (non battery-backed), if present.
    fn prg_work_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Mapper-private RAM (e.g., MMC5 ExRAM), if present.
    fn mapper_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Mutable view of mapper-private RAM, if present.
    fn mapper_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Optional introspection hook for CHR ROM contents.
    fn chr_rom(&self) -> Option<&[u8]> {
        None
    }

    /// Optional introspection hook for CHR RAM contents.
    fn chr_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Optional mutable access to CHR RAM contents.
    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Optional CHR battery-backed RAM region, if distinct from `chr_ram`.
    fn chr_battery_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Mutable view of CHR battery-backed RAM, if present.
    fn chr_battery_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Current nametable mirroring mode exposed by the mapper.
    fn mirroring(&self) -> Mirroring;

    /// Mapper identifier as used in the iNES header.
    fn mapper_id(&self) -> u16;

    /// Human readable mapper name.
    fn name(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Mapper {}", self.mapper_id()))
    }
}

dyn_clone::clone_trait_object!(Mapper);

/// Downcasts a mapper reference to a concrete implementation.
pub fn mapper_downcast_ref<T: Mapper + 'static>(mapper: &dyn Mapper) -> Option<&T> {
    (mapper as &dyn Any).downcast_ref::<T>()
}

/// Downcasts a mutable mapper reference to a concrete implementation.
pub fn mapper_downcast_mut<T: Mapper + 'static>(mapper: &mut dyn Mapper) -> Option<&mut T> {
    (mapper as &mut dyn Any).downcast_mut::<T>()
}

/// Allocate CPU‑visible PRG RAM according to the header hints.
///
/// For NES 2.0 headers this picks the larger of volatile and battery‑backed
/// PRG RAM sizes. Legacy iNES headers with `0` fall back to an empty slice.
pub fn allocate_prg_ram(header: &Header) -> Box<[u8]> {
    let mut size = header.prg_ram_size().max(header.prg_nvram_size());

    // Some iNES 1.0 ROMs specify 0 PRG RAM, but were designed for systems
    // that provided 8 KiB by default. Mesen2 provides 8 KiB for such ROMs
    // (except for specific board types like Action 52), so we do the same
    // here.
    if header.format() == RomFormat::INes && size == 0 {
        size = 8192; // 8 KiB
    }

    if size == 0 {
        Vec::new().into_boxed_slice()
    } else {
        vec![0; size].into_boxed_slice()
    }
}

/// Returns the region of PRG RAM where the optional trainer should be copied.
///
/// When the PRG RAM region is too small to host the trainer, `None` is
/// returned and the trainer contents are silently ignored.
pub fn trainer_destination(prg_ram: &mut [u8]) -> Option<&mut [u8]> {
    if prg_ram.len() < TRAINER_RAM_OFFSET + TRAINER_SIZE {
        return None;
    }
    Some(&mut prg_ram[TRAINER_RAM_OFFSET..TRAINER_RAM_OFFSET + TRAINER_SIZE])
}

/// Allocates CPU-visible PRG RAM and optionally copies the trainer into it.
///
/// This combines [`allocate_prg_ram`] and [`trainer_destination`] into a single
/// convenience helper used by most mappers during initialization.
pub fn allocate_prg_ram_with_trainer(header: &Header, trainer: TrainerBytes) -> Box<[u8]> {
    let mut prg_ram = allocate_prg_ram(header);
    if let (Some(trainer), Some(dst)) = (trainer, trainer_destination(&mut prg_ram)) {
        dst.copy_from_slice(trainer);
    }
    prg_ram
}
