use std::fmt::{Debug, Display};

use crate::bus::{CpuBus, STACK_ADDR};
use crate::context::Context;
use crate::cpu::addressing::Addressing;
use crate::cpu::cycle::{CYCLE_TABLE, Cycle};
use crate::cpu::instruction::Instruction;
use crate::cpu::irq::{IrqKind, IrqSource};
use crate::cpu::lookup::LOOKUP_TABLE;
use crate::cpu::mnemonic::Mnemonic;
use crate::cpu::status::Status;
use crate::memory::cpu::{IRQ_VECTOR_HI, IRQ_VECTOR_LO, NMI_VECTOR_HI, NMI_VECTOR_LO};
use crate::memory::cpu::{RESET_VECTOR_HI, RESET_VECTOR_LO};
use crate::memory::ppu::{self as ppu_mem, Register as PpuRegister};
use crate::reset_kind::ResetKind;

// Debug builds keep the standard checks; release uses unchecked hints to avoid
// the panic path for paths we prove unreachable in the execution tables.
macro_rules! unreachable_step {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            unreachable!($($arg)*)
        }
        #[cfg(not(debug_assertions))]
        unsafe {
            std::hint::unreachable_unchecked()
        }
    }};
}

pub mod addressing;
mod cycle;
mod instruction;
mod irq;
mod lookup;
mod micro_op;
mod mnemonic;
mod status;

/// Lightweight CPU register snapshot used for tracing/debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CpuSnapshot {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
}

/// Minimal opcode metadata for debugging / disassembly helpers.
#[derive(Debug, Clone)]
pub struct OpcodeMeta {
    pub mnemonic: String,
    pub addressing: Addressing,
}

/// Returns the mnemonic and addressing mode for a raw opcode.
pub fn opcode_meta(opcode: u8) -> OpcodeMeta {
    let inst = &LOOKUP_TABLE[opcode as usize];
    OpcodeMeta {
        mnemonic: format!("{:?}", inst.mnemonic),
        addressing: inst.addressing,
    }
}

const OAM_DMA_TRANSFER_BYTES: u16 = ppu_mem::OAM_RAM_SIZE as u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OamDma {
    page: u8,
    offset: u16,
    dummy_cycles: u8,
    read_phase: bool,
    data_latch: u8,
}

impl OamDma {
    fn new(page: u8, start_on_odd_cycle: bool) -> Self {
        // DMA always incurs one dummy cycle; starting on an odd CPU cycle
        // adds a second to align the following read/write alternation.
        let dummy_cycles = 1 + u8::from(start_on_odd_cycle);
        Self {
            page,
            offset: 0,
            dummy_cycles,
            read_phase: true,
            data_latch: 0,
        }
    }

    /// Marks that an external DMA (e.g., DMC) stole this CPU bus cycle while
    /// OAM DMA was in progress. The transfer pauses for this cycle, preserving
    /// read/write phase and remaining bytes.
    fn stall_cycle(&mut self) {
        // No state changes; the DMA simply does not advance this cycle.
    }

    /// Runs one DMA micro-step (one CPU cycle). Returns `true` when the
    /// transfer has finished copying all 256 bytes into OAM.
    fn step(&mut self, cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context) -> bool {
        if self.dummy_cycles > 0 {
            self.dummy_cycles -= 1;
            return false;
        }

        if self.read_phase {
            let addr = ((self.page as u16) << 8) | self.offset;
            self.data_latch = bus.mem_read(addr, cpu, ctx);
            self.read_phase = false;
            return false;
        }

        bus.mem_write(PpuRegister::OamData.addr(), self.data_latch, cpu, ctx);
        self.offset += 1;
        if self.offset >= OAM_DMA_TRANSFER_BYTES {
            return true;
        }
        self.read_phase = true;
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cpu {
    // Registers
    pub(crate) a: u8,     //Accumulator
    pub(crate) x: u8,     //X Index Register
    pub(crate) y: u8,     //Y Index Register
    pub(crate) s: u8,     //Stack Pointer
    pub(crate) p: Status, //Processor Status
    pub(crate) pc: u16,   //Program Counter

    pub(crate) opcode_in_flight: Option<u8>,
    pub(crate) irq_latch: IrqSource,
    pub(crate) prev_irq_active: bool,
    pub(crate) irq_active: bool,
    pub(crate) irq_enable_mask: IrqSource,
    /// Previous sampled NMI line level (for edge detection).
    pub(crate) prev_nmi_level: bool,
    /// NMI pending flag set on rising edge, consumed when NMI is taken.
    pub(crate) nmi_latch: bool,
    /// Previous cycle's pending flag, used to delay NMI by one instruction boundary.
    pub(crate) prev_nmi_latch: bool,
    pub(crate) index: u8,
    pub(crate) base: u8,
    pub(crate) effective_addr: u16,
    pub(crate) oam_dma: Option<OamDma>,
}

impl Cpu {
    /// Create a new CPU instance with default values.
    /// Does not automatically fetch the reset vector — call `reset()` for that.
    pub(crate) fn new() -> Self {
        Self {
            a: 0x00,                             // Accumulator
            x: 0x00,                             // X register
            y: 0x00,                             // Y register
            s: 0xFD,                             // Stack pointer after reset
            p: Status::from_bits_truncate(0x34), // IRQ disabled, bit 5 always set
            pc: 0x0000,                          // Will be set by reset vector
            opcode_in_flight: None,
            irq_latch: IrqSource::empty(),
            prev_irq_active: false,
            irq_active: false,
            irq_enable_mask: IrqSource::empty(),
            prev_nmi_level: false,
            nmi_latch: false,
            prev_nmi_latch: false,
            index: 0,
            base: 0,
            effective_addr: 0,
            oam_dma: None,
        }
    }

    /// Perform a CPU reset. The `kind` distinguishes between a true power-on
    /// reset and a soft reset triggered while the CPU is already running.
    ///
    /// Power-on reset:
    /// - Clears A/X/Y.
    /// - Sets S = $FD and P = INTERRUPT (I=1, bit 5 set, etc.).
    ///
    /// Soft reset:
    /// - Preserves A/X/Y and most status flags.
    /// - Sets the I flag (disables IRQs).
    /// - Decrements S by 3 (wrapping), matching 6502/Mesen behaviour.
    pub(crate) fn reset(&mut self, bus: &mut CpuBus<'_>, kind: ResetKind, ctx: &mut Context) {
        // Read the reset vector from memory ($FFFC-$FFFD) without advancing timing.
        let lo = bus.read(RESET_VECTOR_LO, self, ctx);
        let hi = bus.read(RESET_VECTOR_HI, self, ctx);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        match kind {
            ResetKind::PowerOn => {
                // Full power-on reset: initialize registers to known values.
                self.a = 0;
                self.x = 0;
                self.y = 0;
                self.s = 0xFD;
                self.p = Status::INTERRUPT;
                self.irq_enable_mask = IrqSource::all();
                self.irq_active = false;
            }
            ResetKind::Soft => {
                // Soft reset: keep A/X/Y and most of P.
                // Hardware behaviour is: set I and decrement S by 3.
                self.p.insert(Status::INTERRUPT);
                self.s = self.s.wrapping_sub(3);
            }
        }

        // Reset internal CPU state used by the execution engine and interrupt logic.
        self.opcode_in_flight = None;
        self.irq_latch = IrqSource::empty();
        self.irq_active = false;
        self.prev_nmi_level = false;
        self.nmi_latch = false;
        self.prev_nmi_latch = false;
        self.index = 0;
        self.base = 0;
        self.effective_addr = 0;
        self.oam_dma = None;

        // The CPU takes 8 cycles before it starts executing the ROM's code
        // after a reset/power-up (Mesen does this via 8 Start/End cycles).
        for _ in 0..8 {
            bus.internal_cycle(self, ctx);
        }
    }

    pub(crate) fn step(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context) {
        if self.handle_oam_dma(bus, ctx) {
            return;
        }

        match self.opcode_in_flight {
            Some(opcode) => {
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.exec(bus, ctx, instr);
                self.post_exec(instr);
                if self.index >= instr.len() {
                    self.clear();
                }
            }
            None => {
                if self.prev_irq_active || self.prev_nmi_latch {
                    self.perform_interrupt(bus, ctx);
                } else {
                    let opcode = self.fetch_opcode(bus, ctx);
                    self.opcode_in_flight = Some(opcode);
                    let instr = &LOOKUP_TABLE[opcode as usize];
                    self.pre_exec(instr);
                }
            }
        }
    }

    pub(crate) fn begin_cycle(
        &mut self,
        read_phase: bool,
        bus: &mut CpuBus<'_>,
        ctx: &mut Context,
    ) {
        let start_delta = if read_phase {
            bus.clock_start_count.saturating_sub(1)
        } else {
            bus.clock_start_count.saturating_add(1)
        };
        *bus.cycles = bus.cycles.wrapping_add(1);
        bus.bump_master_clock(start_delta, self, ctx);

        if let Some(cart) = bus.cartridge.as_mut() {
            cart.cpu_clock(*bus.cycles);
        }
        bus.open_bus.step();

        // Run one APU CPU-cycle tick; stash any pending DMC DMA stall.
        let (stall_cycles, dma_addr) = match &mut bus.mixer {
            Some(mixer) => bus.apu.step_with_mixer(mixer),
            None => bus.apu.step(),
        };
        bus.pending_dmc_stall = if stall_cycles > 0 {
            Some((stall_cycles, dma_addr))
        } else {
            None
        };
    }

    pub(crate) fn end_cycle(&mut self, read_phase: bool, bus: &mut CpuBus<'_>, ctx: &mut Context) {
        let end_delta = if read_phase {
            bus.clock_end_count.saturating_add(1)
        } else {
            bus.clock_end_count.saturating_sub(1)
        };
        bus.bump_master_clock(end_delta, self, ctx);
        self.prev_nmi_latch = self.nmi_latch;
        let nmi_level = bus.nmi_level();
        if nmi_level && !self.prev_nmi_level {
            self.nmi_latch = true;
        }
        self.prev_nmi_level = nmi_level;

        self.prev_irq_active = self.irq_active;
        // self.pending_irq =
        // self.irq_latch.intersects(self.irq_enable_mask) && !self.p.contains(Status::INTERRUPT);
        self.irq_active = bus.irq_level() && !self.p.contains(Status::INTERRUPT);
    }

    #[inline]
    pub(crate) fn fetch_opcode(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context) -> u8 {
        let opcode = bus.mem_read(self.pc, self, ctx);
        self.incr_pc();
        opcode
    }

    #[inline]
    pub(crate) fn pre_exec(&mut self, instr: &Instruction) {
        match instr.addressing {
            // Immediate Addressing:
            // This mode only has one execution cycle after the opcode fetch.
            // Set 'effective_addr' now to the current PC (which points to the data byte)
            // so the main execution logic can fetch the data from a unified source,
            // regardless of the addressing mode. Then, advance the PC past the data byte.
            Addressing::Immediate => {
                self.effective_addr = self.pc;
            }

            // Accumulator Addressing:
            // This mode operates directly on the Accumulator (A) register, not memory.
            // It bypasses all memory read/write micro-ops that other modes might use
            // (often involving 'dummy reads').
            // To unify the core 'exec' logic, jump the cycle index directly to the final
            // instruction execution phase (usually the last cycle).
            Addressing::Accumulator => {
                self.index = instr.len() - 1;
            }

            // For all other addressing modes (Absolute, Zero Page, etc.),
            // the effective_addr is calculated during the subsequent micro-ops.
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn exec(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context, instr: &Instruction) {
        match instr.mnemonic {
            // JSR, RTI, and RTS have complex, non-standard instruction cycles (micro-ops),
            // especially during stack manipulation. Their addressing phase cycles are often
            // dedicated to setup and are distinct from standard addressing modes.
            Mnemonic::JSR | Mnemonic::RTI | Mnemonic::RTS if self.index == 0 => {
                // Skip the cycles normally reserved for general addressing mode processing.
                // These instructions have their own custom micro-ops defined immediately
                // following the opcode fetch cycle (index 0).
                self.index += instr.addr_len();

                // Execute the *first* of the custom, non-addressing micro-ops.
                instr.exec(self, bus, ctx, self.index);
            }
            _ => {
                // For all other instructions, or for the remaining cycles of JSR/RTI/RTS,
                // execute the micro-op corresponding to the current cycle index.
                instr.exec(self, bus, ctx, self.index);
            }
        }

        // Move to the next cycle index for the next execution phase.
        self.index += 1;
    }

    #[inline]
    pub(crate) fn post_exec(&mut self, instr: &Instruction) {
        // Context: This function runs *after* a micro-op is executed, and self.index has
        // *already been incremented* by the previous function call's logic.
        // Therefore, index() here represents the total number of cycles executed so far
        // (excluding the Opcode Fetch cycle).
        match instr.addressing {
            Addressing::Immediate => {
                self.incr_pc();
            }
            Addressing::Absolute => {
                // JMP Absolute (3 total cycles, 2 addressing cycles after fetch):
                // The jump must occur immediately upon fetching the final address byte.
                // If addr_len() is 2, the addressing micro-ops are at index 0 and 1.
                // When index() == 2, the addressing is complete, and the next cycle is skipped.
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.index == instr.addr_len() {
                    // JMP is special: it updates the PC right after address calculation,
                    // skipping the final execution phase cycle used by most other instructions.
                    self.pc = self.effective_addr;
                }
            }

            Addressing::Indirect => {
                // JMP Indirect (5 total cycles, 4 addressing cycles after fetch):
                // Addressing micro-ops run at index 0, 1, 2, 3.
                // When index() == 4 (addr_len), the address calculation is finished.
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.index == instr.addr_len() {
                    // Similarly, JMP Indirect updates PC immediately after the 4th addressing cycle
                    // (the final address byte read, including the $XXFF bug handling).
                    self.pc = self.effective_addr;
                }
            }

            // For all other instructions, the PC is either updated later by a dedicated
            // execution micro-op, or the flow continues normally.
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn incr_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.index = 0;
        self.opcode_in_flight = None;
        self.base = 0;
        self.effective_addr = 0;
    }

    /// Accounts for a CPU bus cycle consumed externally (e.g., DMC DMA) without
    /// advancing any instruction micro-ops. This keeps cycle parity and DMA
    /// alignment consistent with hardware timing.
    pub(crate) fn account_dma_cycle(&mut self) {
        if let Some(dma) = self.oam_dma.as_mut() {
            dma.stall_cycle();
        }
    }

    fn handle_oam_dma(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context) -> bool {
        if let Some(mut dma) = self.oam_dma.take() {
            let done = dma.step(self, bus, ctx);
            self.oam_dma = if done { None } else { Some(dma) };
            return true;
        }

        if self.opcode_in_flight.is_none()
            && let Some(page) = bus.take_oam_dma_request()
        {
            let start_on_odd_cycle = (bus.cycles() & 1) == 1;
            let mut dma = OamDma::new(page, start_on_odd_cycle);
            let done = dma.step(self, bus, ctx);
            self.oam_dma = if done { None } else { Some(dma) };
            return true;
        }

        false
    }

    fn perform_interrupt(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context) {
        // Dummy read
        bus.mem_read(self.pc, self, ctx);
        bus.mem_read(self.pc, self, ctx);

        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = self.pc as u8;

        self.push(bus, ctx, pc_hi);
        self.push(bus, ctx, pc_lo);
        let kind = if self.nmi_latch {
            self.nmi_latch = false;
            IrqKind::Nmi
        } else {
            IrqKind::Irq
        };
        let status = self.p | Status::UNUSED;
        self.push(bus, ctx, status.bits());
        self.p.insert(Status::INTERRUPT);

        match kind {
            IrqKind::Nmi => {
                let lo = bus.mem_read(NMI_VECTOR_LO, self, ctx);
                let hi = bus.mem_read(NMI_VECTOR_HI, self, ctx);
                self.pc = ((hi as u16) << 8) | (lo as u16);
            }
            IrqKind::Irq => {
                let lo = bus.mem_read(IRQ_VECTOR_LO, self, ctx);
                let hi = bus.mem_read(IRQ_VECTOR_HI, self, ctx);
                self.pc = ((hi as u16) << 8) | (lo as u16);
            }
        }

        debug_assert!(self.opcode_in_flight.is_none());
        debug_assert_eq!(self.index, 0);
    }

    pub(crate) fn test_branch(&mut self, taken: bool) {
        if taken {
            if self.irq_active && !self.prev_irq_active {
                self.irq_active = false;
            }
        } else {
            // Skip add branch offset and cross page
            self.index += 2;
        }
    }

    pub(crate) const fn always_cross_page(opcode: u8, instr: &Instruction) -> bool {
        let cycle = CYCLE_TABLE[opcode as usize];
        instr.addressing.maybe_cross_page() && matches!(cycle, Cycle::Normal(_))
    }

    pub(crate) fn check_cross_page(&mut self, base: u16, addr: u16) {
        let opcode = self.opcode_in_flight.expect("opcode not set");
        let instr = &LOOKUP_TABLE[opcode as usize];
        if Self::always_cross_page(opcode, instr) {
            return;
        }
        let crossed_page = (base & 0xFF00) != (addr & 0xFF00);
        if !crossed_page {
            self.index += 1;
        }
    }

    pub(crate) fn push(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context, data: u8) {
        bus.mem_write(self.stack_addr(), data, self, ctx);
        self.s = self.s.wrapping_sub(1);
    }

    pub(crate) fn pull(&mut self, bus: &mut CpuBus<'_>, ctx: &mut Context) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.mem_read(self.stack_addr(), self, ctx)
    }

    /// Captures the current CPU registers for tracing/debugging.
    pub(crate) fn snapshot(&self) -> CpuSnapshot {
        CpuSnapshot {
            pc: self.pc,
            a: self.a,
            x: self.x,
            y: self.y,
            s: self.s,
            p: self.p.bits(),
        }
    }

    /// Returns `true` when an instruction is currently in flight.
    pub(crate) fn opcode_active(&self) -> bool {
        self.opcode_in_flight.is_some()
    }

    pub(crate) fn stack_addr(&self) -> u16 {
        STACK_ADDR | self.s as u16
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{:?} PC:{:04X} O:{:02X?} I:{} B:{:02X} E:{:04X}",
            self.a,
            self.x,
            self.y,
            self.s,
            self.p,
            self.pc,
            self.opcode_in_flight,
            self.index,
            self.base,
            self.effective_addr
        )
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        // 寄存器区
        writeln!(f, "╔═════════════════════════════════════════╗")?;
        writeln!(f, "║                 CPU State               ║")?;
        writeln!(f, "╠══════╤══════╤══════╤══════╤══════╤══════╣")?;
        writeln!(f, "║  A   │  X   │  Y   │  S   │  PC  │ OPC  ║")?;
        writeln!(f, "╠══════╤══════╤══════╤══════╤══════╤══════╣")?;
        let opcode = match self.opcode_in_flight {
            Some(opcode) => {
                format!("{:02X}", opcode)
            }
            None => "  ".to_string(),
        };
        writeln!(
            f,
            "║ {:02X}   │ {:02X}   │ {:02X}   │ {:02X}   │ {:04X} │ {}   ║",
            self.a, self.x, self.y, self.s, self.pc, opcode
        )?;

        // 状态标志
        writeln!(f, "╠══════╧══════╧══════╧══════╧══════╧══════╣")?;
        writeln!(f, "║ Flags: {}  ║ ", self.p)?;

        // 地址信息
        writeln!(f, "╠═════════════════════════════════════════╣")?;
        writeln!(
            f,
            "║ base: {:02X}|effective_addr: {:04X}│index: {:02X} ║",
            self.base, self.effective_addr, self.index
        )?;
        writeln!(f, "╚═════════════════════════════════════════╝")?;

        Ok(())
    }
}
