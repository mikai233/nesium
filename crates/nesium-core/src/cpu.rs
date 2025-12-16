use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, OnceLock};

use crate::bus::{CpuBus, STACK_ADDR};
use crate::context::Context;
use crate::cpu::addressing::Addressing;
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
mod instruction;
mod irq;
mod lookup;
mod micro_op;
mod mnemonic;
mod status;
mod timing;

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

static CPU_OPCODE_LOG: OnceLock<Option<Mutex<BufWriter<std::fs::File>>>> = OnceLock::new();

#[inline]
fn cpu_opcode_log_write(cycle: u64, pc: u16, opcode: u8, addr_val: u8) {
    let log = CPU_OPCODE_LOG.get_or_init(|| {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("/Users/mikai/RustroverProjects/nesium/nesium_cpu_opcode.log")
            .ok()
            .map(|f| Mutex::new(BufWriter::with_capacity(256 * 1024, f)))
    });

    if let Some(writer) = log {
        if let Ok(mut w) = writer.lock() {
            // Keep the same format as the C++ logger: "cycle=<u64> opcode=<02X>"
            let _ = writeln!(
                w,
                "cycle={} pc={:04X} opcode={:02X} 0200={:02X}",
                cycle, pc, opcode, addr_val
            )
            .unwrap();
        }
    }
}

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
    // ===== Architectural registers (6502 visible state) =====
    pub(crate) a: u8,     // Accumulator (A)
    pub(crate) x: u8,     // Index register X
    pub(crate) y: u8,     // Index register Y
    pub(crate) s: u8,     // Stack pointer (offset in page $01xx)
    pub(crate) p: Status, // Processor status flags (NV-BDIZC)
    pub(crate) pc: u16,   // Program counter

    // ===== Current instruction / microcycle state =====
    /// Opcode currently being executed (set once fetched; cleared when instruction completes).
    pub(crate) opcode_in_flight: Option<u8>,

    /// Micro-cycle / step index for the current instruction.
    /// e.g. 0 = first cycle after opcode fetch, 1 = next cycle, ...
    /// This drives your per-cycle state machine.
    pub(crate) step: u8,

    // ===== Addressing / bus temporaries (per-instruction scratch) =====
    /// Scratch byte used across micro-ops (often holds low byte, zp addr, fetched operand, etc.).
    pub(crate) tmp: u8,

    /// Effective address resolved for the current bus operation / micro-op.
    pub(crate) effective_addr: u16,

    // ===== Interrupt state (IRQ/NMI) =====
    pub(crate) irq_latch: IrqSource,
    pub(crate) prev_irq_active: bool,
    pub(crate) irq_active: bool,
    pub(crate) irq_enable_mask: IrqSource,

    /// Previous sampled NMI line level (for rising-edge detection).
    pub(crate) prev_nmi_level: bool,
    /// NMI pending flag set on rising edge, consumed when NMI is taken.
    pub(crate) nmi_latch: bool,
    /// Previous cycle's pending flag (used to align NMI timing with instruction boundary).
    pub(crate) prev_nmi_latch: bool,

    // ===== DMA =====
    pub(crate) oam_dma: Option<OamDma>,
}

impl Cpu {
    /// Create a new CPU instance with default values.
    /// Does not automatically fetch the reset vector — call `reset()` for that.
    pub(crate) fn new() -> Self {
        Self {
            a: 0x00,              // Accumulator
            x: 0x00,              // X register
            y: 0x00,              // Y register
            s: 0xFD,              // Stack pointer after reset
            p: Status::INTERRUPT, // IRQ disabled
            pc: 0x0000,           // Will be set by reset vector
            opcode_in_flight: None,
            irq_latch: IrqSource::empty(),
            prev_irq_active: false,
            irq_active: false,
            irq_enable_mask: IrqSource::empty(),
            prev_nmi_level: false,
            nmi_latch: false,
            prev_nmi_latch: false,
            step: 0,
            tmp: 0,
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
        self.step = 0;
        self.tmp = 0;
        self.effective_addr = 0;
        self.oam_dma = None;

        // The CPU takes 8 cycles before it starts executing the ROM's code
        // after a reset/power-up (Mesen does this via 8 Start/End cycles).
        for _ in 0..8 {
            self.begin_cycle(true, bus, ctx);
            self.end_cycle(true, bus, ctx);
        }
    }

    pub(crate) fn step(&mut self, bus: &mut CpuBus, ctx: &mut Context) {
        if self.handle_oam_dma(bus, ctx) {
            return;
        }

        match self.opcode_in_flight {
            Some(opcode) => {
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.execute_step(bus, ctx, instr);
                self.finalize_step(instr);
                if self.step >= instr.len() {
                    self.clear_instruction_state();
                }
            }
            None => {
                if self.prev_irq_active || self.prev_nmi_latch {
                    self.perform_interrupt(bus, ctx);
                } else {
                    // let start_pc = self.pc;
                    // let start_cycles = bus.cycles();
                    // let addr_val = bus.ram[0x0200];
                    let opcode = self.fetch_u8(bus, ctx);
                    // if start_cycles < 20_000_000 {
                    //     cpu_opcode_log_write(start_cycles, start_pc, opcode, addr_val);
                    // }
                    self.opcode_in_flight = Some(opcode);
                    let instr = &LOOKUP_TABLE[opcode as usize];
                    // self.log_trace_line(bus, ctx, start_pc, start_cycles, instr);
                    self.prepare_step(instr);
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
    pub(crate) fn dummy_read(&mut self, bus: &mut CpuBus, ctx: &mut Context) -> u8 {
        bus.mem_read(self.pc, self, ctx)
    }

    #[inline]
    pub(crate) fn dummy_read_at(&mut self, addr: u16, bus: &mut CpuBus, ctx: &mut Context) {
        bus.mem_read(addr, self, ctx);
    }

    #[inline]
    pub(crate) fn dummy_write_at(
        &mut self,
        addr: u16,
        data: u8,
        bus: &mut CpuBus,
        ctx: &mut Context,
    ) {
        bus.mem_write(addr, data, self, ctx);
    }

    pub(crate) fn fetch_u8(&mut self, bus: &mut CpuBus, ctx: &mut Context) -> u8 {
        let v = bus.mem_read(self.pc, self, ctx);
        self.inc_pc();
        v
    }

    pub(crate) fn fetch_u16(&mut self, bus: &mut CpuBus, ctx: &mut Context) -> u16 {
        let lo = self.fetch_u8(bus, ctx) as u16;
        let hi = self.fetch_u8(bus, ctx) as u16;
        (hi << 8) | lo
    }

    #[inline]
    pub(crate) fn prepare_step(&mut self, instr: &Instruction) {
        match instr.addressing {
            Addressing::Accumulator => {
                // Accumulator addressing operates on register A directly and does not
                // perform any operand fetch/address calculation micro-cycles.
                //
                // To reuse the normal per-step execution dispatch, jump `step` straight
                // to the final execution micro-op for this instruction.
                self.step = instr.len() - 1;
            }

            Addressing::Immediate => {
                // Immediate addressing reads the operand byte at the current PC.
                //
                // We precompute `effective_addr` as the location of the immediate value
                // so later micro-ops can fetch the operand through a unified path
                // (`effective_addr`), regardless of addressing mode.
                //
                // Note: PC is advanced in `finalize_step` after the immediate byte is consumed.
                self.effective_addr = self.pc;
            }

            _ => {
                // All other addressing modes compute `effective_addr` through their
                // dedicated addressing micro-ops over subsequent steps.
            }
        }
    }

    #[inline]
    pub(crate) fn execute_step(
        &mut self,
        bus: &mut CpuBus,
        ctx: &mut Context,
        instr: &Instruction,
    ) {
        match instr.mnemonic {
            // JSR has a non-standard micro-cycle layout.
            //
            // Unlike most instructions, JSR does not use the generic addressing
            // micro-ops to compute its target address. Instead, its early cycles
            // are dedicated to stack manipulation and control-flow setup.
            //
            // At step == 0 (immediately after opcode fetch), skip the generic
            // addressing phase and jump directly to JSR's first custom micro-op.
            Mnemonic::JSR if self.step == 0 => {
                // Skip over the cycles that would normally be used for address calculation.
                self.step += instr.addr_len();

                // Execute the first JSR-specific micro-op.
                instr.exec(self, bus, ctx, self.step);
            }

            _ => {
                // Default path:
                // Execute the micro-op corresponding to the current step index.
                // This applies to all non-JSR instructions, as well as subsequent
                // JSR cycles after the initial special-case handling.
                instr.exec(self, bus, ctx, self.step);
            }
        }

        // Advance to the next micro-cycle for the following tick.
        self.step += 1;
    }

    #[inline]
    pub(crate) fn finalize_step(&mut self, instr: &Instruction) {
        // This hook runs after a micro-op has completed and the step counter
        // has advanced. At this point, `self.step` represents the number of
        // micro-cycles executed so far since the opcode fetch.
        //
        // Most instructions either update the PC as part of their execution
        // micro-ops or let it advance naturally. JMP is a special case: the
        // program counter is updated immediately after address resolution,
        // without executing a final "execute" cycle.

        match instr.addressing {
            Addressing::Immediate => {
                // Immediate addressing consumes the operand byte directly.
                // Advance PC past the immediate value once the micro-op completes.
                self.inc_pc();
            }

            Addressing::Absolute => {
                // JMP Absolute timing:
                // - Opcode fetch
                // - Address low byte
                // - Address high byte
                //
                // After the final address byte is fetched (addr_len micro-cycles),
                // the effective address is complete and the jump is taken immediately.
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.step == instr.addr_len() {
                    self.pc = self.effective_addr;
                }
            }

            Addressing::Indirect => {
                // JMP Indirect timing:
                // - Opcode fetch
                // - Pointer low byte
                // - Pointer high byte
                // - Read target low byte
                // - Read target high byte (with $xxFF wraparound behavior)
                //
                // As with absolute JMP, the PC is updated immediately after the
                // final address resolution cycle.
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.step == instr.addr_len() {
                    self.pc = self.effective_addr;
                }
            }

            // For all other addressing modes and instructions, no finalization
            // is required here.
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    #[inline]
    pub(crate) fn clear_instruction_state(&mut self) {
        self.step = 0;
        self.opcode_in_flight = None;
        self.tmp = 0;
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

    fn perform_interrupt(&mut self, bus: &mut CpuBus, ctx: &mut Context) {
        self.dummy_read(bus, ctx);
        self.dummy_read(bus, ctx);

        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = self.pc as u8;

        self.push_stack(bus, ctx, pc_hi);
        self.push_stack(bus, ctx, pc_lo);
        let kind = if self.nmi_latch {
            self.nmi_latch = false;
            IrqKind::Nmi
        } else {
            IrqKind::Irq
        };
        let status = self.p | Status::UNUSED;
        self.push_stack(bus, ctx, status.bits());
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
        debug_assert_eq!(self.step, 0);
    }

    pub(crate) fn test_branch(&mut self, taken: bool) {
        if taken {
            if self.irq_active && !self.prev_irq_active {
                self.irq_active = false;
            }
        } else {
            // Skip add branch offset and cross page
            self.step += 2;
        }
    }

    pub(crate) fn skip_optional_dummy_read_cycle(&mut self, base: u16, addr: u16) {
        let opcode = self.opcode_in_flight.expect("opcode not set");
        if Addressing::forces_dummy_read_cycle(opcode) {
            return;
        }
        if !Addressing::page_crossed(base, addr) {
            self.step += 1;
        }
    }

    #[inline]
    pub(crate) fn push_stack(&mut self, bus: &mut CpuBus, ctx: &mut Context, data: u8) {
        bus.mem_write(self.stack_addr(), data, self, ctx);
        self.s = self.s.wrapping_sub(1);
    }

    #[inline]
    pub(crate) fn pop_stack(&mut self, bus: &mut CpuBus, ctx: &mut Context) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.mem_read(self.stack_addr(), self, ctx)
    }

    #[inline]
    pub(crate) fn push_stack_u16(&mut self, bus: &mut CpuBus, ctx: &mut Context, data: u16) {
        // 6502 pushes high byte first, then low byte
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.push_stack(bus, ctx, hi);
        self.push_stack(bus, ctx, lo);
    }

    #[inline]
    pub(crate) fn pop_stack_u16(&mut self, bus: &mut CpuBus, ctx: &mut Context) -> u16 {
        // 6502 pops low byte first, then high byte
        let lo = self.pop_stack(bus, ctx) as u16;
        let hi = self.pop_stack(bus, ctx) as u16;
        (hi << 8) | lo
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

    #[inline]
    fn log_trace_line(
        &mut self,
        bus: &mut CpuBus<'_>,
        ctx: &mut Context,
        start_pc: u16,
        start_cycles: u64,
        instr: &Instruction,
    ) {
        if !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }

        let operand_len = instr.addressing.operand_len();
        let mut operands = [0u8; 2];
        for i in 0..operand_len {
            let addr = start_pc.wrapping_add(1 + i as u16);
            operands[i] = bus.peek(addr, self, ctx);
        }
        let operand_text =
            Self::format_operand(instr.addressing, &operands[..operand_len], start_pc);

        let mut inst_text = format!("{:?}", instr.mnemonic);
        if !operand_text.is_empty() {
            inst_text.push(' ');
            inst_text.push_str(&operand_text);
        }

        let log_line = format!(
            "{:04X}   {:<24} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{} Cycle:{}",
            start_pc,
            inst_text,
            self.a,
            self.x,
            self.y,
            self.s,
            Self::format_status(self.p),
            start_cycles
        );
        tracing::debug!("{log_line}");
    }

    #[inline]
    fn format_status(status: Status) -> String {
        fn letter(set: bool, ch: char) -> char {
            if set { ch } else { ch.to_ascii_lowercase() }
        }

        let mut out = String::with_capacity(8);
        out.push(letter(status.n(), 'N'));
        out.push(letter(status.v(), 'V'));
        out.push('-');
        out.push('-');
        out.push(letter(status.d(), 'D'));
        out.push(letter(status.i(), 'I'));
        out.push(letter(status.z(), 'Z'));
        out.push(letter(status.c(), 'C'));
        out
    }

    #[inline]
    fn format_operand(addressing: Addressing, operands: &[u8], start_pc: u16) -> String {
        match addressing {
            Addressing::Implied => String::new(),
            Addressing::Accumulator => "A".to_string(),
            Addressing::Immediate => format!("#${:02X}", operands[0]),
            Addressing::Absolute => format!(
                "${:04X}",
                u16::from(operands[0]) | (u16::from(*operands.get(1).unwrap_or(&0)) << 8)
            ),
            Addressing::AbsoluteX => format!(
                "${:04X},X",
                u16::from(operands[0]) | (u16::from(*operands.get(1).unwrap_or(&0)) << 8)
            ),
            Addressing::AbsoluteY => format!(
                "${:04X},Y",
                u16::from(operands[0]) | (u16::from(*operands.get(1).unwrap_or(&0)) << 8)
            ),
            Addressing::Indirect => format!(
                "(${:04X})",
                u16::from(operands[0]) | (u16::from(*operands.get(1).unwrap_or(&0)) << 8)
            ),
            Addressing::ZeroPage => format!("${:02X}", operands[0]),
            Addressing::ZeroPageX => format!("${:02X},X", operands[0]),
            Addressing::ZeroPageY => format!("${:02X},Y", operands[0]),
            Addressing::IndirectX => format!("(${:02X},X)", operands[0]),
            Addressing::IndirectY => format!("(${:02X}),Y", operands[0]),
            Addressing::Relative => {
                let offset = operands[0] as i8 as i16;
                let base = start_pc.wrapping_add(2);
                let target = if offset < 0 {
                    base.wrapping_sub((-offset) as u16)
                } else {
                    base.wrapping_add(offset as u16)
                };
                format!("${:04X}", target)
            }
        }
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
            self.step,
            self.tmp,
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
            self.tmp, self.effective_addr, self.step
        )?;
        writeln!(f, "╚═════════════════════════════════════════╝")?;

        Ok(())
    }
}
