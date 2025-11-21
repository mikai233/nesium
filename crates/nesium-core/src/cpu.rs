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
mod status;

mod addressing;
mod cycle;
mod instruction;
mod lookup;
mod micro_op;
mod mnemonic;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Cpu {
    // Registers
    a: u8,     //Accumulator
    x: u8,     //X Index Register
    y: u8,     //Y Index Register
    s: u8,     //Stack Pointer
    p: Status, //Processor Status
    pc: u16,   //Program Counter

    opcode: Option<u8>,
    /// Suppress servicing maskable IRQs for the next instruction boundary.
    /// Used to model the 6502 behaviour where a pending IRQ is not taken
    /// until one instruction after CLI clears the I flag.
    irq_suppressed: bool,
    /// Allow a single IRQ even though the I flag is set.
    /// This is used to approximate the behaviour of CLI/SEI and related
    /// sequences where a pending IRQ is taken "just after" SEI/PLP.
    force_irq_once: bool,
    index: u8,
    base: u8,
    effective_addr: u16,
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
            opcode: None,
            irq_suppressed: false,
            force_irq_once: false,
            index: 0,
            base: 0,
            effective_addr: 0,
        }
    }

    /// Perform a full CPU reset sequence, as the NES hardware does on power-up.
    ///
    /// The CPU reads two bytes from memory addresses `$FFFC` (low) and `$FFFD` (high)
    /// to determine the starting program counter (reset vector).
    ///
    /// It also clears internal state used by instruction execution.
    pub(crate) fn reset(&mut self, bus: &mut impl Bus) {
        // Read the reset vector from memory ($FFFC-$FFFD)
        let lo = bus.read(RESET_VECTOR_LO);
        let hi = bus.read(RESET_VECTOR_HI);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        // Reset other state
        self.s = 0xFD; // Stack pointer is initialized to $FD
        self.p = Status::from_bits_truncate(0x34); // IRQ disabled
        self.opcode = None;
        self.irq_suppressed = false;
        self.force_irq_once = false;
        self.index = 0;
        self.effective_addr = 0;
    }

    pub(crate) fn clock(&mut self, bus: &mut dyn Bus) {
        if self.opcode.is_none() {
            if self.service_nmi(bus) {
                return;
            }
            if self.service_irq(bus) {
                return;
            }
        }

        match self.opcode {
            Some(opcode) => {
                let instr = &LOOKUP_TABLE[opcode as usize];
                let micro_op = &instr[self.index()];
                self.exec(bus, instr, micro_op);
                self.post_exec(instr);
                if self.index() >= instr.len() {
                    self.clear();
                }
            }
            None => {
                let opcode = self.fetch_opcode(bus);
                self.opcode = Some(opcode);
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.pre_exec(instr);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_clock(&mut self, bus: &mut dyn Bus, instr: &Instruction) -> usize {
        self.opcode = Some(instr.opcode());
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
        let opcode = bus.read(self.pc);
        self.incr_pc();
        // Starting a new instruction boundary clears any one-instruction IRQ suppression.
        self.irq_suppressed = false;
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
                self.incr_pc();
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
        self.opcode = None;
        self.base = 0;
        self.effective_addr = 0;
    }

    fn push_status(&mut self, bus: &mut dyn Bus, set_break: bool) {
        let mut status = self.p;
        status.set(Status::UNUSED, true);
        status.set(Status::BREAK, set_break);
        self.push(bus, status.bits());
    }

    fn service_irq(&mut self, bus: &mut dyn Bus) -> bool {
        if (self.p.i() && !self.force_irq_once) || self.irq_suppressed || !bus.irq_pending() {
            return false;
        }
        self.perform_interrupt(bus, cpu_mem::IRQ_VECTOR_LO, cpu_mem::IRQ_VECTOR_HI, false);
        bus.clear_irq();
        self.force_irq_once = false;
        true
    }

    fn service_nmi(&mut self, bus: &mut dyn Bus) -> bool {
        if !bus.poll_nmi() {
            return false;
        }
        self.perform_interrupt(bus, cpu_mem::NMI_VECTOR_LO, cpu_mem::NMI_VECTOR_HI, false);
        true
    }

    fn perform_interrupt(
        &mut self,
        bus: &mut dyn Bus,
        vector_lo: u16,
        vector_hi: u16,
        set_break: bool,
    ) {
        let pc_hi = (self.pc >> 8) as u8;
        let pc_lo = self.pc as u8;

        self.push(bus, pc_hi);
        self.push(bus, pc_lo);
        self.push_status(bus, set_break);

        self.p.set_i(true);

        let lo = bus.read(vector_lo);
        let hi = bus.read(vector_hi);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        self.opcode = None;
        self.index = 0;
    }

    pub(crate) fn test_branch(&mut self, taken: bool) {
        if !taken {
            self.index += 2; // Skip add branch offset and cross page
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
        let opcode = self.opcode.expect("opcode not set");
        let instr = &LOOKUP_TABLE[opcode as usize];
        if Self::always_cross_page(opcode, instr) {
            return;
        }
        let crossed_page = (base & 0xFF00) != (addr & 0xFF00);
        if !crossed_page {
            self.index += 1;
        }
    }

    pub(crate) fn push(&mut self, bus: &mut dyn Bus, data: u8) {
        bus.write(STACK_ADDR | self.s as u16, data);
        self.s = self.s.wrapping_sub(1);
    }

    pub(crate) fn pull(&mut self, bus: &mut dyn Bus) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.read(STACK_ADDR | self.s as u16)
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
        self.index = 0;
        self.opcode = None;
        self.irq_suppressed = false;
        self.force_irq_once = false;
        self.base = 0;
        self.effective_addr = 0;
    }

    /// Returns `true` when an instruction is currently in flight.
    pub(crate) fn opcode_active(&self) -> bool {
        self.opcode.is_some()
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
            self.opcode,
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
        let opcode = match self.opcode {
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
