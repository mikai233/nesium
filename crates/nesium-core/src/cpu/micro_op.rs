use crate::{bus::Bus, cpu::CPU};

type OpFn = fn(&mut CPU, &mut Bus);

#[derive(Debug, Clone, Copy)]
pub(crate) struct MicroOp2 {
    pub(crate) name: &'static str,
    pub(crate) op: OpFn,
}

impl MicroOp2 {
    pub(crate) fn new(name: &'static str, op: OpFn) -> Self {
        Self { name, op }
    }
}

impl MicroOp2 {
    pub(crate) fn exec(&self, cpu: &mut CPU, bus: &mut Bus) {
        (self.op)(cpu, bus);
    }

    pub(crate) fn fetch_opcode() -> Self {
        Self {
            name: "fetch_opcode",
            op: |cpu, _| {
                cpu.incr_pc();
            },
        }
    }
}

/// Represents one cycle-level micro operation used during address resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MicroOp {
    /// Fetch opcode byte.
    FetchOpcode,

    /// Fetch low byte of address (operand low).
    FetchAddrLo,

    /// Fetch high byte of address (operand high).
    FetchAddrHi,

    /// Fetch zero-page address byte (operand).
    FetchZeroPageAddr,

    /// Read from zero-page memory.
    ReadZeroPage,

    /// Read from zero-page + X.
    ReadZeroPageXBase,
    /// Read from zero-page + Y.
    ReadZeroPageYBase,

    /// Fetch pointer low byte (for indirect addressing).
    ReadIndirectLo,
    /// Fetch pointer high byte.
    ReadIndirectHi,

    /// Read from absolute + X base (may cross page).
    ReadAbsXBase,
    /// Read from absolute + Y base (may cross page).
    ReadAbsYBase,

    /// Dummy read if page boundary crossed (e.g. ABS,X or ABS,Y).
    DummyReadIfCrossed,

    /// Read the effective address (final operand fetch).
    ReadEffective,

    /// Read the immediate operand (fetch the next byte after the opcode).
    ReadImmediate,

    /// Read effective address for branch (PC-relative).
    FetchRelativeOffset,

    /// Dummy read (used for implied/accumulator instructions).
    DummyRead,

    /// Internal no-op (used to pad cycle count for consistency).
    Idle,

    //
    Instruction(fn(&mut CPU, &mut Bus)),
}

impl MicroOp {
    pub(crate) fn exec(&self, cpu: &mut CPU, bus: &mut Bus) {
        match self {
            // -----------------------------------------------------------------
            // Fetch instruction opcode
            // -----------------------------------------------------------------
            MicroOp::FetchOpcode => {
                cpu.incr_pc();
            }

            // -----------------------------------------------------------------
            // Fetch address / operand bytes
            // -----------------------------------------------------------------
            MicroOp::FetchAddrLo => {
                let lo = bus.read(cpu.pc);
                cpu.incr_pc();
                cpu.effective_addr = lo as u16;
            }

            MicroOp::FetchAddrHi => {
                let hi = bus.read(cpu.pc);
                cpu.incr_pc();
                let base = cpu.effective_addr | ((hi as u16) << 8);
                cpu.effective_addr = base;
            }

            MicroOp::FetchZeroPageAddr => {
                let addr = bus.read(cpu.pc);
                cpu.incr_pc();
                cpu.effective_addr = addr as u16;
            }

            MicroOp::FetchRelativeOffset => {
                let offset = bus.read(cpu.pc);
                cpu.incr_pc();
                cpu.data = offset;
            }

            // -----------------------------------------------------------------
            // Zero Page / Indexed
            // -----------------------------------------------------------------
            MicroOp::ReadZeroPage => {
                let addr = cpu.effective_addr & 0xFF;
                cpu.data = bus.read(addr);
            }

            MicroOp::ReadZeroPageXBase => {
                let base = cpu.effective_addr & 0xFF;
                let addr = base.wrapping_add(cpu.x as u16) & 0xFF;
                cpu.effective_addr = addr;
                let _ = bus.read(addr);
            }

            MicroOp::ReadZeroPageYBase => {
                let base = cpu.effective_addr & 0xFF;
                let addr = base.wrapping_add(cpu.y as u16) & 0xFF;
                cpu.effective_addr = addr;
                let _ = bus.read(addr);
            }

            // -----------------------------------------------------------------
            // Absolute + X / Y (check page cross)
            // -----------------------------------------------------------------
            MicroOp::ReadAbsXBase => {
                let base = cpu.effective_addr;
                let addr = base.wrapping_add(cpu.x as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.data = bus.read(cpu.effective_addr);
            }

            MicroOp::ReadAbsYBase => {
                let base = cpu.effective_addr;
                let addr = base.wrapping_add(cpu.y as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.data = bus.read(cpu.effective_addr);
            }

            MicroOp::DummyReadIfCrossed => {
                let _ = bus.read(cpu.effective_addr);
            }

            // -----------------------------------------------------------------
            // Indirect addressing
            // -----------------------------------------------------------------
            MicroOp::ReadIndirectLo => {
                let zp_addr = cpu.data as u16;
                let lo = bus.read(zp_addr);
                cpu.effective_addr = lo as u16;
            }

            MicroOp::ReadIndirectHi => {
                let zp_addr = cpu.effective_addr & 0xFF;
                let hi = bus.read(zp_addr.wrapping_add(1) & 0xFF);
                let base = cpu.effective_addr | ((hi as u16) << 8);
                cpu.effective_addr = base;
            }

            // -----------------------------------------------------------------
            // Read final effective value
            // -----------------------------------------------------------------
            MicroOp::ReadEffective => {
                let addr = cpu.effective_addr;
                cpu.data = bus.read(addr);
            }

            MicroOp::ReadImmediate => {
                let byte = bus.read(cpu.pc);
                cpu.pc = cpu.pc.wrapping_add(1);
                cpu.data = byte;
            }

            // -----------------------------------------------------------------
            // Dummy read / padding
            // -----------------------------------------------------------------
            MicroOp::DummyRead => {
                let _ = bus.read(cpu.pc);
            }

            MicroOp::Idle => {
                // no-op
            }
            MicroOp::Instruction(micro_op) => {
                micro_op(cpu, bus);
            }
        }
    }
}
