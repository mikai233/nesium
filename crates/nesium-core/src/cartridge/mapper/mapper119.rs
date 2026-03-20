//! Mapper 119 – TQROM (MMC3 variant with mixed CHR ROM/RAM).
//!
//! This reuses MMC3 banking and IRQ behaviour but interprets CHR bank bit6 to
//! select CHR RAM instead of ROM. CHR RAM is treated as 8 KiB split into 1 KiB
//! pages; CHR ROM uses the low 6 bits as the 1 KiB page index.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio         |
//! |------|-------------------|----------------------------------------------------|-------------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM with enable/write-protect bits   | None              |
//! | CPU  | `$8000-$DFFF`     | MMC3-style 8 KiB PRG windows (bank select at `$8000`)| None           |
//! | CPU  | `$A000-$BFFF`     | Mirroring + PRG-RAM enable/write-protect          | None              |
//! | CPU  | `$C000-$FFFF`     | IRQ latch/reload and enable/ack registers         | MMC3 scanline IRQ |
//! | PPU  | `$0000-$1FFF`     | 1 KiB CHR banks, bit6 selects CHR RAM vs ROM      | None              |
//! | PPU  | `$2000-$3EFF`     | Mirroring from header or TQROM mirroring control  | None              |

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            MapperEvent, MapperHookMask, PpuVramAccessContext, allocate_prg_ram_with_trainer,
            core::mmc3::{
                MMC3_POWER_ON_BANK_REGS, Mmc3Core, Mmc3CoreResetConfig, Mmc3CpuRegister,
                Mmc3IrqRevision, Mmc3WriteConfig, Mmc3WriteResult, PRG_BANK_SIZE_8K,
                resolve_mmc3_chr_bank,
            },
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;
/// Fixed CHR-RAM size used by TQROM boards.
const CHR_RAM_SIZE: usize = 8 * 1024;
const TQROM_WRITE_CONFIG: Mmc3WriteConfig = Mmc3WriteConfig {
    bank_select_mask: 0xC7,
    clear_counter_on_reload: false,
};

#[derive(Debug, Clone)]
pub struct Mapper119 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr_rom: ChrRom,
    chr_ram: Box<[u8]>,

    /// Number of 8 KiB PRG-ROM banks.
    prg_bank_count: usize,

    base_mirroring: Mirroring,
    mirroring: Mirroring,
    mmc3: Mmc3Core,
    mmc3_reset: Mmc3CoreResetConfig,
}

impl Mapper119 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr_ram = vec![0u8; CHR_RAM_SIZE].into_boxed_slice();
        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);
        let mmc3_reset = Mmc3CoreResetConfig {
            bank_select: 0,
            bank_regs: MMC3_POWER_ON_BANK_REGS,
            prg_ram_enable: false,
            prg_ram_write_protect: false,
            irq_revision: Mmc3IrqRevision::RevB,
        };

        Self {
            prg_rom,
            prg_ram,
            chr_rom,
            chr_ram,
            prg_bank_count,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
            mmc3: Mmc3Core::new(mmc3_reset),
            mmc3_reset,
        }
    }

    #[inline]
    fn chr_invert(&self) -> bool {
        self.mmc3.chr_invert()
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        self.mmc3.read_prg_ram(&self.prg_ram, addr)
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        self.mmc3.write_prg_ram(&mut self.prg_ram, addr, data);
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let Some(bank) = self.mmc3.resolve_prg_rom_bank(self.prg_bank_count, addr) else {
            return 0;
        };

        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn chr_bank_base(&self, bank_reg: u8) -> (bool, usize) {
        // Bit6 selects CHR RAM; lower bits select page.
        let use_ram = bank_reg & 0x40 != 0;
        if use_ram {
            let page = (bank_reg & 0x07) as usize;
            (true, page * CHR_BANK_SIZE_1K)
        } else {
            let page = (bank_reg & 0x3F) as usize;
            (false, page * CHR_BANK_SIZE_1K)
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank_idx, offset) =
            resolve_mmc3_chr_bank(self.mmc3.bank_regs.as_slice(), self.chr_invert(), addr);
        let (use_ram, base) = self.chr_bank_base(bank_idx);
        if use_ram {
            let len = self.chr_ram.len().max(1);
            self.chr_ram[(base + offset) % len]
        } else {
            let len = self.chr_rom.len().max(1);
            self.chr_rom[(base + offset) % len]
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (bank_idx, offset) =
            resolve_mmc3_chr_bank(self.mmc3.bank_regs.as_slice(), self.chr_invert(), addr);
        let (use_ram, base) = self.chr_bank_base(bank_idx);
        if !use_ram || self.chr_ram.is_empty() {
            return;
        }
        let len = self.chr_ram.len();
        let idx = (base + offset) % len;
        self.chr_ram[idx] = data;
    }

    fn observe_ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        self.mmc3.observe_ppu_vram_access(addr, ctx);
    }
}

impl Mapper for Mapper119 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::PPU_BUS_ADDRESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::PpuBusAddress { addr, ctx } = event {
            self.observe_ppu_vram_access(addr, ctx);
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.mmc3.reset(self.mmc3_reset);
        self.mirroring = self.base_mirroring;
    }

    fn cpu_read(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        if (cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END).contains(&addr) {
            self.write_prg_ram(addr, data);
            return;
        }

        if let Some(reg) = Mmc3CpuRegister::from_addr(addr) {
            match self.mmc3.write_register(reg, data, TQROM_WRITE_CONFIG) {
                Mmc3WriteResult::Handled => {}
                Mmc3WriteResult::Mirroring(data) => {
                    self.mirroring = if data & 0x01 == 0 {
                        Mirroring::Vertical
                    } else {
                        Mirroring::Horizontal
                    };
                }
            }
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.mmc3.irq_pending
    }
    fn memory_ref(&self) -> MapperMemoryRef<'_> {
        MapperMemoryRef {
            prg_rom: Some(self.prg_rom.as_ref()),
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_ref()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_rom: (!self.chr_rom.is_empty()).then_some(self.chr_rom.as_ref()),
            chr_ram: (!self.chr_ram.is_empty()).then_some(self.chr_ram.as_ref()),
            chr_battery_ram: None,
        }
    }

    fn memory_mut(&mut self) -> MapperMemoryMut<'_> {
        MapperMemoryMut {
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_mut()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_ram: (!self.chr_ram.is_empty()).then_some(self.chr_ram.as_mut()),
            chr_battery_ram: None,
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        119
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("TQROM (MMC3 variant)")
    }
}
