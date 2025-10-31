use std::fmt::Display;

use crate::{bus::Bus, cpu::micro_op::MicroOp};

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
            Addressing::Implied => &[],
            Addressing::Accumulator => &[],

            Addressing::Immediate => {
                // Cycle 2: read immediate value into base_lo (or use directly in instruction)
                const IMM: [MicroOp; 1] = [MicroOp::fetch_zp_addr_lo()];
                &IMM
            }

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
            Addressing::Relative => {
                // Cycle 2: read signed offset → rel_offset, PC++
                const REL: [MicroOp; 1] = [MicroOp::fetch_rel_offset()];
                &REL
            }
        }
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
