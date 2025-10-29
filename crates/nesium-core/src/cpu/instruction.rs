use std::fmt::Display;

use crate::cpu::{addressing::AddressingMode, micro_op::MicroOp, status::Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Mnemonic {
    //Load/Store
    LAS,
    LAX,
    LDA,
    LDX,
    LDY,
    SAX,
    SHA,
    SHX,
    SHY,
    STA,
    STX,
    STY,
    //Transfer
    SHS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    //Stack
    PHA,
    PHP,
    PLA,
    PLP,
    //Shift
    ASL,
    LSR,
    ROL,
    ROR,
    //Logic
    AND,
    BIT,
    EOR,
    ORA,
    //Arithmetic
    ADC,
    ANC,
    ARR,
    ASR,
    CMP,
    CPX,
    CPY,
    DCP,
    ISC,
    RLA,
    RRA,
    SBC,
    SBX,
    SLO,
    SRE,
    XAA,
    //Arithmetic: Inc/Dec
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    //Control Flow
    BRK,
    JMP,
    JSR,
    RTI,
    RTS,
    //Control Flow: Branch
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    //Flags
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    //KIL
    JAM,
    //NOP
    NOP,
}

macro_rules! status {
    // Entry point — accepts multiple flag:value pairs
    (
        $status:expr, $result:expr, $carry:expr, $overflow:expr;
        $flag:ident : $val:tt $(, $($rest:tt)*)?
    ) => {{
        __update_flag!($status, $result, $carry, $overflow, $flag, $val);
        $(
            status!($status, $result, $carry, $overflow; $($rest)*);
        )?
    }};

    // Empty case — end recursion
    ($status:expr, $result:expr, $carry:expr, $overflow:expr;) => {};
}

// Internal helper macro (not exported)
macro_rules! __update_flag {
    // --- N: Negative ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, N, 0) => {
        $status.remove(Status::NEGATIVE);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, N, 1) => {
        $status.insert(Status::NEGATIVE);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, N, *) => {
        $status.update_negative($result);
    };

    // --- Z: Zero ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, Z, 0) => {
        $status.remove(Status::ZERO);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, Z, 1) => {
        $status.insert(Status::ZERO);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, Z, *) => {
        $status.update_zero($result);
    };

    // --- C: Carry ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, C, 0) => {
        $status.remove(Status::CARRY);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, C, 1) => {
        $status.insert(Status::CARRY);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, C, *) => {
        if let Some(c) = $carry {
            $status.set(Status::CARRY, c);
        }
    };

    // --- V: Overflow ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, V, 0) => {
        $status.remove(Status::OVERFLOW);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, V, 1) => {
        $status.insert(Status::OVERFLOW);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, V, *) => {
        if let Some(v) = $overflow {
            $status.set(Status::OVERFLOW, v);
        }
    };

    // --- Other simple flags (I, D, B, U) ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, I, 0) => {
        $status.remove(Status::INTERRUPT);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, I, 1) => {
        $status.insert(Status::INTERRUPT);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, D, 0) => {
        $status.remove(Status::DECIMAL);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, D, 1) => {
        $status.insert(Status::DECIMAL);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, B, 0) => {
        $status.remove(Status::BREAK);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, B, 1) => {
        $status.insert(Status::BREAK);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, U, 0) => {
        $status.remove(Status::UNUSED);
    };
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, U, 1) => {
        $status.insert(Status::UNUSED);
    };

    // --- Fallback case: unknown flag or invalid value ---
    ($status:expr, $result:expr, $carry:expr, $overflow:expr, $flag:ident, $val:tt) => {
        compile_error!(concat!(
            "Invalid flag or value in status!(): ",
            stringify!($flag),
            ":",
            stringify!($val),
            ". Allowed flags: N,Z,C,V,I,D,B,U; allowed values: 0,1,*"
        ));
    };
}

impl Mnemonic {
    /// Update the processor status flags according to the instruction semantics.
    /// `result` is the value that affects N/Z.
    /// `carry` and `overflow` are optional flags affected by ADC/SBC, shift/rotate, compare.
    pub(crate) fn update_status(
        &self,
        status: &mut Status,
        result: u8,
        carry: Option<bool>,
        overflow: Option<bool>,
    ) {
        match self {
            //Load/Store
            Mnemonic::LAS | Mnemonic::LAX | Mnemonic::LDA | Mnemonic::LDX | Mnemonic::LDY => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Mnemonic::SAX
            | Mnemonic::SHA
            | Mnemonic::SHX
            | Mnemonic::SHY
            | Mnemonic::STA
            | Mnemonic::STX
            | Mnemonic::STY => {}
            //Transfer
            Mnemonic::SHS => {}
            Mnemonic::TAX | Mnemonic::TAY | Mnemonic::TSX | Mnemonic::TXA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Mnemonic::TXS => {}
            Mnemonic::TYA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Stack
            Mnemonic::PHA | Mnemonic::PHP => {}
            Mnemonic::PLA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Mnemonic::PLP => {
                // Restore all flags from stack value
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            //Shift
            Mnemonic::ASL => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::LSR => {
                status!(status, result, carry, overflow; N:0, Z:*, C:*);
            }
            Mnemonic::ROL | Self::ROR => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::AND => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Mnemonic::BIT => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*);
            }
            Mnemonic::EOR | Mnemonic::ORA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Arithmetic
            Mnemonic::ADC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Mnemonic::ANC => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::ARR => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Mnemonic::ASR => {
                status!(status, result, carry, overflow; N:0, Z:*, C:*);
            }
            Mnemonic::CMP | Mnemonic::CPX | Mnemonic::CPY | Mnemonic::DCP => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::ISC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Mnemonic::RLA => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::RRA | Mnemonic::SBC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Mnemonic::SBX | Mnemonic::SLO | Mnemonic::SRE => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Mnemonic::XAA
            | Mnemonic::DEC
            | Mnemonic::DEX
            | Mnemonic::DEY
            | Mnemonic::INC
            | Mnemonic::INX
            | Mnemonic::INY => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Control Flow
            Mnemonic::BRK => {
                status!(status, result, carry, overflow; I:1);
            }
            Mnemonic::JMP | Mnemonic::JSR => {}
            Mnemonic::RTI => {
                //TODO
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            Mnemonic::RTS
            | Mnemonic::BCC
            | Mnemonic::BCS
            | Mnemonic::BEQ
            | Mnemonic::BMI
            | Mnemonic::BNE
            | Mnemonic::BPL
            | Mnemonic::BVC
            | Mnemonic::BVS => {}
            //Flags
            Mnemonic::CLC => {
                status!(status, result, carry, overflow; C:0);
            }
            Mnemonic::CLD => {
                status!(status, result, carry, overflow; D:0);
            }
            Mnemonic::CLI => {
                status!(status, result, carry, overflow; I:0);
            }
            Mnemonic::CLV => {
                status!(status, result, carry, overflow; V:0);
            }
            Mnemonic::SEC => {
                status!(status, result, carry, overflow; C:1);
            }
            Mnemonic::SED => {
                status!(status, result, carry, overflow; D:1);
            }
            Mnemonic::SEI => {
                status!(status, result, carry, overflow; I:1);
            }
            Mnemonic::JAM | Mnemonic::NOP => {}
        }
    }

    pub(crate) const fn micro_ops(&self) -> &'static [MicroOp] {
        match self {
            Mnemonic::LAS => todo!(),
            Mnemonic::LAX => todo!(),
            Mnemonic::LDA => todo!(),
            Mnemonic::LDX => todo!(),
            Mnemonic::LDY => todo!(),
            Mnemonic::SAX => todo!(),
            Mnemonic::SHA => todo!(),
            Mnemonic::SHX => todo!(),
            Mnemonic::SHY => todo!(),
            Mnemonic::STA => todo!(),
            Mnemonic::STX => todo!(),
            Mnemonic::STY => todo!(),
            Mnemonic::SHS => todo!(),
            Mnemonic::TAX => todo!(),
            Mnemonic::TAY => todo!(),
            Mnemonic::TSX => todo!(),
            Mnemonic::TXA => todo!(),
            Mnemonic::TXS => todo!(),
            Mnemonic::TYA => todo!(),
            Mnemonic::PHA => todo!(),
            Mnemonic::PHP => todo!(),
            Mnemonic::PLA => todo!(),
            Mnemonic::PLP => todo!(),
            Mnemonic::ASL => todo!(),
            Mnemonic::LSR => todo!(),
            Mnemonic::ROL => todo!(),
            Mnemonic::ROR => todo!(),
            Mnemonic::AND => todo!(),
            Mnemonic::BIT => todo!(),
            Mnemonic::EOR => todo!(),
            Mnemonic::ORA => todo!(),
            Mnemonic::ADC => todo!(),
            Mnemonic::ANC => todo!(),
            Mnemonic::ARR => todo!(),
            Mnemonic::ASR => todo!(),
            Mnemonic::CMP => todo!(),
            Mnemonic::CPX => todo!(),
            Mnemonic::CPY => todo!(),
            Mnemonic::DCP => todo!(),
            Mnemonic::ISC => todo!(),
            Mnemonic::RLA => todo!(),
            Mnemonic::RRA => todo!(),
            Mnemonic::SBC => todo!(),
            Mnemonic::SBX => todo!(),
            Mnemonic::SLO => todo!(),
            Mnemonic::SRE => todo!(),
            Mnemonic::XAA => todo!(),
            Mnemonic::DEC => todo!(),
            Mnemonic::DEX => todo!(),
            Mnemonic::DEY => todo!(),
            Mnemonic::INC => todo!(),
            Mnemonic::INX => todo!(),
            Mnemonic::INY => todo!(),
            Mnemonic::BRK => todo!(),
            Mnemonic::JMP => todo!(),
            Mnemonic::JSR => todo!(),
            Mnemonic::RTI => todo!(),
            Mnemonic::RTS => todo!(),
            Mnemonic::BCC => todo!(),
            Mnemonic::BCS => todo!(),
            Mnemonic::BEQ => todo!(),
            Mnemonic::BMI => todo!(),
            Mnemonic::BNE => todo!(),
            Mnemonic::BPL => todo!(),
            Mnemonic::BVC => todo!(),
            Mnemonic::BVS => todo!(),
            Mnemonic::CLC => todo!(),
            Mnemonic::CLD => todo!(),
            Mnemonic::CLI => todo!(),
            Mnemonic::CLV => todo!(),
            Mnemonic::SEC => todo!(),
            Mnemonic::SED => todo!(),
            Mnemonic::SEI => todo!(),
            Mnemonic::JAM => todo!(),
            Mnemonic::NOP => todo!(),
        }
    }

    /// Returns the 6502 instruction set opcode table.
    ///
    /// This table maps each 256-byte opcode to its corresponding mnemonic and addressing mode.
    /// The table includes both official instructions and undocumented opcodes.
    pub(crate) const fn table() -> &'static [(Mnemonic, AddressingMode); 256] {
        &[
            // 0x00-0x0F
            (Mnemonic::BRK, AddressingMode::Implied),
            (Mnemonic::ORA, AddressingMode::IndirectX),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::SLO, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::ZeroPage),
            (Mnemonic::ORA, AddressingMode::ZeroPage),
            (Mnemonic::ASL, AddressingMode::ZeroPage),
            (Mnemonic::SLO, AddressingMode::ZeroPage),
            (Mnemonic::PHP, AddressingMode::Implied),
            (Mnemonic::ORA, AddressingMode::Immediate),
            (Mnemonic::ASL, AddressingMode::Accumulator),
            (Mnemonic::ANC, AddressingMode::Immediate),
            (Mnemonic::NOP, AddressingMode::Absolute),
            (Mnemonic::ORA, AddressingMode::Absolute),
            (Mnemonic::ASL, AddressingMode::Absolute),
            (Mnemonic::SLO, AddressingMode::Absolute),
            // 0x10-0x1F
            (Mnemonic::BPL, AddressingMode::Relative),
            (Mnemonic::ORA, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::SLO, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::ORA, AddressingMode::ZeroPageX),
            (Mnemonic::ASL, AddressingMode::ZeroPageX),
            (Mnemonic::SLO, AddressingMode::ZeroPageX),
            (Mnemonic::CLC, AddressingMode::Implied),
            (Mnemonic::ORA, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::SLO, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::ORA, AddressingMode::AbsoluteX),
            (Mnemonic::ASL, AddressingMode::AbsoluteX),
            (Mnemonic::SLO, AddressingMode::AbsoluteX),
            // 0x20-0x2F
            (Mnemonic::JSR, AddressingMode::Absolute),
            (Mnemonic::AND, AddressingMode::IndirectX),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::RLA, AddressingMode::IndirectX),
            (Mnemonic::BIT, AddressingMode::ZeroPage),
            (Mnemonic::AND, AddressingMode::ZeroPage),
            (Mnemonic::ROL, AddressingMode::ZeroPage),
            (Mnemonic::RLA, AddressingMode::ZeroPage),
            (Mnemonic::PLP, AddressingMode::Implied),
            (Mnemonic::AND, AddressingMode::Immediate),
            (Mnemonic::ROL, AddressingMode::Accumulator),
            (Mnemonic::ANC, AddressingMode::Immediate),
            (Mnemonic::BIT, AddressingMode::Absolute),
            (Mnemonic::AND, AddressingMode::Absolute),
            (Mnemonic::ROL, AddressingMode::Absolute),
            (Mnemonic::RLA, AddressingMode::Absolute),
            // 0x30-0x3F
            (Mnemonic::BMI, AddressingMode::Relative),
            (Mnemonic::AND, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::RLA, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::AND, AddressingMode::ZeroPageX),
            (Mnemonic::ROL, AddressingMode::ZeroPageX),
            (Mnemonic::RLA, AddressingMode::ZeroPageX),
            (Mnemonic::SEC, AddressingMode::Implied),
            (Mnemonic::AND, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::RLA, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::AND, AddressingMode::AbsoluteX),
            (Mnemonic::ROL, AddressingMode::AbsoluteX),
            (Mnemonic::RLA, AddressingMode::AbsoluteX),
            // 0x40-0x4F
            (Mnemonic::RTI, AddressingMode::Implied),
            (Mnemonic::EOR, AddressingMode::IndirectX),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::SRE, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::ZeroPage),
            (Mnemonic::EOR, AddressingMode::ZeroPage),
            (Mnemonic::LSR, AddressingMode::ZeroPage),
            (Mnemonic::SRE, AddressingMode::ZeroPage),
            (Mnemonic::PHA, AddressingMode::Implied),
            (Mnemonic::EOR, AddressingMode::Immediate),
            (Mnemonic::LSR, AddressingMode::Accumulator),
            (Mnemonic::ASR, AddressingMode::Immediate),
            (Mnemonic::JMP, AddressingMode::Absolute),
            (Mnemonic::EOR, AddressingMode::Absolute),
            (Mnemonic::LSR, AddressingMode::Absolute),
            (Mnemonic::SRE, AddressingMode::Absolute),
            // 0x50-0x5F
            (Mnemonic::BVC, AddressingMode::Relative),
            (Mnemonic::EOR, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::SRE, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::EOR, AddressingMode::ZeroPageX),
            (Mnemonic::LSR, AddressingMode::ZeroPageX),
            (Mnemonic::SRE, AddressingMode::ZeroPageX),
            (Mnemonic::CLI, AddressingMode::Implied),
            (Mnemonic::EOR, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::SRE, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::EOR, AddressingMode::AbsoluteX),
            (Mnemonic::LSR, AddressingMode::AbsoluteX),
            (Mnemonic::SRE, AddressingMode::AbsoluteX),
            // 0x60-0x6F
            (Mnemonic::RTS, AddressingMode::Implied),
            (Mnemonic::ADC, AddressingMode::IndirectX),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::RRA, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::ZeroPage),
            (Mnemonic::ADC, AddressingMode::ZeroPage),
            (Mnemonic::ROR, AddressingMode::ZeroPage),
            (Mnemonic::RRA, AddressingMode::ZeroPage),
            (Mnemonic::PLA, AddressingMode::Implied),
            (Mnemonic::ADC, AddressingMode::Immediate),
            (Mnemonic::ROR, AddressingMode::Accumulator),
            (Mnemonic::ARR, AddressingMode::Immediate),
            (Mnemonic::JMP, AddressingMode::Indirect),
            (Mnemonic::ADC, AddressingMode::Absolute),
            (Mnemonic::ROR, AddressingMode::Absolute),
            (Mnemonic::RRA, AddressingMode::Absolute),
            // 0x70-0x7F
            (Mnemonic::BVS, AddressingMode::Relative),
            (Mnemonic::ADC, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::RRA, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::ADC, AddressingMode::ZeroPageX),
            (Mnemonic::ROR, AddressingMode::ZeroPageX),
            (Mnemonic::RRA, AddressingMode::ZeroPageX),
            (Mnemonic::SEI, AddressingMode::Implied),
            (Mnemonic::ADC, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::RRA, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::ADC, AddressingMode::AbsoluteX),
            (Mnemonic::ROR, AddressingMode::AbsoluteX),
            (Mnemonic::RRA, AddressingMode::AbsoluteX),
            // 0x80-0x8F
            (Mnemonic::NOP, AddressingMode::Immediate),
            (Mnemonic::STA, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::Immediate),
            (Mnemonic::SAX, AddressingMode::IndirectX),
            (Mnemonic::STY, AddressingMode::ZeroPage),
            (Mnemonic::STA, AddressingMode::ZeroPage),
            (Mnemonic::STX, AddressingMode::ZeroPage),
            (Mnemonic::SAX, AddressingMode::ZeroPage),
            (Mnemonic::DEY, AddressingMode::Implied),
            (Mnemonic::NOP, AddressingMode::Immediate),
            (Mnemonic::TXA, AddressingMode::Implied),
            (Mnemonic::XAA, AddressingMode::Immediate),
            (Mnemonic::STY, AddressingMode::Absolute),
            (Mnemonic::STA, AddressingMode::Absolute),
            (Mnemonic::STX, AddressingMode::Absolute),
            (Mnemonic::SAX, AddressingMode::Absolute),
            // 0x90-0x9F
            (Mnemonic::BCC, AddressingMode::Relative),
            (Mnemonic::STA, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::SHA, AddressingMode::IndirectY),
            (Mnemonic::STY, AddressingMode::ZeroPageX),
            (Mnemonic::STA, AddressingMode::ZeroPageX),
            (Mnemonic::STX, AddressingMode::ZeroPageY),
            (Mnemonic::SAX, AddressingMode::ZeroPageY),
            (Mnemonic::TYA, AddressingMode::Implied),
            (Mnemonic::STA, AddressingMode::AbsoluteY),
            (Mnemonic::TXS, AddressingMode::Implied),
            (Mnemonic::SHS, AddressingMode::AbsoluteY),
            (Mnemonic::SHY, AddressingMode::AbsoluteX),
            (Mnemonic::STA, AddressingMode::AbsoluteX),
            (Mnemonic::SHX, AddressingMode::AbsoluteY),
            (Mnemonic::SHA, AddressingMode::AbsoluteY),
            // 0xA0-0xAF
            (Mnemonic::LDY, AddressingMode::Immediate),
            (Mnemonic::LDA, AddressingMode::IndirectX),
            (Mnemonic::LDX, AddressingMode::Immediate),
            (Mnemonic::LAX, AddressingMode::IndirectX),
            (Mnemonic::LDY, AddressingMode::ZeroPage),
            (Mnemonic::LDA, AddressingMode::ZeroPage),
            (Mnemonic::LDX, AddressingMode::ZeroPage),
            (Mnemonic::LAX, AddressingMode::ZeroPage),
            (Mnemonic::TAY, AddressingMode::Implied),
            (Mnemonic::LDA, AddressingMode::Immediate),
            (Mnemonic::TAX, AddressingMode::Implied),
            (Mnemonic::LAX, AddressingMode::Immediate),
            (Mnemonic::LDY, AddressingMode::Absolute),
            (Mnemonic::LDA, AddressingMode::Absolute),
            (Mnemonic::LDX, AddressingMode::Absolute),
            (Mnemonic::LAX, AddressingMode::Absolute),
            // 0xB0-0xBF
            (Mnemonic::BCS, AddressingMode::Relative),
            (Mnemonic::LDA, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::LAX, AddressingMode::IndirectY),
            (Mnemonic::LDY, AddressingMode::ZeroPageX),
            (Mnemonic::LDA, AddressingMode::ZeroPageX),
            (Mnemonic::LDX, AddressingMode::ZeroPageY),
            (Mnemonic::LAX, AddressingMode::ZeroPageY),
            (Mnemonic::CLV, AddressingMode::Implied),
            (Mnemonic::LDA, AddressingMode::AbsoluteY),
            (Mnemonic::TSX, AddressingMode::Implied),
            (Mnemonic::LAS, AddressingMode::AbsoluteY),
            (Mnemonic::LDY, AddressingMode::AbsoluteX),
            (Mnemonic::LDA, AddressingMode::AbsoluteX),
            (Mnemonic::LDX, AddressingMode::AbsoluteY),
            (Mnemonic::LAX, AddressingMode::AbsoluteY),
            // 0xC0-0xCF
            (Mnemonic::CPY, AddressingMode::Immediate),
            (Mnemonic::CMP, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::Immediate),
            (Mnemonic::DCP, AddressingMode::IndirectX),
            (Mnemonic::CPY, AddressingMode::ZeroPage),
            (Mnemonic::CMP, AddressingMode::ZeroPage),
            (Mnemonic::DEC, AddressingMode::ZeroPage),
            (Mnemonic::DCP, AddressingMode::ZeroPage),
            (Mnemonic::INY, AddressingMode::Implied),
            (Mnemonic::CMP, AddressingMode::Immediate),
            (Mnemonic::DEX, AddressingMode::Implied),
            (Mnemonic::SBX, AddressingMode::Immediate),
            (Mnemonic::CPY, AddressingMode::Absolute),
            (Mnemonic::CMP, AddressingMode::Absolute),
            (Mnemonic::DEC, AddressingMode::Absolute),
            (Mnemonic::DCP, AddressingMode::Absolute),
            // 0xD0-0xDF
            (Mnemonic::BNE, AddressingMode::Relative),
            (Mnemonic::CMP, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::DCP, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::CMP, AddressingMode::ZeroPageX),
            (Mnemonic::DEC, AddressingMode::ZeroPageX),
            (Mnemonic::DCP, AddressingMode::ZeroPageX),
            (Mnemonic::CLD, AddressingMode::Implied),
            (Mnemonic::CMP, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::DCP, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::CMP, AddressingMode::AbsoluteX),
            (Mnemonic::DEC, AddressingMode::AbsoluteX),
            (Mnemonic::DCP, AddressingMode::AbsoluteX),
            // 0xE0-0xEF
            (Mnemonic::CPX, AddressingMode::Immediate),
            (Mnemonic::SBC, AddressingMode::IndirectX),
            (Mnemonic::NOP, AddressingMode::Immediate),
            (Mnemonic::ISC, AddressingMode::IndirectX),
            (Mnemonic::CPX, AddressingMode::ZeroPage),
            (Mnemonic::SBC, AddressingMode::ZeroPage),
            (Mnemonic::INC, AddressingMode::ZeroPage),
            (Mnemonic::ISC, AddressingMode::ZeroPage),
            (Mnemonic::INX, AddressingMode::Implied),
            (Mnemonic::SBC, AddressingMode::Immediate),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::SBC, AddressingMode::Immediate),
            (Mnemonic::CPX, AddressingMode::Absolute),
            (Mnemonic::SBC, AddressingMode::Absolute),
            (Mnemonic::INC, AddressingMode::Absolute),
            (Mnemonic::ISC, AddressingMode::Absolute),
            // 0xF0-0xFF
            (Mnemonic::BEQ, AddressingMode::Relative),
            (Mnemonic::SBC, AddressingMode::IndirectY),
            (Mnemonic::JAM, AddressingMode::Implied),
            (Mnemonic::ISC, AddressingMode::IndirectY),
            (Mnemonic::NOP, AddressingMode::ZeroPageX),
            (Mnemonic::SBC, AddressingMode::ZeroPageX),
            (Mnemonic::INC, AddressingMode::ZeroPageX),
            (Mnemonic::ISC, AddressingMode::ZeroPageX),
            (Mnemonic::SED, AddressingMode::Implied),
            (Mnemonic::SBC, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::Implied),
            (Mnemonic::ISC, AddressingMode::AbsoluteY),
            (Mnemonic::NOP, AddressingMode::AbsoluteX),
            (Mnemonic::SBC, AddressingMode::AbsoluteX),
            (Mnemonic::INC, AddressingMode::AbsoluteX),
            (Mnemonic::ISC, AddressingMode::AbsoluteX),
        ]
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mnemonic::LAS => "las".fmt(f),
            Mnemonic::LAX => "lax".fmt(f),
            Mnemonic::LDA => "lda".fmt(f),
            Mnemonic::LDX => "ldx".fmt(f),
            Mnemonic::LDY => "ldy".fmt(f),
            Mnemonic::SAX => "sax".fmt(f),
            Mnemonic::SHA => "sha".fmt(f),
            Mnemonic::SHX => "shx".fmt(f),
            Mnemonic::SHY => "shy".fmt(f),
            Mnemonic::STA => "sta".fmt(f),
            Mnemonic::STX => "stx".fmt(f),
            Mnemonic::STY => "sty".fmt(f),
            Mnemonic::SHS => "shs".fmt(f),
            Mnemonic::TAX => "tax".fmt(f),
            Mnemonic::TAY => "tay".fmt(f),
            Mnemonic::TSX => "tsx".fmt(f),
            Mnemonic::TXA => "txa".fmt(f),
            Mnemonic::TXS => "txs".fmt(f),
            Mnemonic::TYA => "tya".fmt(f),
            Mnemonic::PHA => "pha".fmt(f),
            Mnemonic::PHP => "php".fmt(f),
            Mnemonic::PLA => "pla".fmt(f),
            Mnemonic::PLP => "plp".fmt(f),
            Mnemonic::ASL => "asl".fmt(f),
            Mnemonic::LSR => "lsr".fmt(f),
            Mnemonic::ROL => "rol".fmt(f),
            Mnemonic::ROR => "ror".fmt(f),
            Mnemonic::AND => "and".fmt(f),
            Mnemonic::BIT => "bit".fmt(f),
            Mnemonic::EOR => "eor".fmt(f),
            Mnemonic::ORA => "ora".fmt(f),
            Mnemonic::ADC => "adc".fmt(f),
            Mnemonic::ANC => "anc".fmt(f),
            Mnemonic::ARR => "arr".fmt(f),
            Mnemonic::ASR => "asr".fmt(f),
            Mnemonic::CMP => "cmp".fmt(f),
            Mnemonic::CPX => "cpx".fmt(f),
            Mnemonic::CPY => "cpy".fmt(f),
            Mnemonic::DCP => "dcp".fmt(f),
            Mnemonic::ISC => "isc".fmt(f),
            Mnemonic::RLA => "rla".fmt(f),
            Mnemonic::RRA => "rra".fmt(f),
            Mnemonic::SBC => "sbc".fmt(f),
            Mnemonic::SBX => "sbx".fmt(f),
            Mnemonic::SLO => "slo".fmt(f),
            Mnemonic::SRE => "sre".fmt(f),
            Mnemonic::XAA => "xaa".fmt(f),
            Mnemonic::DEC => "dec".fmt(f),
            Mnemonic::DEX => "dex".fmt(f),
            Mnemonic::DEY => "dey".fmt(f),
            Mnemonic::INC => "inc".fmt(f),
            Mnemonic::INX => "inx".fmt(f),
            Mnemonic::INY => "iny".fmt(f),
            Mnemonic::BRK => "brk".fmt(f),
            Mnemonic::JMP => "jmp".fmt(f),
            Mnemonic::JSR => "jsr".fmt(f),
            Mnemonic::RTI => "rti".fmt(f),
            Mnemonic::RTS => "rts".fmt(f),
            Mnemonic::BCC => "bcc".fmt(f),
            Mnemonic::BCS => "bcs".fmt(f),
            Mnemonic::BEQ => "beq".fmt(f),
            Mnemonic::BMI => "bmi".fmt(f),
            Mnemonic::BNE => "bne".fmt(f),
            Mnemonic::BPL => "bpl".fmt(f),
            Mnemonic::BVC => "bvc".fmt(f),
            Mnemonic::BVS => "bvs".fmt(f),
            Mnemonic::CLC => "clc".fmt(f),
            Mnemonic::CLD => "cld".fmt(f),
            Mnemonic::CLI => "cli".fmt(f),
            Mnemonic::CLV => "clv".fmt(f),
            Mnemonic::SEC => "sec".fmt(f),
            Mnemonic::SED => "sed".fmt(f),
            Mnemonic::SEI => "sei".fmt(f),
            Mnemonic::JAM => "jam".fmt(f),
            Mnemonic::NOP => "nop".fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Instruction {
    pub(crate) opcode: Mnemonic,
    pub(crate) addressing: AddressingMode,
    pub(crate) micro_ops: &'static [MicroOp],
}

impl Instruction {
    pub(crate) const fn ldx(addr: AddressingMode) -> Self {
        Self {
            opcode: Mnemonic::LDX,
            addressing: addr,
            micro_ops: &[],
        }
    }
}
