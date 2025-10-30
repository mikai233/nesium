use crate::bus::{Bus, BusImpl};
use crate::cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::lookup::LOOKUP_TABLE;
use crate::cpu::status::Status;
mod phase;
mod status;

mod addressing;
mod instruction;
mod lookup;
mod micro_op;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cpu {
    // Registers
    a: u8,     //Accumulator
    x: u8,     //X Index Register
    y: u8,     //Y Index Register
    s: u8,     //Stack Pointer
    p: Status, //Processor Status
    pc: u16,   //Program Counter

    instruction: Option<&'static Instruction>,
    index: usize,
    tmp: u8,
    zp_addr: u8,
    base_lo: u8,
    effective_addr: u16,
    check_cross_page: bool,
    crossed_page: bool,
}

impl Cpu {
    /// Create a new CPU instance with default values.
    /// Does not automatically fetch the reset vector — call `reset()` for that.
    pub fn new() -> Self {
        Self {
            a: 0x00,                             // Accumulator
            x: 0x00,                             // X register
            y: 0x00,                             // Y register
            s: 0xFD,                             // Stack pointer after reset
            p: Status::from_bits_truncate(0x34), // IRQ disabled, bit 5 always set
            pc: 0x0000,                          // Will be set by reset vector
            instruction: None,
            index: 0,
            tmp: 0,
            zp_addr: 0,
            base_lo: 0,
            effective_addr: 0,
            check_cross_page: false,
            crossed_page: false,
        }
    }

    /// Perform a full CPU reset sequence, as the NES hardware does on power-up.
    ///
    /// The CPU reads two bytes from memory addresses `$FFFC` (low) and `$FFFD` (high)
    /// to determine the starting program counter (reset vector).
    ///
    /// It also clears internal state used by instruction execution.
    pub fn reset(&mut self, bus: &mut impl Bus) {
        // Read the reset vector from memory ($FFFC-$FFFD)
        let lo = bus.read(0xFFFC);
        let hi = bus.read(0xFFFD);
        self.pc = ((hi as u16) << 8) | (lo as u16);

        // Reset other state
        self.s = 0xFD; // Stack pointer is initialized to $FD
        self.p = Status::from_bits_truncate(0x34); // IRQ disabled
        self.instruction = None;
        self.index = 0;
        self.tmp = 0;
        self.effective_addr = 0;
        self.check_cross_page = false;
        self.crossed_page = false;
    }

    pub fn clock(&mut self, bus: &mut BusImpl) {
        let instruction = *self.instruction.get_or_insert_with(|| {
            let opcode = bus.read(self.pc);
            &LOOKUP_TABLE[opcode as usize]
        });
        let micro_op = &instruction.micro_ops[self.index];
        micro_op.exec(self, bus);
        self.index += 1;
        if self.check_cross_page && !self.crossed_page {
            self.check_cross_page = false;
            self.index += 1; // skip next cross page op
        }
        if self.index > instruction.micro_ops.len() {
            self.clear();
        }
    }

    #[cfg(test)]
    pub(crate) fn test_clock(&mut self, bus: &mut BusImpl, instruction: &Instruction) {
        let micro_ops = instruction.micro_ops;
        while self.index < micro_ops.len() {
            micro_ops[self.index].exec(self, bus);
            self.index += 1;
            if self.check_cross_page && !self.crossed_page {
                self.check_cross_page = false;
                self.index += 1; // skip next cross page op
            }
        }
    }

    pub fn fetch(&mut self, bus: &mut BusImpl) -> &'static Instruction {
        let opcode = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        &LOOKUP_TABLE[opcode as usize]
    }

    pub(crate) fn incr_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub(crate) fn clear(&mut self) {
        self.index = 0;
        self.instruction = None;
        self.effective_addr = 0;
        self.crossed_page = false;
    }
}
