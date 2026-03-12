//! Mapper 16 - Bandai FCG / LZ93D50.
//!
//! This implements the common Bandai mapper-16 paths used by legacy iNES
//! dumps and NES 2.0 submappers 4/5:
//! - 16 KiB PRG banking at `$8000-$BFFF` with a fixed last bank at `$C000`
//! - Eight 1 KiB CHR banks when CHR ROM is present
//! - Mirroring control via register `$x009`
//! - CPU-cycle IRQ counter via `$x00A-$x00C`
//! - Optional 24C02 serial EEPROM line at `$6000-$7FFF` / register `$x00D`
//!
//! Like Mesen, legacy iNES mapper 16 accepts register writes in both
//! `$6000-$7FFF` and `$8000-$FFFF` because the board variant is ambiguous.

use std::{borrow::Cow, fs::OpenOptions, io::Write};

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring, RomFormat},
        mapper::{ChrStorage, MapperEvent, MapperHookMask, select_chr_storage},
    },
    reset_kind::ResetKind,
};

const PRG_BANK_SIZE_16K: usize = 16 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

#[derive(Debug, Clone)]
struct Eeprom24c02 {
    mode: Eeprom24c02Mode,
    next_mode: Eeprom24c02Mode,
    chip_address: u8,
    address: u8,
    data: u8,
    counter: u8,
    output: u8,
    prev_scl: u8,
    prev_sda: u8,
    storage: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Eeprom24c02Mode {
    Idle,
    ChipAddress,
    Address,
    Read,
    Write,
    SendAck,
    WaitAck,
}

impl Eeprom24c02 {
    fn new() -> Self {
        Self {
            mode: Eeprom24c02Mode::Idle,
            next_mode: Eeprom24c02Mode::Idle,
            chip_address: 0,
            address: 0,
            data: 0,
            counter: 0,
            output: 0,
            prev_scl: 0,
            prev_sda: 0,
            storage: vec![0; 256].into_boxed_slice(),
        }
    }

    fn read_bit(&self) -> u8 {
        self.output & 0x01
    }

    fn storage(&self) -> &[u8] {
        self.storage.as_ref()
    }

    fn storage_mut(&mut self) -> &mut [u8] {
        self.storage.as_mut()
    }

    fn debug_state(&self) -> String {
        format!(
            "mode={:?}|next={:?}|chip={:02X}|addr={:02X}|data={:02X}|counter={}|output={}|prev_scl={}|prev_sda={}",
            self.mode,
            self.next_mode,
            self.chip_address,
            self.address,
            self.data,
            self.counter,
            self.output,
            self.prev_scl,
            self.prev_sda
        )
    }

    fn write_lines(&mut self, scl: u8, sda: u8) {
        if self.prev_scl != 0 && scl != 0 && sda < self.prev_sda {
            self.mode = Eeprom24c02Mode::ChipAddress;
            self.counter = 0;
            self.output = 1;
        } else if self.prev_scl != 0 && scl != 0 && sda > self.prev_sda {
            self.mode = Eeprom24c02Mode::Idle;
            self.output = 1;
        } else if scl > self.prev_scl {
            match self.mode {
                Eeprom24c02Mode::ChipAddress => {
                    Self::write_serial_bit(&mut self.chip_address, &mut self.counter, sda);
                }
                Eeprom24c02Mode::Address => {
                    Self::write_serial_bit(&mut self.address, &mut self.counter, sda);
                }
                Eeprom24c02Mode::Read => self.read_data_bit(),
                Eeprom24c02Mode::Write => {
                    Self::write_serial_bit(&mut self.data, &mut self.counter, sda);
                }
                Eeprom24c02Mode::SendAck => {
                    self.output = 0;
                }
                Eeprom24c02Mode::WaitAck => {
                    if sda == 0 {
                        self.next_mode = Eeprom24c02Mode::Read;
                        self.data = self.storage[self.address as usize];
                    }
                }
                Eeprom24c02Mode::Idle => {}
            }
        } else if scl < self.prev_scl {
            match self.mode {
                Eeprom24c02Mode::ChipAddress if self.counter == 8 => {
                    if (self.chip_address & 0xA0) == 0xA0 {
                        self.mode = Eeprom24c02Mode::SendAck;
                        self.counter = 0;
                        self.output = 1;
                        if self.chip_address & 0x01 != 0 {
                            self.next_mode = Eeprom24c02Mode::Read;
                            self.data = self.storage[self.address as usize];
                        } else {
                            self.next_mode = Eeprom24c02Mode::Address;
                        }
                    } else {
                        self.mode = Eeprom24c02Mode::Idle;
                        self.counter = 0;
                        self.output = 1;
                    }
                }
                Eeprom24c02Mode::Address if self.counter == 8 => {
                    self.counter = 0;
                    self.mode = Eeprom24c02Mode::SendAck;
                    self.next_mode = Eeprom24c02Mode::Write;
                    self.output = 1;
                }
                Eeprom24c02Mode::Read if self.counter == 8 => {
                    self.mode = Eeprom24c02Mode::WaitAck;
                    self.address = self.address.wrapping_add(1);
                }
                Eeprom24c02Mode::Write if self.counter == 8 => {
                    self.counter = 0;
                    self.mode = Eeprom24c02Mode::SendAck;
                    self.next_mode = Eeprom24c02Mode::Write;
                    self.storage[self.address as usize] = self.data;
                    self.address = self.address.wrapping_add(1);
                }
                Eeprom24c02Mode::SendAck | Eeprom24c02Mode::WaitAck => {
                    self.mode = self.next_mode;
                    self.counter = 0;
                    self.output = 1;
                }
                _ => {}
            }
        }

        self.prev_scl = scl;
        self.prev_sda = sda;
    }

    fn write_serial_bit(dest: &mut u8, counter: &mut u8, value: u8) {
        if *counter < 8 {
            let shift = 7 - *counter;
            let mask = !(1 << shift);
            *dest = (*dest & mask) | ((value & 0x01) << shift);
            *counter += 1;
        }
    }

    fn read_data_bit(&mut self) {
        if self.counter < 8 {
            let shift = 7 - self.counter;
            self.output = u8::from((self.data & (1 << shift)) != 0);
            self.counter += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct Eeprom24c01 {
    mode: Eeprom24c01Mode,
    next_mode: Eeprom24c01Mode,
    address: u8,
    data: u8,
    counter: u8,
    output: u8,
    prev_scl: u8,
    prev_sda: u8,
    storage: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Eeprom24c01Mode {
    Idle,
    Address,
    Read,
    Write,
    SendAck,
    WaitAck,
}

impl Eeprom24c01 {
    fn new() -> Self {
        Self {
            mode: Eeprom24c01Mode::Idle,
            next_mode: Eeprom24c01Mode::Idle,
            address: 0,
            data: 0,
            counter: 0,
            output: 0,
            prev_scl: 0,
            prev_sda: 0,
            storage: vec![0; 128].into_boxed_slice(),
        }
    }

    fn read_bit(&self) -> u8 {
        self.output & 0x01
    }

    fn debug_state(&self) -> String {
        format!(
            "mode={:?}|next={:?}|addr={:02X}|data={:02X}|counter={}|output={}|prev_scl={}|prev_sda={}",
            self.mode,
            self.next_mode,
            self.address,
            self.data,
            self.counter,
            self.output,
            self.prev_scl,
            self.prev_sda
        )
    }

    fn write_scl(&mut self, scl: u8) {
        self.write_lines(scl, self.prev_sda);
    }

    fn write_sda(&mut self, sda: u8) {
        self.write_lines(self.prev_scl, sda);
    }

    fn write_lines(&mut self, scl: u8, sda: u8) {
        if self.prev_scl != 0 && scl != 0 && sda < self.prev_sda {
            self.mode = Eeprom24c01Mode::Address;
            self.address = 0;
            self.counter = 0;
            self.output = 1;
        } else if self.prev_scl != 0 && scl != 0 && sda > self.prev_sda {
            self.mode = Eeprom24c01Mode::Idle;
            self.output = 1;
        } else if scl > self.prev_scl {
            match self.mode {
                Eeprom24c01Mode::Address => {
                    if self.counter < 7 {
                        Self::write_serial_bit_lsb(&mut self.address, &mut self.counter, sda);
                    } else if self.counter == 7 {
                        self.counter = 8;
                        if sda != 0 {
                            self.next_mode = Eeprom24c01Mode::Read;
                            self.data = self.storage[(self.address & 0x7F) as usize];
                        } else {
                            self.next_mode = Eeprom24c01Mode::Write;
                        }
                    }
                }
                Eeprom24c01Mode::SendAck => {
                    self.output = 0;
                }
                Eeprom24c01Mode::Read => self.read_data_bit(),
                Eeprom24c01Mode::Write => {
                    Self::write_serial_bit_lsb(&mut self.data, &mut self.counter, sda);
                }
                Eeprom24c01Mode::WaitAck => {
                    if sda != 0 {
                        self.next_mode = Eeprom24c01Mode::Idle;
                    }
                }
                Eeprom24c01Mode::Idle => {}
            }
        } else if scl < self.prev_scl {
            match self.mode {
                Eeprom24c01Mode::Address if self.counter == 8 => {
                    self.mode = Eeprom24c01Mode::SendAck;
                    self.output = 1;
                }
                Eeprom24c01Mode::SendAck => {
                    self.mode = self.next_mode;
                    self.counter = 0;
                    self.output = 1;
                }
                Eeprom24c01Mode::Read if self.counter == 8 => {
                    self.mode = Eeprom24c01Mode::WaitAck;
                    self.address = self.address.wrapping_add(1) & 0x7F;
                }
                Eeprom24c01Mode::Write if self.counter == 8 => {
                    self.mode = Eeprom24c01Mode::SendAck;
                    self.next_mode = Eeprom24c01Mode::Idle;
                    self.storage[(self.address & 0x7F) as usize] = self.data;
                    self.address = self.address.wrapping_add(1) & 0x7F;
                }
                _ => {}
            }
        }

        self.prev_scl = scl;
        self.prev_sda = sda;
    }

    fn write_serial_bit_lsb(dest: &mut u8, counter: &mut u8, value: u8) {
        if *counter < 8 {
            let mask = !(1 << *counter);
            *dest = (*dest & mask) | ((value & 0x01) << *counter);
            *counter += 1;
        }
    }

    fn read_data_bit(&mut self) {
        if self.counter < 8 {
            self.output = u8::from((self.data & (1 << self.counter)) != 0);
            self.counter += 1;
        }
    }
}

#[derive(Debug, Clone)]
struct DatachBarcodeReader {
    data: Vec<u8>,
    insert_master_clock: u64,
}

impl DatachBarcodeReader {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            insert_master_clock: 0,
        }
    }

    fn output(&self, master_clock: u64) -> u8 {
        let elapsed = master_clock.saturating_sub(self.insert_master_clock);
        let bit = (elapsed / 1000) as usize;
        self.data.get(bit).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mapper16Variant {
    Standard,
    Datach,
}

#[derive(Debug, Clone)]
pub struct Mapper16 {
    prg_rom: PrgRom,
    chr: ChrStorage,
    base_mirroring: Mirroring,
    mirroring: Mirroring,
    submapper: u8,
    variant: Mapper16Variant,
    prg_bank_count_16k: usize,
    chr_banks: [u8; 8],
    prg_page: u8,
    prg_bank_select: u8,
    lower_prg_mapped: bool,
    irq_enabled: bool,
    irq_counter: u16,
    irq_reload: u16,
    irq_pending: bool,
    master_clock: u64,
    eeprom: Option<Eeprom24c02>,
    extra_eeprom: Option<Eeprom24c01>,
    barcode_reader: Option<DatachBarcodeReader>,
    trace_path: Option<String>,
    trace_cycle_start: u64,
    trace_cycle_end: u64,
    trace_seq: u64,
}

impl Mapper16 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, _trainer: TrainerBytes) -> Self {
        let chr = select_chr_storage(&header, chr_rom);
        let submapper = header.submapper();
        let variant = Self::detect_variant(&header);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);
        let eeprom = Self::create_eeprom(&header, variant);
        let extra_eeprom = Self::create_extra_eeprom(&header, variant);
        let barcode_reader = (variant == Mapper16Variant::Datach).then(DatachBarcodeReader::new);
        let trace_path = std::env::var("NESIUM_MAPPER16_TRACE_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let trace_cycle_start = std::env::var("NESIUM_MAPPER16_TRACE_CYCLE_START")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let trace_cycle_end = std::env::var("NESIUM_MAPPER16_TRACE_CYCLE_END")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(u64::MAX);

        Self {
            prg_rom,
            chr,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
            submapper,
            variant,
            prg_bank_count_16k,
            chr_banks: [0; 8],
            prg_page: 0,
            prg_bank_select: 0,
            lower_prg_mapped: false,
            irq_enabled: false,
            irq_counter: 0,
            irq_reload: 0,
            irq_pending: false,
            master_clock: 0,
            eeprom,
            extra_eeprom,
            barcode_reader,
            trace_path,
            trace_cycle_start,
            trace_cycle_end,
            trace_seq: 0,
        }
    }

    fn detect_variant(header: &Header) -> Mapper16Variant {
        if header.mapper() == 157 {
            Mapper16Variant::Datach
        } else {
            Mapper16Variant::Standard
        }
    }

    fn create_eeprom(header: &Header, variant: Mapper16Variant) -> Option<Eeprom24c02> {
        if header.mapper() != 16 && header.mapper() != 157 {
            return None;
        }

        if variant == Mapper16Variant::Datach {
            return Some(Eeprom24c02::new());
        }

        match (header.format(), header.submapper()) {
            (RomFormat::INes, 0) => Some(Eeprom24c02::new()),
            (RomFormat::Nes20, 0) => Some(Eeprom24c02::new()),
            (RomFormat::Nes20, 5) if header.prg_nvram_size().max(header.prg_ram_size()) == 256 => {
                Some(Eeprom24c02::new())
            }
            _ => None,
        }
    }

    fn create_extra_eeprom(_header: &Header, variant: Mapper16Variant) -> Option<Eeprom24c01> {
        if variant == Mapper16Variant::Datach {
            Some(Eeprom24c01::new())
        } else {
            None
        }
    }

    #[inline]
    fn uses_lower_register_range(&self) -> bool {
        self.variant == Mapper16Variant::Standard && self.submapper != 5
    }

    #[inline]
    fn uses_upper_register_range(&self) -> bool {
        self.variant == Mapper16Variant::Datach || self.submapper != 4
    }

    #[inline]
    fn lower_read_port_enabled(&self) -> bool {
        true
    }

    fn decode_register_addr(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF if self.uses_lower_register_range() => Some((addr & 0x0F) as u8),
            0x8000..=0xFFFF if self.uses_upper_register_range() => Some((addr & 0x0F) as u8),
            _ => None,
        }
    }

    #[inline]
    fn has_banked_chr_rom(&self) -> bool {
        self.chr.as_rom().is_some()
    }

    fn recompute_prg_bank_select(&mut self) {
        if self.prg_bank_count_16k >= 0x20 {
            self.prg_bank_select = self
                .chr_banks
                .iter()
                .fold(0u8, |acc, &value| acc | ((value & 0x01) << 4));
        } else {
            self.prg_bank_select = 0;
        }
    }

    #[inline]
    fn prg_bank_index(&self, bank: u8) -> usize {
        if self.prg_bank_count_16k == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count_16k
        }
    }

    fn read_prg(&self, bank: u8, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let base = self.prg_bank_index(bank) * PRG_BANK_SIZE_16K;
        let offset = (addr as usize) & (PRG_BANK_SIZE_16K - 1);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn read_chr(&self, addr: u16) -> u8 {
        if !self.has_banked_chr_rom() {
            return self.chr.read(addr);
        }

        let a = addr & 0x1FFF;
        let slot = ((a >> 10) & 0x07) as usize;
        let base = self.chr_banks[slot] as usize * CHR_BANK_SIZE_1K;
        let offset = (a & 0x03FF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if !self.has_banked_chr_rom() {
            self.chr.write(addr, data);
            return;
        }

        let a = addr & 0x1FFF;
        let slot = ((a >> 10) & 0x07) as usize;
        let base = self.chr_banks[slot] as usize * CHR_BANK_SIZE_1K;
        let offset = (a & 0x03FF) as usize;
        self.chr.write_indexed(base, offset, data);
    }

    fn write_register(&mut self, reg: u8, value: u8) {
        match reg {
            0x00..=0x07 => {
                self.chr_banks[reg as usize] = value;
                self.recompute_prg_bank_select();
                if self.prg_bank_count_16k >= 0x20 {
                    self.lower_prg_mapped = true;
                }
                if let Some(extra_eeprom) = &mut self.extra_eeprom
                    && self.variant == Mapper16Variant::Datach
                    && reg <= 0x03
                {
                    extra_eeprom.write_scl((value >> 3) & 0x01);
                }
            }
            0x08 => {
                self.prg_page = value & 0x0F;
                self.lower_prg_mapped = true;
            }
            0x09 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLower,
                    _ => Mirroring::SingleScreenUpper,
                };
            }
            0x0A => {
                self.irq_enabled = value & 0x01 != 0;
                if self.submapper != 4 {
                    self.irq_counter = self.irq_reload;
                }
                self.irq_pending = false;
            }
            0x0B => {
                if self.submapper == 4 {
                    self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
                } else {
                    self.irq_reload = (self.irq_reload & 0xFF00) | value as u16;
                }
            }
            0x0C => {
                if self.submapper == 4 {
                    self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
                } else {
                    self.irq_reload = (self.irq_reload & 0x00FF) | ((value as u16) << 8);
                }
            }
            0x0D => {
                if let Some(eeprom) = &mut self.eeprom {
                    eeprom.write_lines((value >> 5) & 0x01, (value >> 6) & 0x01);
                }
                if let Some(extra_eeprom) = &mut self.extra_eeprom
                    && self.variant == Mapper16Variant::Datach
                {
                    extra_eeprom.write_sda((value >> 6) & 0x01);
                }
            }
            _ => {}
        }
    }

    fn trace_cpu_bus_access(
        &mut self,
        kind: &'static str,
        addr: u16,
        value: u8,
        cpu_cycle: u64,
        master_clock: u64,
    ) {
        let Some(path) = self.trace_path.as_deref() else {
            return;
        };
        if cpu_cycle < self.trace_cycle_start || cpu_cycle > self.trace_cycle_end {
            return;
        }
        let tracked_addr = addr == 0x6000 || (0x8000..=0x800D).contains(&addr);
        if !tracked_addr {
            return;
        }
        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
            return;
        };
        let eeprom_state = self
            .eeprom
            .as_ref()
            .map(Eeprom24c02::debug_state)
            .unwrap_or_else(|| "none".to_string());
        let extra_eeprom_state = self
            .extra_eeprom
            .as_ref()
            .map(Eeprom24c01::debug_state)
            .unwrap_or_else(|| "none".to_string());
        let _ = writeln!(
            file,
            "M16TRACE|seq={}|kind={}|cpu_cycle={}|master_clock={}|addr={:04X}|value={:02X}|prg_page={:02X}|prg_bank_select={:02X}|selected_bank={:02X}|eeprom={}|extra_eeprom={}",
            self.trace_seq,
            kind,
            cpu_cycle,
            master_clock,
            addr,
            value,
            self.prg_page,
            self.prg_bank_select,
            self.prg_page | self.prg_bank_select,
            eeprom_state,
            extra_eeprom_state
        );
        self.trace_seq = self.trace_seq.saturating_add(1);
    }

    #[doc(hidden)]
    pub fn debug_irq_state(&self) -> (bool, bool, u16, u16, u8) {
        (
            self.irq_enabled,
            self.irq_pending,
            self.irq_counter,
            self.irq_reload,
            self.prg_page,
        )
    }
}

impl Mapper for Mapper16 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK | MapperHookMask::CPU_BUS_ACCESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuBusAccess {
            kind,
            addr,
            value,
            cpu_cycle,
            master_clock,
        } = event
        {
            let label = match kind {
                crate::cartridge::mapper::CpuBusAccessKind::Read => "read",
                crate::cartridge::mapper::CpuBusAccessKind::Write => "write",
                _ => return,
            };
            self.trace_cpu_bus_access(label, addr, value, cpu_cycle, master_clock);
        } else if let MapperEvent::CpuClock { master_clock, .. } = event
            && self.irq_enabled
        {
            self.master_clock = master_clock;
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
            self.irq_counter = self.irq_counter.wrapping_sub(1);
        } else if let MapperEvent::CpuClock { master_clock, .. } = event {
            self.master_clock = master_clock;
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.chr_banks = [0; 8];
        self.prg_page = 0;
        self.prg_bank_select = 0;
        self.lower_prg_mapped = false;
        self.irq_enabled = false;
        self.irq_counter = 0;
        self.irq_reload = 0;
        self.irq_pending = false;
        self.master_clock = 0;
        self.mirroring = self.base_mirroring;
    }

    fn cpu_read(&self, addr: u16, open_bus: u8) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF if self.lower_read_port_enabled() => {
                let mut output = open_bus & 0xE7;
                if let Some(reader) = &self.barcode_reader {
                    output |= reader.output(self.master_clock);
                }
                if let (Some(eeprom), Some(extra)) = (&self.eeprom, &self.extra_eeprom) {
                    output |= u8::from(eeprom.read_bit() != 0 && extra.read_bit() != 0) << 4;
                } else if let Some(eeprom) = &self.eeprom {
                    output |= eeprom.read_bit() << 4;
                }
                Some(output)
            }
            0x8000..=0xBFFF => self
                .lower_prg_mapped
                .then(|| self.read_prg(self.prg_page | self.prg_bank_select, addr)),
            0xC000..=0xFFFF => Some(self.read_prg(0x0F | self.prg_bank_select, addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        if let Some(reg) = self.decode_register_addr(addr) {
            self.write_register(reg, data);
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn memory_ref(&self) -> MapperMemoryRef<'_> {
        MapperMemoryRef {
            prg_rom: Some(self.prg_rom.as_ref()),
            prg_ram: None,
            prg_work_ram: None,
            mapper_ram: self.eeprom.as_ref().map(Eeprom24c02::storage),
            chr_rom: self.chr.as_rom(),
            chr_ram: self.chr.as_ram(),
            chr_battery_ram: None,
        }
    }

    fn memory_mut(&mut self) -> MapperMemoryMut<'_> {
        MapperMemoryMut {
            prg_ram: None,
            prg_work_ram: None,
            mapper_ram: self.eeprom.as_mut().map(Eeprom24c02::storage_mut),
            chr_ram: self.chr.as_ram_mut(),
            chr_battery_ram: None,
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        match self.variant {
            Mapper16Variant::Standard => 16,
            Mapper16Variant::Datach => 157,
        }
    }

    fn name(&self) -> Cow<'static, str> {
        if self.variant == Mapper16Variant::Datach {
            Cow::Borrowed("Bandai Datach (legacy iNES mapper 16)")
        } else {
            Cow::Borrowed("Bandai FCG / LZ93D50")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cartridge::header::Header;

    fn ines_header(prg_16k_units: u8, chr_8k_units: u8) -> Header {
        let mut rom = [0u8; 16];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = prg_16k_units;
        rom[5] = chr_8k_units;
        rom[7] = 0x10;
        Header::parse(&rom).expect("valid iNES header")
    }

    fn nes20_header(prg_16k_units: u8, chr_8k_units: u8, submapper: u8) -> Header {
        let mut rom = [0u8; 16];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = prg_16k_units;
        rom[5] = chr_8k_units;
        rom[7] = 0x18;
        rom[8] = submapper << 4;
        Header::parse(&rom).expect("valid NES 2.0 header")
    }

    fn test_mapper(header: Header, prg_banks_16k: usize, chr_banks_8k: usize) -> Mapper16 {
        let mut prg = vec![0u8; prg_banks_16k * PRG_BANK_SIZE_16K];
        for (bank, chunk) in prg.chunks_exact_mut(PRG_BANK_SIZE_16K).enumerate() {
            chunk.fill(bank as u8);
        }
        let mut chr = vec![0u8; chr_banks_8k * 8 * 1024];
        for (bank, chunk) in chr.chunks_exact_mut(8 * 1024).enumerate() {
            chunk.fill((bank as u8) << 4);
        }
        Mapper16::new(header, prg.into(), chr.into(), None)
    }

    #[test]
    fn switches_16k_prg_bank_and_keeps_last_bank_fixed() {
        let header = ines_header(16, 1);
        let mut mapper = test_mapper(header, 16, 1);
        mapper.reset(ResetKind::PowerOn);

        assert_eq!(mapper.cpu_read(0xC000, 0), Some(15));
        assert_eq!(mapper.cpu_read(0x8000, 0), None);
        mapper.cpu_write(0x6008, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x8000, 0), Some(3));
    }

    #[test]
    fn lower_prg_window_starts_unmapped_until_bank_select() {
        let header = ines_header(16, 1);
        let mut mapper = test_mapper(header, 16, 1);
        mapper.reset(ResetKind::PowerOn);

        assert_eq!(mapper.cpu_read(0x8000, 0), None);
        assert_eq!(mapper.cpu_read(0xBFFF, 0), None);
        assert_eq!(mapper.cpu_read(0xC000, 0), Some(15));

        mapper.cpu_write(0x6008, 0x00, 0);
        assert_eq!(mapper.cpu_read(0x8000, 0), Some(0));
    }

    #[test]
    fn chr_registers_select_individual_1k_rom_pages() {
        let header = ines_header(2, 2);
        let mut mapper = test_mapper(header, 2, 2);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x6001, 0x08, 0);
        assert_eq!(mapper.ppu_read(0x0400), Some(0x10));
    }

    #[test]
    fn lower_range_only_submapper_4() {
        let header = nes20_header(16, 1, 4);
        let mut mapper = test_mapper(header, 16, 1);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x8008, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x8000, 0), None);

        mapper.cpu_write(0x6008, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x8000, 0), Some(3));
    }

    #[test]
    fn irq_reload_on_enable_for_default_path() {
        let header = ines_header(16, 1);
        let mut mapper = test_mapper(header, 16, 1);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x600B, 0x00, 0);
        mapper.cpu_write(0x600C, 0x00, 0);
        mapper.cpu_write(0x600A, 0x01, 0);
        mapper.on_mapper_event(MapperEvent::CpuClock {
            cpu_cycle: 0,
            master_clock: 0,
        });

        assert!(mapper.irq_pending());
    }
}
