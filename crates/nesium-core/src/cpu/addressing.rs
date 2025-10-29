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

// impl Addressing {
//     /// Returns the sequence of micro operations for this addressing mode.
//     ///
//     /// The micro operations represent the cycle-by-cycle steps required to
//     /// resolve the effective address or operand for this addressing mode.
//     /// This sequence stops once the operand is available, excluding the
//     /// actual instruction execution.
//     pub const fn micro_ops(self) -> &'static [MicroOp] {
//         match self {
//             Addressing::Implied => &[MicroOp::ImpliedC1],
//             Addressing::Accumulator => &[MicroOp::AccumulatorC1],

//             Addressing::Immediate => &[MicroOp::ImmediateC1, MicroOp::ImmediateC2],

//             Addressing::Absolute => &[
//                 MicroOp::AbsoluteC1,
//                 MicroOp::AbsoluteC2,
//                 MicroOp::AbsoluteC3,
//             ],

//             Addressing::AbsoluteX => &[
//                 MicroOp::AbsoluteXC1,
//                 MicroOp::AbsoluteXC2,
//                 MicroOp::AbsoluteXC3,
//                 MicroOp::AbsoluteXC4,
//             ],

//             Addressing::AbsoluteY => &[
//                 MicroOp::AbsoluteYC1,
//                 MicroOp::AbsoluteYC2,
//                 MicroOp::AbsoluteYC3,
//                 MicroOp::AbsoluteYC4,
//                 // Note: AbsoluteYC5 is used only when page boundary is crossed
//                 // and is handled dynamically during execution
//             ],

//             Addressing::Indirect => &[
//                 MicroOp::IndirectC1,
//                 MicroOp::IndirectC2,
//                 MicroOp::IndirectC3,
//                 MicroOp::IndirectC4,
//                 MicroOp::IndirectC5,
//             ],

//             Addressing::ZeroPage => &[MicroOp::ZeroPageC1, MicroOp::ZeroPageC2],

//             Addressing::ZeroPageX => &[
//                 MicroOp::ZeroPageXC1,
//                 MicroOp::ZeroPageXC2,
//                 MicroOp::ZeroPageXC3,
//                 MicroOp::ZeroPageXC4,
//             ],

//             Addressing::ZeroPageY => &[
//                 MicroOp::ZeroPageYC1,
//                 MicroOp::ZeroPageYC2,
//                 MicroOp::ZeroPageYC3,
//                 MicroOp::ZeroPageYC4,
//             ],

//             Addressing::IndirectX => &[
//                 MicroOp::IndirectXC1,
//                 MicroOp::IndirectXC2,
//                 MicroOp::IndirectXC3,
//                 MicroOp::IndirectXC4,
//                 MicroOp::IndirectXC5,
//                 MicroOp::IndirectXC6,
//             ],

//             Addressing::IndirectY => &[
//                 MicroOp::IndirectYC1,
//                 MicroOp::IndirectYC2,
//                 MicroOp::IndirectYC3,
//                 MicroOp::IndirectYC4,
//                 MicroOp::IndirectYC5,
//                 // Note: IndirectYC6 is used only when page boundary is crossed
//                 // and is handled dynamically during execution
//             ],

//             Addressing::Relative => &[
//                 MicroOp::RelativeC1,
//                 MicroOp::RelativeC2,
//                 MicroOp::RelativeC3,
//                 // Note: RelativeC4 is used only when page boundary is crossed
//                 // and branch is taken, handled dynamically during execution
//             ],
//         }
//     }
// }

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
