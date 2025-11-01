use std::fmt::Display;

use crate::bus::{Bus, BusImpl};
use crate::cpu::addressing::Addressing;
use crate::cpu::cycle::{CYCLE_TABLE, Cycle};
use crate::cpu::instruction::Instruction;
use crate::cpu::lookup::LOOKUP_TABLE;
use crate::cpu::status::Status;
mod phase;
mod status;

mod addressing;
mod cycle;
mod instruction;
mod lookup;
mod micro_op;
mod mnemonic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Cpu {
    // Registers
    a: u8,     //Accumulator
    x: u8,     //X Index Register
    y: u8,     //Y Index Register
    s: u8,     //Stack Pointer
    p: Status, //Processor Status
    pc: u16,   //Program Counter

    opcode: Option<u8>,
    index: u8,
    zp_addr: u8,
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
            index: 0,
            zp_addr: 0,
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
        let lo = bus.read(0xFFFC);
        let hi = bus.read(0xFFFD);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        // Reset other state
        self.s = 0xFD; // Stack pointer is initialized to $FD
        self.p = Status::from_bits_truncate(0x34); // IRQ disabled
        self.opcode = None;
        self.index = 0;
        self.effective_addr = 0;
    }

    pub(crate) fn clock(&mut self, bus: &mut BusImpl) {
        match self.opcode {
            Some(opcode) => {
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.prepare_imm_addr(instr);
                let micro_op = &instr[self.index()];
                micro_op.exec(self, bus);
                self.index += 1;
                self.prepare_zp_addr(instr);
                if self.index() > instr.len() {
                    self.clear();
                }
            }
            None => {
                self.opcode = Some(self.fetch_opcode(bus));
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_clock(&mut self, bus: &mut BusImpl, instr: &Instruction) -> usize {
        self.incr_pc(); // Fetch opcode
        let mut cycles = 1; // Fetch opcode has 1 cycle
        self.prepare_imm_addr(instr);
        while self.index() < instr.len() {
            instr[self.index()].exec(self, bus);
            self.index += 1;
            self.prepare_zp_addr(instr);
            cycles += 1;
        }
        cycles
    }

    pub(crate) fn fetch_opcode(&mut self, bus: &mut BusImpl) -> u8 {
        let opcode = bus.read(self.pc);
        self.incr_pc();
        opcode
    }

    pub(crate) fn prepare_imm_addr(&mut self, instr: &Instruction) {
        if matches!(instr.addressing, Addressing::Immediate) {
            self.effective_addr = self.pc as u16;
            self.incr_pc();
        }
    }

    pub(crate) fn prepare_zp_addr(&mut self, instr: &Instruction) {
        if self.index == 1 && matches!(instr.addressing, Addressing::ZeroPage) {
            self.effective_addr = self.zp_addr as u16;
        }
    }

    pub(crate) fn incr_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub(crate) fn clear(&mut self) {
        self.index = 0;
        self.opcode = None;
        self.base = 0;
        self.zp_addr = 0;
        self.effective_addr = 0;
    }

    pub(crate) fn test_branch(&mut self, taken: bool) {
        if taken {
            let old_pc = self.pc;
            let new_pc = old_pc.wrapping_add(self.base as u16);
            self.check_cross_page(old_pc, new_pc);
            self.pc = new_pc;
        } else {
            self.index += 1;
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
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[a:0x{:02x},x:0x{:02x},y:0x{:02x},s:0x{:02x},pc:0x{:04x}]",
            self.a, self.x, self.y, self.s, self.pc
        )
    }
}
