use std::fmt::Display;

use crate::{
    bus::Bus,
    cpu::{addressing::Addressing, micro_op::MicroOp, status::Status},
};

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
    pub(crate) const fn table() -> &'static [(Mnemonic, Addressing); 256] {
        &[
            // 0x00-0x0F
            (Mnemonic::BRK, Addressing::Implied),
            (Mnemonic::ORA, Addressing::IndirectX),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SLO, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::ZeroPage),
            (Mnemonic::ORA, Addressing::ZeroPage),
            (Mnemonic::ASL, Addressing::ZeroPage),
            (Mnemonic::SLO, Addressing::ZeroPage),
            (Mnemonic::PHP, Addressing::Implied),
            (Mnemonic::ORA, Addressing::Immediate),
            (Mnemonic::ASL, Addressing::Accumulator),
            (Mnemonic::ANC, Addressing::Immediate),
            (Mnemonic::NOP, Addressing::Absolute),
            (Mnemonic::ORA, Addressing::Absolute),
            (Mnemonic::ASL, Addressing::Absolute),
            (Mnemonic::SLO, Addressing::Absolute),
            // 0x10-0x1F
            (Mnemonic::BPL, Addressing::Relative),
            (Mnemonic::ORA, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SLO, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::ORA, Addressing::ZeroPageX),
            (Mnemonic::ASL, Addressing::ZeroPageX),
            (Mnemonic::SLO, Addressing::ZeroPageX),
            (Mnemonic::CLC, Addressing::Implied),
            (Mnemonic::ORA, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::SLO, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::ORA, Addressing::AbsoluteX),
            (Mnemonic::ASL, Addressing::AbsoluteX),
            (Mnemonic::SLO, Addressing::AbsoluteX),
            // 0x20-0x2F
            (Mnemonic::JSR, Addressing::Absolute),
            (Mnemonic::AND, Addressing::IndirectX),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RLA, Addressing::IndirectX),
            (Mnemonic::BIT, Addressing::ZeroPage),
            (Mnemonic::AND, Addressing::ZeroPage),
            (Mnemonic::ROL, Addressing::ZeroPage),
            (Mnemonic::RLA, Addressing::ZeroPage),
            (Mnemonic::PLP, Addressing::Implied),
            (Mnemonic::AND, Addressing::Immediate),
            (Mnemonic::ROL, Addressing::Accumulator),
            (Mnemonic::ANC, Addressing::Immediate),
            (Mnemonic::BIT, Addressing::Absolute),
            (Mnemonic::AND, Addressing::Absolute),
            (Mnemonic::ROL, Addressing::Absolute),
            (Mnemonic::RLA, Addressing::Absolute),
            // 0x30-0x3F
            (Mnemonic::BMI, Addressing::Relative),
            (Mnemonic::AND, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RLA, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::AND, Addressing::ZeroPageX),
            (Mnemonic::ROL, Addressing::ZeroPageX),
            (Mnemonic::RLA, Addressing::ZeroPageX),
            (Mnemonic::SEC, Addressing::Implied),
            (Mnemonic::AND, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::RLA, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::AND, Addressing::AbsoluteX),
            (Mnemonic::ROL, Addressing::AbsoluteX),
            (Mnemonic::RLA, Addressing::AbsoluteX),
            // 0x40-0x4F
            (Mnemonic::RTI, Addressing::Implied),
            (Mnemonic::EOR, Addressing::IndirectX),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SRE, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::ZeroPage),
            (Mnemonic::EOR, Addressing::ZeroPage),
            (Mnemonic::LSR, Addressing::ZeroPage),
            (Mnemonic::SRE, Addressing::ZeroPage),
            (Mnemonic::PHA, Addressing::Implied),
            (Mnemonic::EOR, Addressing::Immediate),
            (Mnemonic::LSR, Addressing::Accumulator),
            (Mnemonic::ASR, Addressing::Immediate),
            (Mnemonic::JMP, Addressing::Absolute),
            (Mnemonic::EOR, Addressing::Absolute),
            (Mnemonic::LSR, Addressing::Absolute),
            (Mnemonic::SRE, Addressing::Absolute),
            // 0x50-0x5F
            (Mnemonic::BVC, Addressing::Relative),
            (Mnemonic::EOR, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SRE, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::EOR, Addressing::ZeroPageX),
            (Mnemonic::LSR, Addressing::ZeroPageX),
            (Mnemonic::SRE, Addressing::ZeroPageX),
            (Mnemonic::CLI, Addressing::Implied),
            (Mnemonic::EOR, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::SRE, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::EOR, Addressing::AbsoluteX),
            (Mnemonic::LSR, Addressing::AbsoluteX),
            (Mnemonic::SRE, Addressing::AbsoluteX),
            // 0x60-0x6F
            (Mnemonic::RTS, Addressing::Implied),
            (Mnemonic::ADC, Addressing::IndirectX),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RRA, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::ZeroPage),
            (Mnemonic::ADC, Addressing::ZeroPage),
            (Mnemonic::ROR, Addressing::ZeroPage),
            (Mnemonic::RRA, Addressing::ZeroPage),
            (Mnemonic::PLA, Addressing::Implied),
            (Mnemonic::ADC, Addressing::Immediate),
            (Mnemonic::ROR, Addressing::Accumulator),
            (Mnemonic::ARR, Addressing::Immediate),
            (Mnemonic::JMP, Addressing::Indirect),
            (Mnemonic::ADC, Addressing::Absolute),
            (Mnemonic::ROR, Addressing::Absolute),
            (Mnemonic::RRA, Addressing::Absolute),
            // 0x70-0x7F
            (Mnemonic::BVS, Addressing::Relative),
            (Mnemonic::ADC, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RRA, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::ADC, Addressing::ZeroPageX),
            (Mnemonic::ROR, Addressing::ZeroPageX),
            (Mnemonic::RRA, Addressing::ZeroPageX),
            (Mnemonic::SEI, Addressing::Implied),
            (Mnemonic::ADC, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::RRA, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::ADC, Addressing::AbsoluteX),
            (Mnemonic::ROR, Addressing::AbsoluteX),
            (Mnemonic::RRA, Addressing::AbsoluteX),
            // 0x80-0x8F
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::STA, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::SAX, Addressing::IndirectX),
            (Mnemonic::STY, Addressing::ZeroPage),
            (Mnemonic::STA, Addressing::ZeroPage),
            (Mnemonic::STX, Addressing::ZeroPage),
            (Mnemonic::SAX, Addressing::ZeroPage),
            (Mnemonic::DEY, Addressing::Implied),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::TXA, Addressing::Implied),
            (Mnemonic::XAA, Addressing::Immediate),
            (Mnemonic::STY, Addressing::Absolute),
            (Mnemonic::STA, Addressing::Absolute),
            (Mnemonic::STX, Addressing::Absolute),
            (Mnemonic::SAX, Addressing::Absolute),
            // 0x90-0x9F
            (Mnemonic::BCC, Addressing::Relative),
            (Mnemonic::STA, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SHA, Addressing::IndirectY),
            (Mnemonic::STY, Addressing::ZeroPageX),
            (Mnemonic::STA, Addressing::ZeroPageX),
            (Mnemonic::STX, Addressing::ZeroPageY),
            (Mnemonic::SAX, Addressing::ZeroPageY),
            (Mnemonic::TYA, Addressing::Implied),
            (Mnemonic::STA, Addressing::AbsoluteY),
            (Mnemonic::TXS, Addressing::Implied),
            (Mnemonic::SHS, Addressing::AbsoluteY),
            (Mnemonic::SHY, Addressing::AbsoluteX),
            (Mnemonic::STA, Addressing::AbsoluteX),
            (Mnemonic::SHX, Addressing::AbsoluteY),
            (Mnemonic::SHA, Addressing::AbsoluteY),
            // 0xA0-0xAF
            (Mnemonic::LDY, Addressing::Immediate),
            (Mnemonic::LDA, Addressing::IndirectX),
            (Mnemonic::LDX, Addressing::Immediate),
            (Mnemonic::LAX, Addressing::IndirectX),
            (Mnemonic::LDY, Addressing::ZeroPage),
            (Mnemonic::LDA, Addressing::ZeroPage),
            (Mnemonic::LDX, Addressing::ZeroPage),
            (Mnemonic::LAX, Addressing::ZeroPage),
            (Mnemonic::TAY, Addressing::Implied),
            (Mnemonic::LDA, Addressing::Immediate),
            (Mnemonic::TAX, Addressing::Implied),
            (Mnemonic::LAX, Addressing::Immediate),
            (Mnemonic::LDY, Addressing::Absolute),
            (Mnemonic::LDA, Addressing::Absolute),
            (Mnemonic::LDX, Addressing::Absolute),
            (Mnemonic::LAX, Addressing::Absolute),
            // 0xB0-0xBF
            (Mnemonic::BCS, Addressing::Relative),
            (Mnemonic::LDA, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::LAX, Addressing::IndirectY),
            (Mnemonic::LDY, Addressing::ZeroPageX),
            (Mnemonic::LDA, Addressing::ZeroPageX),
            (Mnemonic::LDX, Addressing::ZeroPageY),
            (Mnemonic::LAX, Addressing::ZeroPageY),
            (Mnemonic::CLV, Addressing::Implied),
            (Mnemonic::LDA, Addressing::AbsoluteY),
            (Mnemonic::TSX, Addressing::Implied),
            (Mnemonic::LAS, Addressing::AbsoluteY),
            (Mnemonic::LDY, Addressing::AbsoluteX),
            (Mnemonic::LDA, Addressing::AbsoluteX),
            (Mnemonic::LDX, Addressing::AbsoluteY),
            (Mnemonic::LAX, Addressing::AbsoluteY),
            // 0xC0-0xCF
            (Mnemonic::CPY, Addressing::Immediate),
            (Mnemonic::CMP, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::DCP, Addressing::IndirectX),
            (Mnemonic::CPY, Addressing::ZeroPage),
            (Mnemonic::CMP, Addressing::ZeroPage),
            (Mnemonic::DEC, Addressing::ZeroPage),
            (Mnemonic::DCP, Addressing::ZeroPage),
            (Mnemonic::INY, Addressing::Implied),
            (Mnemonic::CMP, Addressing::Immediate),
            (Mnemonic::DEX, Addressing::Implied),
            (Mnemonic::SBX, Addressing::Immediate),
            (Mnemonic::CPY, Addressing::Absolute),
            (Mnemonic::CMP, Addressing::Absolute),
            (Mnemonic::DEC, Addressing::Absolute),
            (Mnemonic::DCP, Addressing::Absolute),
            // 0xD0-0xDF
            (Mnemonic::BNE, Addressing::Relative),
            (Mnemonic::CMP, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::DCP, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::CMP, Addressing::ZeroPageX),
            (Mnemonic::DEC, Addressing::ZeroPageX),
            (Mnemonic::DCP, Addressing::ZeroPageX),
            (Mnemonic::CLD, Addressing::Implied),
            (Mnemonic::CMP, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::DCP, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::CMP, Addressing::AbsoluteX),
            (Mnemonic::DEC, Addressing::AbsoluteX),
            (Mnemonic::DCP, Addressing::AbsoluteX),
            // 0xE0-0xEF
            (Mnemonic::CPX, Addressing::Immediate),
            (Mnemonic::SBC, Addressing::IndirectX),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::ISC, Addressing::IndirectX),
            (Mnemonic::CPX, Addressing::ZeroPage),
            (Mnemonic::SBC, Addressing::ZeroPage),
            (Mnemonic::INC, Addressing::ZeroPage),
            (Mnemonic::ISC, Addressing::ZeroPage),
            (Mnemonic::INX, Addressing::Implied),
            (Mnemonic::SBC, Addressing::Immediate),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::SBC, Addressing::Immediate),
            (Mnemonic::CPX, Addressing::Absolute),
            (Mnemonic::SBC, Addressing::Absolute),
            (Mnemonic::INC, Addressing::Absolute),
            (Mnemonic::ISC, Addressing::Absolute),
            // 0xF0-0xFF
            (Mnemonic::BEQ, Addressing::Relative),
            (Mnemonic::SBC, Addressing::IndirectY),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::ISC, Addressing::IndirectY),
            (Mnemonic::NOP, Addressing::ZeroPageX),
            (Mnemonic::SBC, Addressing::ZeroPageX),
            (Mnemonic::INC, Addressing::ZeroPageX),
            (Mnemonic::ISC, Addressing::ZeroPageX),
            (Mnemonic::SED, Addressing::Implied),
            (Mnemonic::SBC, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::ISC, Addressing::AbsoluteY),
            (Mnemonic::NOP, Addressing::AbsoluteX),
            (Mnemonic::SBC, Addressing::AbsoluteX),
            (Mnemonic::INC, Addressing::AbsoluteX),
            (Mnemonic::ISC, Addressing::AbsoluteX),
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
    pub(crate) addressing: Addressing,
    pub(crate) micro_ops: &'static [MicroOp],
}

impl Instruction {
    pub(crate) const fn ldx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LDX,
            addressing: addr,
            micro_ops: &[],
        }
    }
}
