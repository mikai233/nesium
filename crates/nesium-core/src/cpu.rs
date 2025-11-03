use std::fmt::{Debug, Display};

use crate::bus::{Bus, STACK_ADDR};
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

    pub(crate) fn clock(&mut self, bus: &mut dyn Bus) {
        match self.opcode {
            Some(opcode) => {
                let instr = &LOOKUP_TABLE[opcode as usize];
                let micro_op = &instr[self.index()];
                micro_op.exec(self, bus);
                self.index += 1;
                self.prepare_zp_addr(instr);
                if self.index() > instr.len() {
                    self.clear();
                }
            }
            None => {
                let opcode = self.fetch_opcode(bus);
                self.opcode = Some(opcode);
                let instr = &LOOKUP_TABLE[opcode as usize];
                self.prepare_imm_addr(instr);
                self.prepare_accumulator(instr);
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_clock(&mut self, bus: &mut dyn Bus, instr: &Instruction) -> usize {
        self.opcode = Some(instr.opcode());
        self.incr_pc(); // Fetch opcode
        let mut cycles = 1; // Fetch opcode has 1 cycle
        self.prepare_imm_addr(instr);
        self.prepare_accumulator(instr);
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
            op.exec(self, bus);
            tracing::event!(
                tracing::Level::TRACE,
                before_cpu = ?before,
                after_cpu = ?self,
                "Instruction executed"
            );
            self.index += 1;
            self.prepare_zp_addr(instr);
            cycles += 1;
        }
        cycles
    }

    #[inline]
    pub(crate) fn fetch_opcode(&mut self, bus: &mut dyn Bus) -> u8 {
        let opcode = bus.read(self.pc);
        self.incr_pc();
        opcode
    }

    #[inline]
    pub(crate) fn prepare_accumulator(&mut self, instr: &Instruction) {
        if matches!(instr.addressing, Addressing::Accumulator) {
            self.index = (instr.len() - 1) as u8;
        }
    }

    #[inline]
    pub(crate) fn prepare_imm_addr(&mut self, instr: &Instruction) {
        if matches!(instr.addressing, Addressing::Immediate) {
            self.effective_addr = self.pc as u16;
            self.incr_pc();
        }
    }

    #[inline]
    pub(crate) fn prepare_zp_addr(&mut self, instr: &Instruction) {
        if self.index == 1 && matches!(instr.addressing, Addressing::ZeroPage) {
            self.effective_addr = self.zp_addr as u16;
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

    pub(crate) fn push(&mut self, bus: &mut dyn Bus, data: u8) {
        bus.write(STACK_ADDR | self.s as u16, data);
        self.s = self.s.wrapping_sub(1);
    }

    pub(crate) fn pull(&mut self, bus: &mut dyn Bus) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.read(STACK_ADDR | self.s as u16)
    }
}

impl Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{:?} PC:{:04X} O:{:02X?} I:{} Z:{:02X} B:{:02X} E:{:04X}",
            self.a,
            self.x,
            self.y,
            self.s,
            self.p,
            self.pc,
            self.opcode,
            self.index,
            self.zp_addr,
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
            None => {
                format!("  ")
            }
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
            "║     zp_addr: {:02X}      │    base: {:02X}      ║",
            self.zp_addr, self.base
        )?;
        writeln!(
            f,
            "║ effective_addr: {:04X} │    index: {:02X}     ║",
            self.effective_addr, self.index
        )?;
        writeln!(f, "╚═════════════════════════════════════════╝")?;

        Ok(())
    }
}
