use crate::bus::Bus;
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
    effective_addr: u16,
    data: u8,
    crossed_page: bool,
}

impl Cpu {
    pub fn clock<B>(&mut self, bus: &mut B)
    where
        B: Bus,
    {
        let instruction = *self.instruction.get_or_insert_with(|| {
            let opcode = bus.read(self.pc);
            &LOOKUP_TABLE[opcode as usize]
        });
        let micro_op = &instruction.micro_ops[self.index];
        micro_op.exec(self, bus);
        self.index += 1;
        if micro_op.check_cross_page() && !self.crossed_page {
            self.index += 1; // Ignore next cross page op
        }
        if self.index > instruction.micro_ops.len() {
            self.clear();
        }
    }

    pub fn fetch<B>(&mut self, bus: &mut B) -> &'static Instruction
    where
        B: Bus,
    {
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
        self.data = 0;
        self.crossed_page = false;
    }
}
