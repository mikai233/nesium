use std::fmt::Display;

use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{
        Cpu,
        lookup::LOOKUP_TABLE,
        timing::{CYCLE_TABLE, Timing},
    },
};

/// Represents the addressing modes supported by the 6502 CPU.
///
/// Addressing modes define how the CPU interprets the operand bytes
/// of an instruction to determine the effective memory address or
/// immediate value for the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Addressing {
    /// No additional data required. The instruction operates implicitly.
    ///
    /// # Examples
    /// - `CLC` (Clear Carry Flag)
    /// - `SEC` (Set Carry Flag)
    /// - `NOP` (No Operation)
    Implied,

    /// The operation is performed on the accumulator register.
    ///
    /// # Examples
    /// - `ASL A` (Arithmetic Shift Left Accumulator)
    /// - `LSR A` (Logical Shift Right Accumulator)
    Accumulator,

    /// The byte following the opcode is the operand value itself.
    ///
    /// # Examples
    /// - `LDA #$42` (Load Accumulator with immediate value $42)
    /// - `ORA #$FF` (OR Accumulator with immediate value $FF)
    Immediate,

    /// Uses the full 16-bit address specified by the two bytes following the opcode.
    ///
    /// # Examples
    /// - `LDA $1234` (Load Accumulator from address $1234)
    /// - `STA $5678` (Store Accumulator to address $5678)
    Absolute,

    /// Absolute address indexed by the X register.
    ///
    /// The effective address is calculated as `address + X`.
    /// May require an extra cycle if a page boundary is crossed.
    ///
    /// # Examples
    /// - `LDA $1234,X` (Load Accumulator from address $1234 + X)
    AbsoluteX,

    /// Absolute address indexed by the Y register.
    ///
    /// The effective address is calculated as `address + Y`.
    /// May require an extra cycle if a page boundary is crossed.
    ///
    /// # Examples
    /// - `LDA $1234,Y` (Load Accumulator from address $1234 + Y)
    AbsoluteY,

    /// Indirect addressing used exclusively by the JMP instruction.
    ///
    /// The two bytes following the opcode point to a memory location
    /// that contains the actual target address.
    ///
    /// # Examples
    /// - `JMP ($1234)` (Jump to the address stored at $1234)
    Indirect,

    /// Uses a single byte address that refers to the zero page ($0000-$00FF).
    ///
    /// Zero page accesses are faster and use fewer bytes than absolute addressing.
    ///
    /// # Examples
    /// - `LDA $42` (Load Accumulator from zero page address $42)
    ZeroPage,

    /// Zero page address indexed by the X register.
    ///
    /// The effective address wraps within the zero page: `(address + X) & 0xFF`.
    ///
    /// # Examples
    /// - `LDA $42,X` (Load Accumulator from zero page address $42 + X)
    ZeroPageX,

    /// Zero page address indexed by the Y register.
    ///
    /// The effective address wraps within the zero page: `(address + Y) & 0xFF`.
    ///
    /// # Examples
    /// - `LDX $42,Y` (Load X Register from zero page address $42 + Y)
    ZeroPageY,

    /// Pre-indexed indirect addressing (also known as "indexed indirect").
    ///
    /// The zero page address is first added to X, then used as a pointer
    /// to the actual operand address. Notation: `(zp,X)`
    ///
    /// # Examples
    /// - `LDA ($42,X)` (Load Accumulator using indirect addressing via $42 + X)
    IndirectX,

    /// Post-indexed indirect addressing (also known as "indirect indexed").
    ///
    /// The zero page address points to a pointer, which is then added to Y
    /// to form the effective address. Notation: `(zp),Y`
    ///
    /// # Examples
    /// - `LDA ($42),Y` (Load Accumulator using indirect addressing via $42 then + Y)
    IndirectY,

    /// Used for branch instructions. The operand is a signed 8-bit offset
    /// relative to the address of the next instruction.
    ///
    /// # Examples
    /// - `BNE $F0` (Branch if Not Equal to PC + $F0)
    /// - `BCC $05` (Branch if Carry Clear to PC + $05)
    Relative,
}

impl Addressing {
    /// Number of operand bytes following the opcode.
    pub const fn operand_len(&self) -> usize {
        match self {
            Addressing::Implied | Addressing::Accumulator => 0,
            Addressing::Immediate
            | Addressing::ZeroPage
            | Addressing::ZeroPageX
            | Addressing::ZeroPageY
            | Addressing::IndirectX
            | Addressing::IndirectY
            | Addressing::Relative => 1,
            Addressing::Absolute
            | Addressing::AbsoluteX
            | Addressing::AbsoluteY
            | Addressing::Indirect => 2,
        }
    }

    /// Longest addressing sequence length (in cycles) for static-dispatch exec.
    ///
    /// Includes optional page-cross cycles; modes without addressing cycles return 0.
    pub const fn exec_len(&self) -> u8 {
        match self {
            Addressing::Implied | Addressing::Accumulator | Addressing::Immediate => 0,
            Addressing::Absolute => 2,
            Addressing::AbsoluteX | Addressing::AbsoluteY => 3, // includes possible cross-page dummy
            Addressing::Indirect => 4,
            Addressing::ZeroPage => 1,
            Addressing::ZeroPageX | Addressing::ZeroPageY => 2,
            Addressing::IndirectX | Addressing::IndirectY => 4, // includes possible cross-page dummy
            Addressing::Relative => 0,
        }
    }

    /// Static-dispatch addressing executor (prototype; not yet wired into CPU).
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
        match self {
            Addressing::Implied | Addressing::Accumulator | Addressing::Immediate => {
                unreachable_step!("no addressing cycles for {:?}", self)
            }
            Addressing::Absolute => exec_absolute(cpu, bus, ctx, step),
            Addressing::AbsoluteX => exec_absolute_x(cpu, bus, ctx, step),
            Addressing::AbsoluteY => exec_absolute_y(cpu, bus, ctx, step),
            Addressing::Indirect => exec_indirect(cpu, bus, ctx, step),
            Addressing::ZeroPage => exec_zero_page(cpu, bus, ctx, step),
            Addressing::ZeroPageX => exec_zero_page_x(cpu, bus, ctx, step),
            Addressing::ZeroPageY => exec_zero_page_y(cpu, bus, ctx, step),
            Addressing::IndirectX => exec_indirect_x(cpu, bus, ctx, step),
            Addressing::IndirectY => exec_indirect_y(cpu, bus, ctx, step),
            Addressing::Relative => unreachable_step!("no addressing cycles for Relative"),
        }
    }

    pub(crate) const fn has_page_cross_penalty(&self) -> bool {
        matches!(
            self,
            Addressing::AbsoluteX
                | Addressing::AbsoluteY
                | Addressing::IndirectY
                | Addressing::Relative
        )
    }

    pub(crate) fn page_crossed(base_addr: u16, effective_addr: u16) -> bool {
        (base_addr & 0xFF00) != (effective_addr & 0xFF00)
    }

    pub(crate) const fn forces_dummy_read_cycle(opcode: u8) -> bool {
        let timing = CYCLE_TABLE[opcode as usize];
        let instr = &LOOKUP_TABLE[opcode as usize];
        instr.addressing.has_page_cross_penalty() && matches!(timing, Timing::Fixed(_))
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            // 6502 manuals/disassemblers often treat implied as no operand.
            Addressing::Implied => "impl",

            // Accumulator addressing is written as operand "A".
            Addressing::Accumulator => "A",

            // Immediate is typically shown with a leading '#'.
            Addressing::Immediate => "#",

            // Generic names used in many 6502 disassemblers.
            Addressing::Absolute => "abs",
            Addressing::AbsoluteX => "abs,X",
            Addressing::AbsoluteY => "abs,Y",
            Addressing::Indirect => "(abs)",

            Addressing::ZeroPage => "zp",
            Addressing::ZeroPageX => "zp,X",
            Addressing::ZeroPageY => "zp,Y",

            Addressing::IndirectX => "(zp,X)",
            Addressing::IndirectY => "(zp),Y",

            Addressing::Relative => "rel",
        };

        f.write_str(s)
    }
}

fn exec_absolute(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        1 => {
            let hi = cpu.fetch_u8(bus, ctx);
            cpu.effective_addr |= (hi as u16) << 8;
        }
        _ => unreachable_step!("invalid Absolute step {step}"),
    }
}

fn exec_absolute_x(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        1 => {
            let hi = cpu.fetch_u8(bus, ctx);
            let base = ((hi as u16) << 8) | cpu.effective_addr;
            cpu.tmp = hi;
            // if cpu.opcode_in_flight == Some(0x9C) {
            //     cpu.base = hi;
            // }
            let addr = base.wrapping_add(cpu.x as u16);
            cpu.skip_optional_dummy_read_cycle(base, addr);
            cpu.effective_addr = addr;
        }
        2 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
            let page_crossed = Addressing::page_crossed(base, cpu.effective_addr);
            let dummy_addr = if page_crossed {
                cpu.effective_addr.wrapping_sub(0x100)
            } else {
                // Force dummy read cycle
                cpu.effective_addr
            };
            cpu.dummy_read_at(dummy_addr, bus, ctx);
        }
        _ => unreachable_step!("invalid AbsoluteX step {step}"),
    }
}

fn exec_absolute_y(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        1 => {
            let hi = cpu.fetch_u8(bus, ctx);
            let base = ((hi as u16) << 8) | cpu.effective_addr;
            cpu.tmp = hi;
            // if cpu.opcode_in_flight == Some(0x9F)
            //     || cpu.opcode_in_flight == Some(0x9E)
            //     || cpu.opcode_in_flight == Some(0x9B)
            // {
            //     cpu.base = hi;
            // }
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.skip_optional_dummy_read_cycle(base, addr);
            cpu.effective_addr = addr;
        }
        2 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let page_crossed = Addressing::page_crossed(base, cpu.effective_addr);
            let dummy_addr = if page_crossed {
                cpu.effective_addr.wrapping_sub(0x100)
            } else {
                // Force dummy read cycle
                cpu.effective_addr
            };
            cpu.dummy_read_at(dummy_addr, bus, ctx);
        }
        _ => unreachable_step!("invalid AbsoluteY step {step}"),
    }
}

fn exec_indirect(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        1 => {
            let hi = cpu.fetch_u8(bus, ctx);
            cpu.effective_addr |= (hi as u16) << 8;
        }
        2 => {
            cpu.tmp = bus.mem_read(cpu.effective_addr, cpu, ctx);
        }
        3 => {
            let hi_addr = if (cpu.effective_addr & 0xFF) == 0xFF {
                cpu.effective_addr & 0xFF00
            } else {
                cpu.effective_addr + 1
            };
            let hi = bus.mem_read(hi_addr, cpu, ctx);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
        }
        _ => unreachable_step!("invalid Indirect step {step}"),
    }
}

fn exec_zero_page(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        _ => unreachable_step!("invalid ZeroPage step {step}"),
    }
}

fn exec_zero_page_x(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        // T1: Fetch zero-page base address (BAL) from the instruction stream.
        // Operand is an 8-bit address; high byte is implicitly $00.
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }

        // T2: Dummy read at $00:BAL (discarded).
        // The real 6502 performs this extra read while the internal adder
        // computes (BAL + X) to form the final zero-page effective address.
        //
        // After this cycle, `effective_addr` holds the indexed zero-page address
        // that will be used by the instruction execution micro-ops on the next cycle.
        1 => {
            let base = cpu.effective_addr & 0x00FF;
            cpu.dummy_read_at(base, bus, ctx);

            let addr = (cpu.effective_addr + cpu.x as u16) & 0x00FF; // wrap within zero page
            cpu.effective_addr = addr;
        }

        _ => unreachable_step!("invalid ZeroPageX step {step}"),
    }
}

fn exec_zero_page_y(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        // T1: Fetch zero-page base address (BAL) from the instruction stream.
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }

        // T2: Dummy read at $00:BAL (discarded), then compute (BAL + Y).
        // Same timing behavior as ZeroPage,X, but using the Y index register.
        1 => {
            let base = cpu.effective_addr & 0x00FF;
            cpu.dummy_read_at(base, bus, ctx);

            let addr = (cpu.effective_addr + cpu.y as u16) & 0x00FF; // wrap within zero page
            cpu.effective_addr = addr;
        }

        _ => unreachable_step!("invalid ZeroPageY step {step}"),
    }
}

fn exec_indirect_x(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        // T1: fetch zero-page base address (BAL)
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16; // BAL in low byte
        }

        // T2: dummy read at $00:BAL (discarded), then compute (BAL + X)
        1 => {
            let base = cpu.effective_addr & 0x00FF;
            cpu.dummy_read_at(base, bus, ctx);

            // reuse effective_addr to hold the indexed zero-page pointer location
            cpu.effective_addr = (base + cpu.x as u16) & 0x00FF;
        }

        // T3: fetch low byte of effective address from $00:(BAL+X)
        2 => {
            let ptr = cpu.effective_addr & 0x00FF;
            cpu.tmp = bus.mem_read(ptr, cpu, ctx); // ADL
        }

        // T4: fetch high byte from $00:(BAL+X+1), then form 16-bit effective address
        3 => {
            let ptr = cpu.effective_addr & 0x00FF;
            let hi = bus.mem_read((ptr + 1) & 0x00FF, cpu, ctx); // ADH (wrap in zero page)
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
        }

        _ => unreachable_step!("invalid IndirectX step {step}"),
    }
}

fn exec_indirect_y(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
        }
        1 => {
            cpu.tmp = bus.mem_read(cpu.effective_addr, cpu, ctx);
        }
        2 => {
            let hi_addr = (cpu.effective_addr + 1) & 0x00FF;
            let hi = bus.mem_read(hi_addr, cpu, ctx);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            cpu.tmp = hi;
            // if cpu.opcode_in_flight == Some(0x93) {
            //     cpu.base = hi;
            // }
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.skip_optional_dummy_read_cycle(base, addr);
            cpu.effective_addr = addr;
        }
        3 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let page_crossed = Addressing::page_crossed(base, cpu.effective_addr);
            let dummy_addr = if page_crossed {
                cpu.effective_addr.wrapping_sub(0x100)
            } else {
                // Force dummy read cycle
                cpu.effective_addr
            };
            cpu.dummy_read_at(dummy_addr, bus, ctx);
        }
        _ => unreachable_step!("invalid IndirectY step {step}"),
    }
}
