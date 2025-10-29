use crate::{bus::Bus, cpu::Cpu};
pub mod load;

pub trait MicroFn: Sized {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus;
}

/// Represents one cycle-level micro operation used during address resolution.
///
/// Micro operations break down instruction execution into individual clock cycles,
/// providing cycle-accurate emulation of the 6502 CPU's internal operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MicroOp {
    // ===== Implied Addressing =====
    /// C1: Fetch opcode and decode instruction (no operand bytes)
    ImpliedC1,

    // ===== Accumulator Addressing =====
    /// C1: Fetch opcode and decode instruction (operates on accumulator only)
    AccumulatorC1,

    // ===== Immediate Addressing =====
    /// C1: Fetch opcode and decode instruction
    ImmediateC1,
    /// C2: Read immediate data byte and complete operation
    ImmediateC2,

    // ===== Absolute Addressing =====
    /// C1: Fetch opcode and decode instruction
    AbsoluteC1,
    /// C2: Read low byte of absolute address
    AbsoluteC2,
    /// C3: Read high byte of absolute address and complete operation
    AbsoluteC3,

    // ===== Absolute,X Addressing =====
    /// C1: Fetch opcode and decode instruction
    AbsoluteXC1,
    /// C2: Read low byte of absolute address
    AbsoluteXC2,
    /// C3: Read high byte of absolute address, add X to low byte (page boundary check)
    AbsoluteXC3,
    /// C4: Read from effective address (page cross penalty cycle)
    AbsoluteXC4,

    // ===== Absolute,Y Addressing =====
    /// C1: Fetch opcode and decode instruction
    AbsoluteYC1,
    /// C2: Read low byte of absolute address
    AbsoluteYC2,
    /// C3: Read high byte of absolute address, add Y to low byte (page boundary check)
    AbsoluteYC3,
    /// C4: Read from effective address (no page cross)
    AbsoluteYC4,
    /// C5: Read from effective address (page cross penalty cycle)
    AbsoluteYC5,

    // ===== Indirect Addressing (JMP only) =====
    /// C1: Fetch opcode and decode instruction
    IndirectC1,
    /// C2: Read low byte of pointer address
    IndirectC2,
    /// C3: Read high byte of pointer address
    IndirectC3,
    /// C4: Read low byte of target address from pointer
    IndirectC4,
    /// C5: Read high byte of target address from pointer (with page boundary bug)
    IndirectC5,

    // ===== Zero Page Addressing =====
    /// C1: Fetch opcode and decode instruction
    ZeroPageC1,
    /// C2: Read zero page address and complete operation
    ZeroPageC2,

    // ===== Zero Page,X Addressing =====
    /// C1: Fetch opcode and decode instruction
    ZeroPageXC1,
    /// C2: Read zero page address
    ZeroPageXC2,
    /// C3: Dummy read of original zero page address (before adding X)
    ZeroPageXC3,
    /// C4: Read from effective zero page address (with wrap-around)
    ZeroPageXC4,

    // ===== Zero Page,Y Addressing =====
    /// C1: Fetch opcode and decode instruction
    ZeroPageYC1,
    /// C2: Read zero page address
    ZeroPageYC2,
    /// C3: Dummy read of original zero page address (before adding Y)
    ZeroPageYC3,
    /// C4: Read from effective zero page address (with wrap-around)
    ZeroPageYC4,

    // ===== (Indirect,X) Addressing =====
    /// C1: Fetch opcode and decode instruction
    IndirectXC1,
    /// C2: Read zero page address
    IndirectXC2,
    /// C3: Dummy read of original zero page address (before adding X)
    IndirectXC3,
    /// C4: Read low byte of pointer from (zero_page + X)
    IndirectXC4,
    /// C5: Read high byte of pointer from (zero_page + X + 1) with zero page wrap
    IndirectXC5,
    /// C6: Read data from final effective address
    IndirectXC6,

    // ===== (Indirect),Y Addressing =====
    /// C1: Fetch opcode and decode instruction
    IndirectYC1,
    /// C2: Read zero page address
    IndirectYC2,
    /// C3: Read low byte of pointer from zero page address
    IndirectYC3,
    /// C4: Read high byte of pointer from (zero_page + 1) with zero page wrap
    IndirectYC4,
    /// C5: Read from effective address (no page cross)
    IndirectYC5,
    /// C6: Read from effective address (page cross penalty cycle)
    IndirectYC6,

    // ===== Relative Addressing =====
    /// C1: Fetch opcode and decode instruction
    RelativeC1,
    /// C2: Read relative offset and calculate branch target
    RelativeC2,
    /// C3: Dummy read while checking branch condition and page cross
    RelativeC3,
    /// C4: Read from branch target address (page cross penalty cycle)
    RelativeC4,
}

impl MicroOp {
    pub(crate) fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        match self {
            MicroOp::ImpliedC1 => ImpliedC1.exec(cpu, bus),
            MicroOp::AccumulatorC1 => AccumulatorC1.exec(cpu, bus),
            MicroOp::ImmediateC1 => ImmediateC1.exec(cpu, bus),
            MicroOp::ImmediateC2 => ImmediateC2.exec(cpu, bus),
            MicroOp::AbsoluteC1 => AbsoluteC1.exec(cpu, bus),
            MicroOp::AbsoluteC2 => AbsoluteC2.exec(cpu, bus),
            MicroOp::AbsoluteC3 => AbsoluteC3.exec(cpu, bus),
            MicroOp::AbsoluteXC1 => AbsoluteXC1.exec(cpu, bus),
            MicroOp::AbsoluteXC2 => AbsoluteXC2.exec(cpu, bus),
            MicroOp::AbsoluteXC3 => AbsoluteXC3.exec(cpu, bus),
            MicroOp::AbsoluteXC4 => AbsoluteXC4.exec(cpu, bus),
            MicroOp::AbsoluteYC1 => AbsoluteYC1.exec(cpu, bus),
            MicroOp::AbsoluteYC2 => AbsoluteYC2.exec(cpu, bus),
            MicroOp::AbsoluteYC3 => AbsoluteYC3.exec(cpu, bus),
            MicroOp::AbsoluteYC4 => AbsoluteYC4.exec(cpu, bus),
            MicroOp::AbsoluteYC5 => AbsoluteYC5.exec(cpu, bus),
            MicroOp::IndirectC1 => IndirectC1.exec(cpu, bus),
            MicroOp::IndirectC2 => IndirectC2.exec(cpu, bus),
            MicroOp::IndirectC3 => IndirectC3.exec(cpu, bus),
            MicroOp::IndirectC4 => IndirectC4.exec(cpu, bus),
            MicroOp::IndirectC5 => IndirectC5.exec(cpu, bus),
            MicroOp::ZeroPageC1 => ZeroPageC1.exec(cpu, bus),
            MicroOp::ZeroPageC2 => ZeroPageC2.exec(cpu, bus),
            MicroOp::ZeroPageXC1 => ZeroPageXC1.exec(cpu, bus),
            MicroOp::ZeroPageXC2 => ZeroPageXC2.exec(cpu, bus),
            MicroOp::ZeroPageXC3 => ZeroPageXC3.exec(cpu, bus),
            MicroOp::ZeroPageXC4 => ZeroPageXC4.exec(cpu, bus),
            MicroOp::ZeroPageYC1 => ZeroPageYC1.exec(cpu, bus),
            MicroOp::ZeroPageYC2 => ZeroPageYC2.exec(cpu, bus),
            MicroOp::ZeroPageYC3 => ZeroPageYC3.exec(cpu, bus),
            MicroOp::ZeroPageYC4 => ZeroPageYC4.exec(cpu, bus),
            MicroOp::IndirectXC1 => IndirectXC1.exec(cpu, bus),
            MicroOp::IndirectXC2 => IndirectXC2.exec(cpu, bus),
            MicroOp::IndirectXC3 => IndirectXC3.exec(cpu, bus),
            MicroOp::IndirectXC4 => IndirectXC4.exec(cpu, bus),
            MicroOp::IndirectXC5 => IndirectXC5.exec(cpu, bus),
            MicroOp::IndirectXC6 => IndirectXC6.exec(cpu, bus),
            MicroOp::IndirectYC1 => IndirectYC1.exec(cpu, bus),
            MicroOp::IndirectYC2 => IndirectYC2.exec(cpu, bus),
            MicroOp::IndirectYC3 => IndirectYC3.exec(cpu, bus),
            MicroOp::IndirectYC4 => IndirectYC4.exec(cpu, bus),
            MicroOp::IndirectYC5 => IndirectYC5.exec(cpu, bus),
            MicroOp::IndirectYC6 => IndirectYC6.exec(cpu, bus),
            MicroOp::RelativeC1 => RelativeC1.exec(cpu, bus),
            MicroOp::RelativeC2 => RelativeC2.exec(cpu, bus),
            MicroOp::RelativeC3 => RelativeC3.exec(cpu, bus),
            MicroOp::RelativeC4 => RelativeC4.exec(cpu, bus),
        }
    }

    pub(crate) fn check_cross_page(&self) -> bool {
        match self {
            MicroOp::AbsoluteXC3 => true,
            _ => false,
        }
    }
}

// Implied addressing mode
struct ImpliedC1;

// Accumulator addressing mode
struct AccumulatorC1;

// Immediate addressing mode
struct ImmediateC1;
struct ImmediateC2;

// Absolute addressing mode
struct AbsoluteC1;
struct AbsoluteC2;
struct AbsoluteC3;

// Absolute X-indexed addressing mode
struct AbsoluteXC1;
struct AbsoluteXC2;
struct AbsoluteXC3;
struct AbsoluteXC4;
struct AbsoluteXC5;

// Absolute Y-indexed addressing mode
struct AbsoluteYC1;
struct AbsoluteYC2;
struct AbsoluteYC3;
struct AbsoluteYC4;
struct AbsoluteYC5;

// Indirect addressing mode
struct IndirectC1;
struct IndirectC2;
struct IndirectC3;
struct IndirectC4;
struct IndirectC5;

// Zero page addressing mode
struct ZeroPageC1;
struct ZeroPageC2;

// Zero page X-indexed addressing mode
struct ZeroPageXC1;
struct ZeroPageXC2;
struct ZeroPageXC3;
struct ZeroPageXC4;

// Zero page Y-indexed addressing mode
struct ZeroPageYC1;
struct ZeroPageYC2;
struct ZeroPageYC3;
struct ZeroPageYC4;

// Indirect X-indexed addressing mode
struct IndirectXC1;
struct IndirectXC2;
struct IndirectXC3;
struct IndirectXC4;
struct IndirectXC5;
struct IndirectXC6;

// Indirect Y-indexed addressing mode
struct IndirectYC1;
struct IndirectYC2;
struct IndirectYC3;
struct IndirectYC4;
struct IndirectYC5;
struct IndirectYC6;

// Relative addressing mode
struct RelativeC1;
struct RelativeC2;
struct RelativeC3;
struct RelativeC4;

impl MicroFn for ImpliedC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for AccumulatorC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for ImmediateC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for ImmediateC2 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现立即寻址周期2
    }
}

impl MicroFn for AbsoluteC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for AbsoluteC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = lo;
    }
}

impl MicroFn for AbsoluteC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let hi = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
    }
}

impl MicroFn for AbsoluteXC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for AbsoluteXC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = lo;
    }
}

impl MicroFn for AbsoluteXC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let hi = bus.read(cpu.pc);
        cpu.incr_pc();
        let base = ((hi as u16) << 8) | (cpu.tmp as u16);
        let eff = base.wrapping_add(cpu.x as u16);
        cpu.crossed_page = (base & 0xFF00) != (eff & 0xFF00);
        cpu.effective_addr = eff;
    }
}

impl MicroFn for AbsoluteXC4 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        if cpu.crossed_page {
            // Dummy read if crossed page
            let _ = bus.read(
                (cpu.effective_addr & 0xFF00) | ((cpu.effective_addr.wrapping_sub(0x100)) & 0x00FF),
            );
        }
    }
}

impl MicroFn for AbsoluteXC5 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现绝对X变址周期5
    }
}

impl MicroFn for AbsoluteYC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for AbsoluteYC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = lo;
    }
}

impl MicroFn for AbsoluteYC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let hi = bus.read(cpu.pc);
        cpu.incr_pc();
        let base = ((hi as u16) << 8) | (cpu.tmp as u16);
        let eff = base.wrapping_add(cpu.y as u16);
        cpu.crossed_page = (base & 0xFF00) != (eff & 0xFF00);
        cpu.effective_addr = eff;
    }
}

impl MicroFn for AbsoluteYC4 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        if cpu.crossed_page {
            let _ = bus.read(
                (cpu.effective_addr & 0xFF00) | ((cpu.effective_addr.wrapping_sub(0x100)) & 0x00FF),
            );
        }
    }
}

impl MicroFn for AbsoluteYC5 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现绝对Y变址周期5
    }
}

impl MicroFn for IndirectC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for IndirectC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = lo;
    }
}

impl MicroFn for IndirectC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let hi = bus.read(cpu.pc);
        cpu.incr_pc();
        // Build full 16-bit indirect address: $xxFF
        cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
    }
}

impl MicroFn for IndirectC4 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let ptr = cpu.effective_addr;
        let lo = bus.read(ptr);
        cpu.tmp = lo; // Reuse tmp to hold target low byte
    }
}

impl MicroFn for IndirectC5 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let ptr = cpu.effective_addr;

        // 6502 bug: if low byte of ptr is $FF, high byte is read
        // from $xx00 instead of $xx+1 (no carry)
        let hi_addr = if (ptr & 0x00FF) == 0x00FF {
            ptr & 0xFF00 // Stay on same page (bug)
        } else {
            ptr.wrapping_add(1) // Normal increment
        };

        let hi = bus.read(hi_addr);
        cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
    }
}

impl MicroFn for ZeroPageC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for ZeroPageC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let addr = bus.read(cpu.pc) as u16;
        cpu.incr_pc();
        cpu.effective_addr = addr;
    }
}

impl MicroFn for ZeroPageXC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for ZeroPageXC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = base; // Save base address in tmp
    }
}

impl MicroFn for ZeroPageXC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = cpu.tmp as u16;
        // dummy read
        let _ = bus.read(base);
        let offset = cpu.x as u16;
        cpu.effective_addr = (base + offset) & 0xFF; // Wrap within zero page
    }
}

impl MicroFn for ZeroPageXC4 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现零页X变址周期4
    }
}

impl MicroFn for ZeroPageYC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for ZeroPageYC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = base; // Save base ZP address
    }
}

impl MicroFn for ZeroPageYC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = cpu.tmp as u16;
        // dummy read
        let _ = bus.read(base);
        let offset = cpu.y as u16;
        cpu.effective_addr = (base + offset) & 0xFF; // Wrap within zero page
    }
}

impl MicroFn for ZeroPageYC4 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现零页Y变址周期4
    }
}

impl MicroFn for IndirectXC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for IndirectXC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = base; // tmp = ZP base (e.g., $34)
    }
}

impl MicroFn for IndirectXC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let _ = bus.read(cpu.tmp as u16); // Dummy read (critical!)
        let ptr = cpu.tmp.wrapping_add(cpu.x); // Wrap in ZP
        cpu.tmp = ptr; // tmp = final ZP pointer
    }
}

impl MicroFn for IndirectXC4 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.tmp as u16);
        cpu.effective_addr = lo as u16; // Store low byte
    }
}

impl MicroFn for IndirectXC5 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let ptr_hi = (cpu.tmp as u16).wrapping_add(1);
        let hi = bus.read(ptr_hi);
        cpu.effective_addr = ((hi as u16) << 8) | (cpu.effective_addr & 0xFF);
    }
}

impl MicroFn for IndirectXC6 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现间接X变址周期6
    }
}

impl MicroFn for IndirectYC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for IndirectYC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let zp = bus.read(cpu.pc);
        cpu.incr_pc();
        cpu.tmp = zp; // ZP address (e.g., $12)
    }
}

impl MicroFn for IndirectYC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let lo = bus.read(cpu.tmp as u16);
        cpu.tmp = lo; // Reuse tmp for base_lo
    }
}

impl MicroFn for IndirectYC4 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let zp_hi = (cpu.tmp.wrapping_add(1)) as u16; // ZP+1
        let hi = bus.read(zp_hi);
        let base = ((hi as u16) << 8) | (cpu.tmp as u16);
        cpu.effective_addr = base; // Store base address
    }
}

impl MicroFn for IndirectYC5 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let base = cpu.effective_addr;
        let eff = base.wrapping_add(cpu.y as u16);

        let crossed = (base & 0xFF00) != (eff & 0xFF00);
        cpu.crossed_page = crossed;

        if crossed {
            // 6502 BUG: dummy read from (eff & 0xFF) in SAME page as base
            let dummy_addr = (base & 0xFF00) | (eff & 0x00FF);
            let _ = bus.read(dummy_addr);
        }

        cpu.effective_addr = eff; // Final address
    }
}

impl MicroFn for IndirectYC6 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现间接Y变址周期6
    }
}

impl MicroFn for RelativeC1 {
    fn exec<B>(&self, cpu: &mut Cpu, _: &mut B)
    where
        B: Bus,
    {
        cpu.incr_pc();
    }
}

impl MicroFn for RelativeC2 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        let offset = bus.read(cpu.pc) as i8;
        cpu.incr_pc();

        // PC now points to next instruction
        let target = cpu.pc.wrapping_add(offset as u16);

        cpu.effective_addr = target; // Save target address
        cpu.crossed_page = (cpu.pc & 0xFF00) != (target & 0xFF00);
    }
}

impl MicroFn for RelativeC3 {
    fn exec<B>(&self, cpu: &mut Cpu, bus: &mut B)
    where
        B: Bus,
    {
        if cpu.crossed_page {
            // 6502 does a dummy read from: (old_pc & 0xFF00) | (target & 0x00FF)
            let wrong_addr = (cpu.pc & 0xFF00) | (cpu.effective_addr & 0x00FF);
            let _ = bus.read(wrong_addr);
        }
    }
}

impl MicroFn for RelativeC4 {
    fn exec<B>(&self, _cpu: &mut Cpu, _bus: &mut B)
    where
        B: Bus,
    {
        // TODO: 实现相对寻址周期4
    }
}
