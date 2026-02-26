use std::fmt::{Debug, Display};

use crate::apu::Apu;
use crate::bus::{CpuBus, DmcDmaEvent, STACK_ADDR};
use crate::context::Context;
use crate::cpu::addressing::Addressing;
use crate::cpu::instruction::Instruction;
use crate::cpu::irq::IrqKind;
use crate::cpu::lookup::LOOKUP_TABLE;
use crate::cpu::mnemonic::Mnemonic;
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
mod mnemonic;
mod status;
mod timing;

pub(crate) use irq::IrqSource;
pub(crate) use status::Status;

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

/// Unified DMA controller state to match Mesen's implementation.
/// Handles the interleaving of OAM and DMC DMA, including the specific "Halt" and "Dummy Read" cycle stealing behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub(crate) struct DmaController {
    // === Control Flags (Mesen: _needHalt, _needDummyRead) ===
    pub(crate) halt_needed: bool,
    pub(crate) dummy_read_needed: bool, // Specific to DMC start-up alignment

    // === DMC DMA State (Mesen: _dmcDmaRunning, _abortDmcDma) ===
    pub(crate) dmc_active: bool,
    pub(crate) dmc_abort_pending: bool,
    pub(crate) dmc_addr: u16,
    pub(crate) is_dmc_read: bool, // Mesen: _isDmcDmaRead

    // === OAM DMA State (Mesen: _spriteDmaTransfer) ===
    pub(crate) oam_active: bool,
    pub(crate) oam_page: u8,
    pub(crate) oam_cycle_counter: u16, // Mesen: spriteDmaCounter (0-511)
    pub(crate) oam_latch: u8,          // Temporary storage between read and write cycles
}

impl DmaController {
    // Mesen: StartDmcTransfer
    #[inline]
    fn request_dmc(&mut self, addr: u16) {
        self.dmc_active = true;
        self.dmc_abort_pending = false;
        self.dummy_read_needed = true;
        self.halt_needed = true;
        self.dmc_addr = addr;
    }

    // Mesen: RunDMATransfer
    #[inline]
    fn request_oam(&mut self, page: u8) {
        self.oam_active = true;
        self.oam_page = page;
        self.oam_cycle_counter = 0;
        self.halt_needed = true;
    }

    #[inline]
    fn is_active(&self) -> bool {
        self.dmc_active || self.oam_active || self.halt_needed
    }

    // Mesen: StopDmcTransfer
    #[inline]
    fn stop_dmc(&mut self) {
        if !self.dmc_active {
            return;
        }

        if self.halt_needed {
            // If interrupted before the halt cycle starts, cancel DMA completely
            self.dmc_active = false;
            self.dmc_abort_pending = false;
            self.dummy_read_needed = false;
            self.halt_needed = false;
        } else {
            // Abort DMA if possible (only appears possible within the first cycle of DMA)
            self.dmc_abort_pending = true;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cpu {
    // ===== Architectural registers (6502 visible state) =====
    pub a: u8,            // Accumulator (A)
    pub x: u8,            // Index register X
    pub y: u8,            // Index register Y
    pub s: u8,            // Stack pointer (offset in page $01xx)
    pub(crate) p: Status, // Processor status flags (NV-BDIZC)
    pub pc: u16,          // Program counter

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
    pub(crate) dma: DmaController,
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
            dma: DmaController::default(),
        }
    }

    /// Returns the processor status flags (P register) as a raw u8.
    pub fn status_bits(&self) -> u8 {
        self.p.bits()
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
    pub(crate) fn reset(&mut self, bus: &mut CpuBus, kind: ResetKind, ctx: &mut Context) {
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
        self.dma = DmaController::default();

        // The CPU takes 8 cycles before it starts executing the ROM's code
        // after a reset/power-up (Mesen does this via 8 Start/End cycles).
        for _ in 0..8 {
            self.begin_cycle(true, bus, ctx);
            self.end_cycle(true, bus, ctx);
        }
    }

    pub(crate) fn step(&mut self, bus: &mut CpuBus, ctx: &mut Context) {
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
                    let opcode = self.fetch_u8(bus, ctx);
                    self.opcode_in_flight = Some(opcode);
                    let instr = &LOOKUP_TABLE[opcode as usize];
                    self.prepare_step(instr);
                }
            }
        }
    }

    pub(crate) fn begin_cycle(&mut self, read_phase: bool, bus: &mut CpuBus, ctx: &mut Context) {
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
        // Run one APU CPU-cycle tick; DMA requests are queued on the bus.
        Apu::step(bus, self, ctx);
    }

    pub(crate) fn end_cycle(&mut self, read_phase: bool, bus: &mut CpuBus, ctx: &mut Context) {
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
        // No-op: DMA stolen cycles are executed explicitly through `handle_dma`.
        // This hook remains for compatibility with older code paths.
    }

    /// Handles pending DMA transfers based on Mesen's `ProcessPendingDma` logic.
    ///
    /// IMPORTANT architectural notes (matching Mesen):
    /// - This is called from `CpuBus::mem_read()` *before* the CPU read cycle starts.
    /// - If DMA is pending/active, this function will clock **all** DMA cycles until DMA is no
    ///   longer stealing cycles. Only then may the CPU's requested read proceed.
    /// - DMA bus accesses MUST bypass `mem_read/mem_write` (otherwise we'd recurse back into DMA).
    ///   Use `bus.dma_read/dma_write`.
    ///
    /// TODO parity with Mesen:
    /// - PAL gating: DMA can only start on opcode fetch (`ExecOpCode`) on PAL.
    /// - 4016/4017 input timing quirks: `skipFirstInputClock`, `skipDummyReads`, and HVC001 behavior.
    /// - Internal-reg conflict glitch: open bus masks + merging for 4016/4017, and 4015 side effects.
    /// - Exact address used for dummy reads (should be the CPU's pending read address / opType-specific).
    /// - DMC buffer delivery: feed the DMC read byte into APU (SetDmcReadBuffer).
    pub(crate) fn handle_dma(&mut self, addr: u16, bus: &mut CpuBus, ctx: &mut Context) {
        // Drain bus mailbox once at entry.
        if let Some(evt) = bus.take_dmc_dma_event() {
            match evt {
                DmcDmaEvent::Request { addr } => self.dma.request_dmc(addr),
                DmcDmaEvent::Abort => self.dma.stop_dmc(),
            }
        }

        // Start OAM DMA only at instruction boundary (before opcode fetch), mirroring Mesen.
        if self.opcode_in_flight.is_none()
            && !self.dma.oam_active
            && let Some(page) = bus.take_oam_dma()
        {
            self.dma.request_oam(page);
        }

        // Fast exit if DMA isn't active.
        if !self.dma.is_active() {
            return;
        }

        // Mesen local state.
        let mut prev_read_address: u16 = addr;
        let enable_internal_reg_reads: bool = (addr & 0xFFE0) == 0x4000;

        // Helper: clock a stolen DMA cycle without performing a bus read.
        // This is required to model Mesen's behavior where certain dummy reads to $4016/$4017
        // are skipped to avoid extra controller side effects, while still consuming time.
        #[inline(always)]
        fn dma_idle(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context) {
            cpu.begin_cycle(true, bus, ctx);
            cpu.end_cycle(true, bus, ctx);
        }

        // TODO: plumb console type + region via bus/config. This controls Famicom vs NES behavior.
        // Mesen: `isNesBehavior = ConsoleType != Hvc001`.
        let is_nes_behavior: bool = true;

        // Mesen: skipFirstInputClock
        // If the CPU is halted while reading $4016/$4017 and DMC DMA reads the same *internal* input reg,
        // the controller won't see the first dummy read because /OE stays active.
        let skip_first_input_clock: bool = enable_internal_reg_reads
            && self.dma.dmc_active
            && (addr == 0x4016 || addr == 0x4017)
            && ((self.dma.dmc_addr & 0x1F) == (addr & 0x1F));

        // Mesen: skipDummyReads
        // On NES/AV Famicom, only the first dummy/idle read to $4016/$4017 causes side effects;
        // further dummy reads during DMA are effectively hidden.
        let skip_dummy_reads: bool = is_nes_behavior && (addr == 0x4016 || addr == 0x4017);

        // === Halt cycle (Mesen: initial StartCpuCycle(true) + optional read + EndCpuCycle(true)) ===
        // Mesen: ProcessPendingDma exits early when !_needHalt.
        // In our model, `halt_needed` is the shared entry latch for both OAM and DMC DMA.
        if self.dma.halt_needed {
            self.dma.halt_needed = false;

            // Mesen performs the halt-cycle dummy read on the address the CPU was about to read.
            // NOTE: must bypass `mem_read` to avoid recursion.
            // Mesen special-case: when aborting DMC and the CPU will read $4016/$4017 next (NES behavior),
            // skip the memory access to avoid creating two visible reads for the controllers.
            let skip_halt_mem_access = (self.dma.dmc_abort_pending
                && is_nes_behavior
                && (addr == 0x4016 || addr == 0x4017))
                || skip_first_input_clock;

            if skip_halt_mem_access {
                dma_idle(self, bus, ctx);
            } else {
                bus.dma_read(addr, self, ctx);
            }

            // If DMC was aborted during/just before the halt cycle, clear it now.
            // Mesen clears `_needDummyRead` and may return early if OAM isn't about to start.
            if self.dma.dmc_abort_pending {
                self.dma.dmc_active = false;
                self.dma.dmc_abort_pending = false;

                if !self.dma.oam_active {
                    self.dma.dummy_read_needed = false;
                    return;
                }
            }
        }

        while self.dma.dmc_active || self.dma.oam_active {
            // Allow late aborts to be consumed while DMA is running (closer to Mesen).
            if let Some(evt) = bus.take_dmc_dma_event() {
                if let DmcDmaEvent::Abort = evt {
                    self.dma.stop_dmc();
                } else {
                    // Request while running is unusual, but keep last-wins for now.
                    if let DmcDmaEvent::Request { addr } = evt {
                        self.dma.request_dmc(addr);
                    }
                }
            }

            // Match Mesen: classify GET/PUT from the current CPU cycle counter
            // *before* the next DMA cycle starts.
            let get_cycle = (bus.cycles() & 1) == 0;

            // Snapshot DMC readiness BEFORE we clear `dummy_read_needed` this cycle.
            let dmc_ready_pre = self.dma.dmc_active
                && !self.dma.halt_needed
                && !self.dma.dummy_read_needed
                && !self.dma.dmc_abort_pending;

            // Mesen's `processCycle` clears abort/halt/dummy flags and then starts the timed cycle.
            // Here we model the flag clears first; the timed cycle is performed by `dma_read/dma_write`.
            if self.dma.dmc_abort_pending {
                // Abort window: stop DMC and clear setup flags.
                self.dma.dmc_active = false;
                self.dma.dmc_abort_pending = false;
                self.dma.dummy_read_needed = false;
                self.dma.halt_needed = false;
            } else if self.dma.halt_needed {
                self.dma.halt_needed = false;
            } else if self.dma.dummy_read_needed {
                self.dma.dummy_read_needed = false;
            }

            if get_cycle {
                // === GET cycle (read phase) ===
                if dmc_ready_pre {
                    // DMC DMA read takes priority over OAM read.
                    self.dma.is_dmc_read = true;
                    let dmc_addr = self.dma.dmc_addr;
                    let val = self.process_dma_read(
                        bus,
                        ctx,
                        dmc_addr,
                        &mut prev_read_address,
                        enable_internal_reg_reads,
                        is_nes_behavior,
                    );
                    self.dma.is_dmc_read = false;

                    self.dma.dmc_active = false;
                    self.dma.dmc_abort_pending = false;

                    bus.apu.finish_dma_fetch(val);

                    // This cycle was fully consumed by DMA; continue the while-loop.
                    continue;
                }

                if self.dma.oam_active {
                    // OAM DMA alternates read/write. Even counter => read.
                    if (self.dma.oam_cycle_counter & 1) == 0 {
                        // Derive the low byte of the OAM DMA read address from the DMA cycle counter.
                        // Even counter values are the read phase. Completed reads so far = (counter + 1) / 2.
                        let sprite_read_addr: u8 = self.dma.oam_cycle_counter.div_ceil(2) as u8;
                        let src_addr =
                            ((self.dma.oam_page as u16) << 8) | (sprite_read_addr as u16);

                        let v = self.process_dma_read(
                            bus,
                            ctx,
                            src_addr,
                            &mut prev_read_address,
                            enable_internal_reg_reads,
                            is_nes_behavior,
                        );

                        self.dma.oam_latch = v;
                        self.dma.oam_cycle_counter += 1;
                        continue;
                    }

                    // Alignment: waiting for the write phase, burn a dummy/idle DMA read.
                    if skip_dummy_reads {
                        dma_idle(self, bus, ctx);
                    } else {
                        bus.dma_read(addr, self, ctx);
                    }
                    continue;
                }

                // DMC running but not ready and no OAM: dummy/idle cycle.
                if skip_dummy_reads {
                    dma_idle(self, bus, ctx);
                } else {
                    bus.dma_read(addr, self, ctx);
                }
                continue;
            } else {
                // === PUT cycle (write/alignment phase) ===
                if self.dma.oam_active && (self.dma.oam_cycle_counter & 1) != 0 {
                    // OAM write cycle.
                    // NOTE: must bypass `mem_write` to avoid recursion/ordering differences.
                    bus.dma_write(PpuRegister::OamData.addr(), self.dma.oam_latch, self, ctx);

                    self.dma.oam_cycle_counter += 1;
                    if self.dma.oam_cycle_counter >= 512 {
                        self.dma.oam_active = false;
                    }
                    continue;
                }

                // Alignment: burn a dummy/idle cycle.
                if skip_dummy_reads {
                    dma_idle(self, bus, ctx);
                } else {
                    bus.dma_read(addr, self, ctx);
                }
                continue;
            }
        }
    }

    // Mesen: ProcessDmaRead
    #[inline]
    fn process_dma_read(
        &mut self,
        bus: &mut CpuBus,
        ctx: &mut Context,
        dma_addr: u16,
        prev_read_address: &mut u16,
        enable_internal_reg_reads: bool,
        is_nes_behavior: bool,
    ) -> u8 {
        // This models Mesen's "CPU internal register conflict" glitch during DMA.
        //
        // Mesen runs this logic within a single DMA cycle; "internal" and
        // "external" reads can both occur without stealing extra cycles.
        self.begin_cycle(true, bus, ctx);

        let value = if !enable_internal_reg_reads {
            let v = if (0x4000..=0x401F).contains(&dma_addr) {
                // Nothing responds on $4000-$401F on the external bus.
                bus.open_bus.sample()
            } else {
                bus.read(dma_addr, self, ctx)
            };
            *prev_read_address = dma_addr;
            v
        } else {
            // Internal-reg glitch path: CPU reads from internal APU/Input regs
            // regardless of the DMA address.
            let internal_addr = 0x4000 | (dma_addr & 0x1F);
            let is_same_address = internal_addr == dma_addr;

            let v = match internal_addr {
                0x4015 => {
                    // Side effect matches Mesen: reading $4015 can clear frame IRQ.
                    let read_value = bus.read(internal_addr, self, ctx);
                    if !is_same_address {
                        // Trigger external read on the bus as well.
                        let _ = bus.read(dma_addr, self, ctx);
                    }
                    read_value
                }
                0x4016 | 0x4017 => {
                    // On NES/AV Famicom, repeated reads of the same input register
                    // can be hidden from controllers during this glitch path.
                    let consecutive_same = is_nes_behavior && *prev_read_address == internal_addr;
                    let mut read_value = if consecutive_same {
                        bus.open_bus.sample()
                    } else {
                        bus.read(internal_addr, self, ctx)
                    };

                    if !is_same_address {
                        let external_value = bus.read(dma_addr, self, ctx);
                        let open_bus_mask = Self::controller_open_bus_mask(internal_addr - 0x4016);
                        read_value = (external_value & open_bus_mask)
                            | ((read_value & !open_bus_mask) & (external_value & !open_bus_mask));
                    }

                    read_value
                }
                _ => bus.read(dma_addr, self, ctx),
            };

            // Mesen updates prevReadAddress with internalAddr on this path.
            *prev_read_address = internal_addr;
            v
        };

        self.end_cycle(true, bus, ctx);
        value
    }

    #[inline]
    fn controller_open_bus_mask(port: u16) -> u8 {
        // Match Mesen's default NES-001 behavior for now.
        // TODO: plumb console model and use console-specific masks.
        match port {
            0 => 0xE0,
            1 => 0xE0,
            _ => 0xE0,
        }
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

        // Match hardware/Mesen behavior: the first opcode in an IRQ/NMI handler
        // must run before a newly-latched NMI can immediately retrigger.
        self.prev_nmi_latch = false;

        debug_assert!(self.opcode_in_flight.is_none());
        debug_assert_eq!(self.step, 0);
    }

    pub(crate) fn apply_branch_decision(&mut self, taken: bool) {
        if taken {
            if self.irq_active && !self.prev_irq_active {
                self.irq_active = false;
            }
        } else {
            // Skip add branch offset and cross page
            self.step += 2;
        }
    }

    /// Compute the intermediate branch target used on the first taken-branch cycle (T2*).
    ///
    /// 6502 branch timing uses an address where the signed offset is added to the low byte
    /// of PC (already pointing to the next instruction, i.e. PC+2) *without* carrying into
    /// the high byte. This matches the datasheet's:
    ///   PC + 2 + offset (w/o carry)
    #[inline]
    pub(crate) fn branch_target_wo_carry(pc_next: u16, offset: i8) -> u16 {
        let lo = (pc_next as u8).wrapping_add(offset as u8);
        (pc_next & 0xFF00) | (lo as u16)
    }

    /// Execute the taken-branch bookkeeping for cycle T2*:
    ///
    /// - Perform the required dummy/prefetch read at the "w/o carry" target address.
    /// - Commit the final branch target PC (full 16-bit add).
    /// - Optionally skip the extra cycle T3** if the branch does not cross a page boundary
    ///   (unless this opcode forces the extra cycle).
    ///
    /// The caller typically provides the additional T3** cycle as a separate step that reads
    /// from the final PC when needed.
    #[inline]
    pub(crate) fn branch_taken_cycle(&mut self, bus: &mut CpuBus, ctx: &mut Context, offset: i8) {
        // PC after fetching the offset byte; should point at the next instruction (PC+2).
        let pc_next = self.pc;

        // T2*: dummy/prefetch read at PC+2+offset (w/o carry).
        let pc_wo_carry = Self::branch_target_wo_carry(pc_next, offset);
        self.dummy_read_at(pc_wo_carry, bus, ctx);

        // Commit final branch target using full 16-bit addition (with carry into high byte).
        let pc_final = pc_next.wrapping_add(offset as u16);
        self.pc = pc_final;

        // If no page cross, skip T3** (extra cycle). Some opcodes may force it regardless.
        self.skip_optional_dummy_read_cycle(pc_next, pc_final);
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
        bus: &mut CpuBus,
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
