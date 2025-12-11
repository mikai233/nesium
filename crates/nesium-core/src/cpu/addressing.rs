use std::fmt::Display;

use crate::{
    bus::Bus,
    cpu::{Cpu, micro_op::MicroOp, unreachable_step},
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

    /// Returns the sequence of micro-ops to execute **after the opcode has been fetched**.
    ///
    /// - All sequences **exclude** the opcode fetch cycle (handled during decode).
    /// - Extra cycles due to page crossing are handled via `dummy_read_cross_*`
    ///   (automatically based on `crossed_page`).
    /// - At completion, `effective_addr`, `zp_addr`, `rel_offset`, etc. are ready
    ///   for the instruction execution phase.
    pub const fn micro_ops(&self) -> &'static [MicroOp] {
        match self {
            // ─────────────────────────────────────────────────────────────────────
            //  Implied / Accumulator / Immediate
            // ─────────────────────────────────────────────────────────────────────
            Addressing::Implied | Addressing::Accumulator | Addressing::Immediate => &[],

            // ─────────────────────────────────────────────────────────────────────
            //  Absolute
            // ─────────────────────────────────────────────────────────────────────
            Addressing::Absolute => {
                // Cycle 2: low  → base_lo, PC++
                // Cycle 3: high → effective_addr, PC++
                const ABS: [MicroOp; 2] =
                    [MicroOp::fetch_abs_addr_lo(), MicroOp::fetch_abs_addr_hi()];
                &ABS
            }

            // ─────────────────────────────────────────────────────────────────────
            //  Absolute,X
            // ─────────────────────────────────────────────────────────────────────
            Addressing::AbsoluteX => {
                // Cycle 2: low  → base_lo, PC++
                // Cycle 3: high + X → effective_addr, detect page cross, PC++
                // Cycle 4 (if cross): dummy read
                const ABSX: [MicroOp; 3] = [
                    MicroOp::fetch_abs_addr_lo(),
                    MicroOp::fetch_abs_addr_hi_add_x(),
                    MicroOp::dummy_read_cross_x(),
                ];
                &ABSX
            }

            // ─────────────────────────────────────────────────────────────────────
            //  Absolute,Y
            // ─────────────────────────────────────────────────────────────────────
            Addressing::AbsoluteY => {
                const ABSY: [MicroOp; 3] = [
                    MicroOp::fetch_abs_addr_lo(),
                    MicroOp::fetch_abs_addr_hi_add_y(),
                    MicroOp::dummy_read_cross_y(),
                ];
                &ABSY
            }

            // ─────────────────────────────────────────────────────────────────────
            //  Indirect (JMP only) – with 6502 page-boundary bug
            // ─────────────────────────────────────────────────────────────────────
            Addressing::Indirect => {
                // Cycle 2: low  of pointer → base_lo, PC++
                // Cycle 3: high of pointer → effective_addr = pointer, PC++
                // Cycle 4: read low  byte of target → tmp
                // Cycle 5: read high byte of target (buggy wrap if ptr & 0xFF == 0xFF)
                const INDIRECT: [MicroOp; 4] = [
                    MicroOp::fetch_abs_addr_lo(),
                    MicroOp::fetch_abs_addr_hi(),
                    MicroOp::read_indirect_lo(),       // target low → tmp
                    MicroOp::read_indirect_hi_buggy(), // target high → effective_addr (final)
                ];
                &INDIRECT
            }

            // ─────────────────────────────────────────────────────────────────────
            //  ZeroPage
            // ─────────────────────────────────────────────────────────────────────
            Addressing::ZeroPage => {
                // Cycle 2: read zero-page address → zp_addr, PC++
                const ZP: [MicroOp; 1] = [MicroOp::fetch_zp_addr_lo()];
                &ZP
            }

            // ─────────────────────────────────────────────────────────────────────
            //  ZeroPage,X
            // ─────────────────────────────────────────────────────────────────────
            Addressing::ZeroPageX => {
                // Cycle 2: read zp address → zp_addr, PC++
                // Cycle 3: (zp + X) & 0xFF, dummy read → effective_addr
                const ZPX: [MicroOp; 2] = [
                    MicroOp::fetch_zp_addr_lo(),
                    MicroOp::read_zero_page_add_x_dummy(),
                ];
                &ZPX
            }

            // ─────────────────────────────────────────────────────────────────────
            //  ZeroPage,Y (used by LDX, STX)
            // ─────────────────────────────────────────────────────────────────────
            Addressing::ZeroPageY => {
                const ZPY: [MicroOp; 2] = [
                    MicroOp::fetch_zp_addr_lo(),
                    MicroOp::read_zero_page_add_y_dummy(),
                ];
                &ZPY
            }

            // ─────────────────────────────────────────────────────────────────────
            //  (Indirect,X) – Pre-indexed indirect
            // ─────────────────────────────────────────────────────────────────────
            Addressing::IndirectX => {
                // Cycle 2: read zp pointer → zp_addr, PC++
                // Cycle 3: dummy read (zp + X) & 0xFF
                // Cycle 4: read low  from (zp + X)
                // Cycle 5: read high from (zp + X + 1)
                const INDX: [MicroOp; 4] = [
                    MicroOp::fetch_zp_addr_lo(),
                    MicroOp::read_indirect_x_dummy(),
                    MicroOp::read_indirect_x_lo(),
                    MicroOp::read_indirect_x_hi(),
                ];
                &INDX
            }

            // ─────────────────────────────────────────────────────────────────────
            //  (Indirect),Y – Post-indexed indirect
            // ─────────────────────────────────────────────────────────────────────
            Addressing::IndirectY => {
                // Cycle 2: read zp pointer → zp_addr, PC++
                // Cycle 3: read low  from zp → base_lo
                // Cycle 4: read high from (zp+1), add Y, detect cross → effective_addr
                // Cycle 5 (if cross): dummy read
                const INDY: [MicroOp; 4] = [
                    MicroOp::fetch_zp_addr_lo(),
                    MicroOp::read_zero_page(),     // low → base_lo
                    MicroOp::read_indirect_y_hi(), // high + Y
                    MicroOp::dummy_read_cross_y(),
                ];
                &INDY
            }

            // ─────────────────────────────────────────────────────────────────────
            //  Relative (branch instructions)
            // ─────────────────────────────────────────────────────────────────────
            Addressing::Relative => &[],
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
    pub(crate) fn exec<B: Bus>(&self, cpu: &mut Cpu, bus: &mut B, step: u8) {
        match self {
            Addressing::Implied | Addressing::Accumulator | Addressing::Immediate => {
                unreachable_step!("no addressing cycles for {:?}", self)
            }
            Addressing::Absolute => exec_absolute(cpu, bus, step),
            Addressing::AbsoluteX => exec_absolute_x(cpu, bus, step),
            Addressing::AbsoluteY => exec_absolute_y(cpu, bus, step),
            Addressing::Indirect => exec_indirect(cpu, bus, step),
            Addressing::ZeroPage => exec_zero_page(cpu, bus, step),
            Addressing::ZeroPageX => exec_zero_page_x(cpu, bus, step),
            Addressing::ZeroPageY => exec_zero_page_y(cpu, bus, step),
            Addressing::IndirectX => exec_indirect_x(cpu, bus, step),
            Addressing::IndirectY => exec_indirect_y(cpu, bus, step),
            Addressing::Relative => unreachable_step!("no addressing cycles for Relative"),
        }
    }

    pub(crate) const fn maybe_cross_page(&self) -> bool {
        matches!(
            self,
            Addressing::AbsoluteX
                | Addressing::AbsoluteY
                | Addressing::IndirectY
                | Addressing::Relative
        )
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Addressing::Implied => "implied".fmt(f),
            Addressing::Accumulator => "accumulator".fmt(f),
            Addressing::Immediate => "immediate".fmt(f),
            Addressing::Absolute => "absolute".fmt(f),
            Addressing::AbsoluteX => "absolute_x".fmt(f),
            Addressing::AbsoluteY => "absolute_y".fmt(f),
            Addressing::Indirect => "indirect".fmt(f),
            Addressing::ZeroPage => "zero_page".fmt(f),
            Addressing::ZeroPageX => "zero_page_x".fmt(f),
            Addressing::ZeroPageY => "zero_page_y".fmt(f),
            Addressing::IndirectX => "indirect_x".fmt(f),
            Addressing::IndirectY => "indirect_y".fmt(f),
            Addressing::Relative => "relative".fmt(f),
        }
    }
}

fn exec_absolute<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let hi = bus.mem_read(cpu, cpu.pc);
            cpu.effective_addr |= (hi as u16) << 8;
            cpu.incr_pc();
        }
        _ => unreachable_step!("invalid Absolute step {step}"),
    }
}

fn exec_absolute_x<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let hi = bus.mem_read(cpu, cpu.pc);
            let base = ((hi as u16) << 8) | cpu.effective_addr;
            if cpu.opcode_in_flight == Some(0x9C) {
                cpu.base = hi;
            }
            let addr = base.wrapping_add(cpu.x as u16);
            cpu.check_cross_page(base, addr);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        }
        2 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
            let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
            let _ = bus.mem_read(cpu, dummy_addr);
        }
        _ => unreachable_step!("invalid AbsoluteX step {step}"),
    }
}

fn exec_absolute_y<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let hi = bus.mem_read(cpu, cpu.pc);
            let base = ((hi as u16) << 8) | cpu.effective_addr;
            if cpu.opcode_in_flight == Some(0x9F)
                || cpu.opcode_in_flight == Some(0x9E)
                || cpu.opcode_in_flight == Some(0x9B)
            {
                cpu.base = hi;
            }
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page(base, addr);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        }
        2 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
            let _ = bus.mem_read(cpu, dummy_addr);
        }
        _ => unreachable_step!("invalid AbsoluteY step {step}"),
    }
}

fn exec_indirect<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let hi = bus.mem_read(cpu, cpu.pc);
            cpu.effective_addr |= (hi as u16) << 8;
            cpu.incr_pc();
        }
        2 => {
            cpu.base = bus.mem_read(cpu, cpu.effective_addr);
        }
        3 => {
            let hi_addr = if (cpu.effective_addr & 0xFF) == 0xFF {
                cpu.effective_addr & 0xFF00
            } else {
                cpu.effective_addr + 1
            };
            let hi = bus.mem_read(cpu, hi_addr);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.base as u16);
        }
        _ => unreachable_step!("invalid Indirect step {step}"),
    }
}

fn exec_zero_page<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        _ => unreachable_step!("invalid ZeroPage step {step}"),
    }
}

fn exec_zero_page_x<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let addr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
            let _ = bus.mem_read(cpu, addr);
            cpu.effective_addr = addr;
        }
        _ => unreachable_step!("invalid ZeroPageX step {step}"),
    }
}

fn exec_zero_page_y<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let addr = (cpu.effective_addr + cpu.y as u16) & 0x00FF;
            let _ = bus.mem_read(cpu, addr);
            cpu.effective_addr = addr;
        }
        _ => unreachable_step!("invalid ZeroPageY step {step}"),
    }
}

fn exec_indirect_x<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            let ptr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
            let _ = bus.mem_read(cpu, ptr);
        }
        2 => {
            let ptr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
            cpu.base = bus.mem_read(cpu, ptr);
        }
        3 => {
            let ptr = (cpu.effective_addr + cpu.x as u16 + 1) & 0x00FF;
            let hi = bus.mem_read(cpu, ptr);
            cpu.effective_addr = ((hi as u16) << 8) | cpu.base as u16;
        }
        _ => unreachable_step!("invalid IndirectX step {step}"),
    }
}

fn exec_indirect_y<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            cpu.effective_addr = bus.mem_read(cpu, cpu.pc) as u16;
            cpu.incr_pc();
        }
        1 => {
            cpu.base = bus.mem_read(cpu, cpu.effective_addr);
        }
        2 => {
            let hi_addr = (cpu.effective_addr + 1) & 0x00FF;
            let hi = bus.mem_read(cpu, hi_addr);
            let base = ((hi as u16) << 8) | (cpu.base as u16);
            if cpu.opcode_in_flight == Some(0x93) {
                cpu.base = hi;
            }
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page(base, addr);
            cpu.effective_addr = addr;
        }
        3 => {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
            let _ = bus.mem_read(cpu, dummy_addr);
        }
        _ => unreachable_step!("invalid IndirectY step {step}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bus::mock::MockBus,
        cpu::{Cpu, lookup::LOOKUP_TABLE},
        reset_kind::ResetKind,
    };

    fn opcode_for_mode(mode: Addressing) -> u8 {
        LOOKUP_TABLE
            .iter()
            .enumerate()
            .find(|(_, instr)| instr.addressing == mode)
            .map(|(op, _)| op as u8)
            .expect("no opcode found for addressing mode")
    }

    fn seed_bus_for_mode(mode: Addressing, bus: &mut MockBus) {
        match mode {
            Addressing::Absolute | Addressing::AbsoluteX | Addressing::AbsoluteY => {
                // Base address 0x00FF so adding X/Y=1 crosses the page.
                bus.mem_write(0, 0xFF);
                bus.mem_write(1, 0x00);
            }
            Addressing::Indirect => {
                // Pointer at $0000 -> target at $1000 -> value $5678.
                bus.mem_write(0, 0x00);
                bus.mem_write(1, 0x10);
                bus.mem_write(0x1000, 0x78);
                bus.mem_write(0x1001, 0x56);
            }
            Addressing::ZeroPage | Addressing::ZeroPageX | Addressing::ZeroPageY => {
                bus.mem_write(0, 0x80);
            }
            Addressing::IndirectX => {
                bus.mem_write(0, 0x10); // zp pointer
                bus.mem_write(0x14, 0xAA); // low
                bus.mem_write(0x15, 0xBB); // high
            }
            Addressing::IndirectY => {
                bus.mem_write(0, 0x20); // zp pointer
                bus.mem_write(0x20, 0x00); // low
                bus.mem_write(0x21, 0x01); // high -> base 0x0100, Y=1 crosses
            }
            _ => {}
        }
    }

    /// Ensure `exec_len()` stays in sync with the actual `exec` step table for each addressing mode.
    #[test]
    fn exec_len_matches_steps() {
        let modes = [
            Addressing::Absolute,
            Addressing::AbsoluteX,
            Addressing::AbsoluteY,
            Addressing::Indirect,
            Addressing::ZeroPage,
            Addressing::ZeroPageX,
            Addressing::ZeroPageY,
            Addressing::IndirectX,
            Addressing::IndirectY,
        ];

        for mode in modes {
            let opcode = opcode_for_mode(mode);
            let len = mode.exec_len();
            let mut cpu = Cpu::new();
            let mut bus = MockBus::default();
            seed_bus_for_mode(mode, &mut bus);
            cpu.reset(&mut bus, ResetKind::PowerOn);
            cpu.opcode_in_flight = Some(opcode);
            cpu.x = 1;
            cpu.y = 1;

            // Each valid step 0..len should succeed.
            for step in 0..len {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mode.exec(&mut cpu, &mut bus, step as u8);
                }));
                assert!(
                    result.is_ok(),
                    "addressing {:?} failed at step {} (exec_len={})",
                    mode,
                    step,
                    len
                );
            }

            // Stepping past len should hit unreachable in debug (guards regressions if table changes).
            let past = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                mode.exec(&mut cpu, &mut bus, len as u8);
            }));
            assert!(
                past.is_err(),
                "addressing {:?} unexpectedly allowed step {} (exec_len={})",
                mode,
                len,
                len
            );
        }
    }
}
