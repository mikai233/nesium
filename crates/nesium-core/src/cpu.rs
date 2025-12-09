use std::fmt::{Debug, Display};

use crate::bus::{Bus, STACK_ADDR};
use crate::cpu::addressing::Addressing;
use crate::cpu::cycle::{CYCLE_TABLE, Cycle};
use crate::cpu::instruction::Instruction;
use crate::cpu::lookup::LOOKUP_TABLE;
use crate::cpu::micro_op::MicroOp;
use crate::cpu::mnemonic::Mnemonic;
use crate::cpu::status::Status;
use crate::memory::cpu as cpu_mem;
use crate::memory::cpu::{RESET_VECTOR_HI, RESET_VECTOR_LO};
use crate::memory::ppu::{self as ppu_mem, Register as PpuRegister};
mod status;

pub mod addressing;
mod cycle;
mod instruction;
mod lookup;
mod micro_op;
mod mnemonic;

// pub static CpuPtr: AtomicPtr<Cpu> = AtomicPtr::new(std::ptr::null_mut());

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
struct OamDma {
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
    fn step(&mut self, bus: &mut dyn Bus) -> bool {
        if self.dummy_cycles > 0 {
            self.dummy_cycles -= 1;
            return false;
        }

        if self.read_phase {
            let addr = ((self.page as u16) << 8) | self.offset;
            self.data_latch = bus.mem_read(addr);
            self.read_phase = false;
            return false;
        }

        bus.mem_write(PpuRegister::OamData.addr(), self.data_latch);
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

    opcode_in_flight: Option<u8>,
    /// Effective I flag used for interrupt gating (true = IRQs masked).
    irq_masked: bool,
    /// Pending update to the effective IRQ mask (I flag), to be applied at the next
    /// instruction boundary when no opcode is in flight.
    pending_irq_mask: Option<bool>,
    /// Suppress servicing maskable IRQs for the next instruction boundary.
    /// Used to model the 6502 behaviour where a pending IRQ is not taken
    /// until one instruction after CLI/PLP clear the I flag.
    irq_inhibit_next: bool,
    /// Allow a single IRQ even though the effective I flag is set.
    /// This approximates the behaviour of SEI/PLP where a pending IRQ
    /// is still taken "just after" the instruction that sets I.
    allow_irq_once: bool,
    /// Marks a taken branch so we can defer IRQ if it does not cross a page.
    branch_taken_defer_irq: bool,
    /// Previous sampled NMI line level (for edge detection).
    prev_nmi_line: bool,
    /// NMI pending flag set on rising edge, consumed when NMI is taken.
    nmi_pending: bool,
    /// Previous cycle's pending flag, used to delay NMI by one instruction boundary.
    prev_nmi_pending: bool,
    index: u8,
    base: u8,
    effective_addr: u16,
    oam_dma: Option<OamDma>,
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
            irq_masked: true, // Matches initial I=1 in status (IRQs masked)
            pending_irq_mask: None,
            irq_inhibit_next: false,
            allow_irq_once: false,
            branch_taken_defer_irq: false,
            prev_nmi_line: false,
            nmi_pending: false,
            prev_nmi_pending: false,
            index: 0,
            base: 0,
            effective_addr: 0,
            oam_dma: None,
        }
    }

    /// Perform a full CPU reset sequence, as the NES hardware does on power-up.
    ///
    /// The CPU reads two bytes from memory addresses `$FFFC` (low) and `$FFFD` (high)
    /// to determine the starting program counter (reset vector).
    ///
    /// It also clears internal state used by instruction execution.
    pub(crate) fn reset(&mut self, bus: &mut impl Bus) {
        // CpuPtr.store(self as *mut _, std::sync::atomic::Ordering::Release);
        // Read the reset vector from memory ($FFFC-$FFFD)
        let lo = bus.peek(RESET_VECTOR_LO);
        let hi = bus.peek(RESET_VECTOR_HI);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        // Reset other state
        self.s = 0xFD; // Stack pointer is initialized to $FD
        self.p = Status::INTERRUPT;
        self.opcode_in_flight = None;
        self.irq_masked = self.p.i();
        self.pending_irq_mask = None;
        self.irq_inhibit_next = false;
        self.allow_irq_once = false;
        self.branch_taken_defer_irq = false;
        self.prev_nmi_line = false;
        self.nmi_pending = false;
        self.prev_nmi_pending = false;
        self.index = 0;
        self.effective_addr = 0;
        self.oam_dma = None;
        for _ in 0..8 {
            bus.internal_cycle();
        }
    }

    pub(crate) fn clock(&mut self, bus: &mut dyn Bus) {
        // Propagate the current pending flag to `prev_nmi_pending` so that
        // NMI is effectively delayed by one instruction boundary, matching
        // the NES/Mesen behaviour where the interrupt lines are sampled on
        // the "second-to-last" cycle.
        self.prev_nmi_pending = self.nmi_pending;

        // Sample the NMI line every CPU cycle and set `nmi_pending` on
        // a rising edge. The pending flag is only consumed when an NMI
        // is actually serviced.
        let nmi_line = bus.nmi_line();
        if nmi_line && !self.prev_nmi_line {
            self.nmi_pending = true;
        }
        self.prev_nmi_line = nmi_line;

        // Deferred I-flag updates take effect at the next instruction boundary.
        if self.opcode_in_flight.is_none() {
            self.apply_pending_irq_mask();
        }

        if self.handle_oam_dma(bus) {
            return;
        }

        match self.opcode_in_flight {
            // Instruction in flight: optionally sample interrupts mid-instruction for
            // specific opcodes that can be interrupted in the middle of their sequence.
            Some(opcode) => {
                // Special-case BRK: allow NMI/IRQ to be serviced after the dummy read
                // micro-op (index 1) so that NMI can "interrupt" BRK and still push
                // a status byte with the B flag set.
                if opcode == 0x00 && self.index == 1 && self.sample_interrupts(bus) {
                    return;
                }

                let instr = &LOOKUP_TABLE[opcode as usize];
                let micro_op = &instr[self.index()];
                self.exec(bus, instr, micro_op);
                self.post_exec(instr);
                if self.index() >= instr.len() {
                    self.clear();
                }
            }
            // No instruction in flight: first service any pending interrupts, then fetch.
            None => {
                if self.sample_interrupts(bus) {
                    return;
                }
                let opcode = self.fetch_opcode(bus);
                self.opcode_in_flight = Some(opcode);
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.pre_exec(instr);
            }
        }
        // self.cycles = self.cycles.wrapping_add(1);
    }

    #[cfg(test)]
    pub(crate) fn test_clock(&mut self, bus: &mut dyn Bus, instr: &Instruction) -> usize {
        self.opcode_in_flight = Some(instr.opcode());
        self.incr_pc(); // Fetch opcode
        let mut cycles = 1; // Fetch opcode has 1 cycle
        self.pre_exec(instr);
        while self.index() < instr.len() {
            let op = &instr[self.index()];
            let _span = tracing::span!(
                tracing::Level::TRACE,
                "instruction_exec",
                op = ?op,
                index = self.index()
            );
            let _enter = _span.enter();
            let before = *self;
            self.exec(bus, instr, op);
            tracing::event!(
                tracing::Level::TRACE,
                before_cpu = ?before,
                after_cpu = ?self,
                "Instruction executed"
            );
            self.post_exec(instr);
            cycles += 1;
        }
        cycles
    }

    #[inline]
    pub(crate) fn fetch_opcode(&mut self, bus: &mut dyn Bus) -> u8 {
        let opcode = bus.mem_read(self.pc);
        self.incr_pc();
        // Starting a new instruction boundary clears any one-instruction IRQ suppression.
        self.irq_inhibit_next = false;
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
                self.index = (instr.len() - 1) as u8;
            }

            // For all other addressing modes (Absolute, Zero Page, etc.),
            // the effective_addr is calculated during the subsequent micro-ops.
            _ => {}
        }
    }

    #[inline]
    pub(crate) fn exec(&mut self, bus: &mut dyn Bus, instr: &Instruction, micro_op: &MicroOp) {
        match instr.mnemonic {
            // JSR, RTI, and RTS have complex, non-standard instruction cycles (micro-ops),
            // especially during stack manipulation. Their addressing phase cycles are often
            // dedicated to setup and are distinct from standard addressing modes.
            Mnemonic::JSR | Mnemonic::RTI | Mnemonic::RTS if self.index == 0 => {
                // Skip the cycles normally reserved for general addressing mode processing.
                // These instructions have their own custom micro-ops defined immediately
                // following the opcode fetch cycle (index 0).
                self.index += instr.addr_len() as u8;

                // Execute the *first* of the custom, non-addressing micro-ops.
                instr[self.index()].exec(self, bus);
            }
            _ => {
                // For all other instructions, or for the remaining cycles of JSR/RTI/RTS,
                // execute the micro-op corresponding to the current cycle index.
                micro_op.exec(self, bus);
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
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.index() == instr.addr_len() {
                    // JMP is special: it updates the PC right after address calculation,
                    // skipping the final execution phase cycle used by most other instructions.
                    self.pc = self.effective_addr;
                }
            }

            Addressing::Indirect => {
                // JMP Indirect (5 total cycles, 4 addressing cycles after fetch):
                // Addressing micro-ops run at index 0, 1, 2, 3.
                // When index() == 4 (addr_len), the address calculation is finished.
                if matches!(instr.mnemonic, Mnemonic::JMP) && self.index() == instr.addr_len() {
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
        self.branch_taken_defer_irq = false;
    }

    /// Accounts for a CPU bus cycle consumed externally (e.g., DMC DMA) without
    /// advancing any instruction micro-ops. This keeps cycle parity and DMA
    /// alignment consistent with hardware timing.
    pub(crate) fn account_dma_cycle(&mut self) {
        if let Some(dma) = self.oam_dma.as_mut() {
            dma.stall_cycle();
        }
        // self.cycles = self.cycles.wrapping_add(1);
    }

    fn apply_pending_irq_mask(&mut self) {
        if let Some(new_i) = self.pending_irq_mask.take() {
            self.irq_masked = new_i;
        }
    }

    fn handle_oam_dma(&mut self, bus: &mut dyn Bus) -> bool {
        if let Some(dma) = self.oam_dma.as_mut() {
            if dma.step(bus) {
                self.oam_dma = None;
            }
            return true;
        }

        if self.opcode_in_flight.is_none()
            && let Some(page) = bus.take_oam_dma_request()
        {
            let start_on_odd_cycle = (bus.cycles() & 1) == 1;
            let mut dma = OamDma::new(page, start_on_odd_cycle);
            let done = dma.step(bus);
            self.oam_dma = if done { None } else { Some(dma) };
            return true;
        }

        false
    }

    fn push_status(&mut self, bus: &mut dyn Bus, set_break: bool) {
        let mut status = self.p;
        status.set(Status::UNUSED, true);
        status.set(Status::BREAK, set_break);
        self.push(bus, status.bits());
    }

    /// Queue an update to the I flag that will take effect for IRQ gating
    /// at the next instruction boundary.
    pub(crate) fn queue_i_update(&mut self, new_i: bool) {
        self.p.set_i(new_i);
        self.pending_irq_mask = Some(new_i);
    }

    /// Immediately updates the I flag and effective IRQ mask. Used when
    /// entering an interrupt so that nested IRQs are masked.
    pub(crate) fn set_i_immediate(&mut self, new_i: bool) {
        self.p.set_i(new_i);
        self.irq_masked = new_i;
        self.pending_irq_mask = None;
    }

    fn service_irq(&mut self, bus: &mut dyn Bus) -> bool {
        // When the effective I flag is set and no override is armed, or when
        // IRQs are explicitly suppressed for this instruction boundary, mask
        // maskable IRQs.
        if (self.irq_masked && !self.allow_irq_once) || self.irq_inhibit_next || !bus.irq_pending()
        {
            return false;
        }
        self.perform_interrupt(bus, cpu_mem::IRQ_VECTOR_LO, cpu_mem::IRQ_VECTOR_HI, false);
        bus.clear_irq();
        self.allow_irq_once = false;
        true
    }

    fn service_nmi(&mut self, bus: &mut dyn Bus) -> bool {
        // Only service NMI when the pending flag from the previous cycle is set,
        // which matches the Mesen/NES behaviour of delaying NMI by one cycle
        // (effectively one instruction boundary).
        if !self.prev_nmi_pending {
            return false;
        }
        // Clear the current pending flag so we don't immediately retrigger
        // until a new rising edge is observed.
        self.nmi_pending = false;

        // If an NMI overlaps a BRK instruction, hardware behaviour is that the
        // stacked status byte has the B flag set. Approximate this by using
        // the BRK-style push when BRK is the instruction currently in flight.
        let set_break = matches!(self.opcode_in_flight, Some(0x00));
        self.perform_interrupt(
            bus,
            cpu_mem::NMI_VECTOR_LO,
            cpu_mem::NMI_VECTOR_HI,
            set_break,
        );
        true
    }

    fn perform_interrupt(
        &mut self,
        bus: &mut dyn Bus,
        vector_lo: u16,
        vector_hi: u16,
        set_break: bool,
    ) {
        // Dummy read
        bus.mem_read(self.pc);
        bus.mem_read(self.pc);

        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = self.pc as u8;

        self.push(bus, pc_hi);
        self.push(bus, pc_lo);
        self.push_status(bus, set_break);

        // Mask further IRQs immediately upon entering the handler.
        self.set_i_immediate(true);

        let lo = bus.mem_read(vector_lo);
        let hi = bus.mem_read(vector_hi);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        self.opcode_in_flight = None;
        self.index = 0;
    }

    /// Helper that checks for pending NMI/IRQ and services them if allowed.
    fn sample_interrupts(&mut self, bus: &mut dyn Bus) -> bool {
        if self.service_nmi(bus) {
            return true;
        }
        if self.service_irq(bus) {
            return true;
        }
        false
    }

    pub(crate) fn test_branch(&mut self, taken: bool) {
        if !taken {
            self.index += 2; // Skip add branch offset and cross page
        } else {
            // Mark that this branch was taken. Whether we suppress IRQ depends
            // on page crossing (handled later in check_cross_page).
            self.branch_taken_defer_irq = true;
        }
    }

    #[inline]
    pub(crate) fn index(&self) -> usize {
        self.index as usize
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
            if self.branch_taken_defer_irq {
                self.irq_inhibit_next = true;
            }
            self.index += 1;
        }
    }

    pub(crate) fn push(&mut self, bus: &mut dyn Bus, data: u8) {
        bus.mem_write(self.stack_addr(), data);
        self.s = self.s.wrapping_sub(1);
    }

    pub(crate) fn pull(&mut self, bus: &mut dyn Bus) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.mem_read(self.stack_addr())
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

    /// Overwrites CPU registers from a snapshot (resets in-flight instruction).
    pub(crate) fn load_snapshot(&mut self, snapshot: CpuSnapshot) {
        self.pc = snapshot.pc;
        self.a = snapshot.a;
        self.x = snapshot.x;
        self.y = snapshot.y;
        self.s = snapshot.s;
        self.p = Status::from_bits_truncate(snapshot.p);
        self.irq_masked = self.p.i();
        self.pending_irq_mask = None;
        self.irq_inhibit_next = false;
        self.allow_irq_once = false;
        self.branch_taken_defer_irq = false;
        self.prev_nmi_line = false;
        self.nmi_pending = false;
        self.prev_nmi_pending = false;
        self.index = 0;
        self.opcode_in_flight = None;
        self.base = 0;
        self.effective_addr = 0;
        self.oam_dma = None;
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
