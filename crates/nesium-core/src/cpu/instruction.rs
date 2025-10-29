use std::fmt::Display;

use crate::cpu::{addressing::Addressing, micro_op::MicroOp, status::Status};

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

    pub(crate) const fn table() -> &'static [(Mnemonic, Addressing); 256] {
        &[
            // 0x00-0x0F
            (Mnemonic::BRK, Addressing::Implied),
            (Mnemonic::ORA, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SLO, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::ORA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SLO, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::ORA, Addressing::XIndexedZeroPage),
            (Mnemonic::ASL, Addressing::XIndexedZeroPage),
            (Mnemonic::SLO, Addressing::XIndexedZeroPage),
            (Mnemonic::CLC, Addressing::Implied),
            (Mnemonic::ORA, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::SLO, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::ORA, Addressing::XIndexedAbsolute),
            (Mnemonic::ASL, Addressing::XIndexedAbsolute),
            (Mnemonic::SLO, Addressing::XIndexedAbsolute),
            // 0x20-0x2F
            (Mnemonic::JSR, Addressing::Absolute),
            (Mnemonic::AND, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RLA, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::AND, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RLA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::AND, Addressing::XIndexedZeroPage),
            (Mnemonic::ROL, Addressing::XIndexedZeroPage),
            (Mnemonic::RLA, Addressing::XIndexedZeroPage),
            (Mnemonic::SEC, Addressing::Implied),
            (Mnemonic::AND, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::RLA, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::AND, Addressing::XIndexedAbsolute),
            (Mnemonic::ROL, Addressing::XIndexedAbsolute),
            (Mnemonic::RLA, Addressing::XIndexedAbsolute),
            // 0x40-0x4F
            (Mnemonic::RTI, Addressing::Implied),
            (Mnemonic::EOR, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SRE, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::EOR, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SRE, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::EOR, Addressing::XIndexedZeroPage),
            (Mnemonic::LSR, Addressing::XIndexedZeroPage),
            (Mnemonic::SRE, Addressing::XIndexedZeroPage),
            (Mnemonic::CLI, Addressing::Implied),
            (Mnemonic::EOR, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::SRE, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::EOR, Addressing::XIndexedAbsolute),
            (Mnemonic::LSR, Addressing::XIndexedAbsolute),
            (Mnemonic::SRE, Addressing::XIndexedAbsolute),
            // 0x60-0x6F
            (Mnemonic::RTS, Addressing::Implied),
            (Mnemonic::ADC, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RRA, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::NOP, Addressing::ZeroPage),
            (Mnemonic::ADC, Addressing::ZeroPage),
            (Mnemonic::ROR, Addressing::ZeroPage),
            (Mnemonic::RRA, Addressing::ZeroPage),
            (Mnemonic::PLA, Addressing::Implied),
            (Mnemonic::ADC, Addressing::Immediate),
            (Mnemonic::ROR, Addressing::Accumulator),
            (Mnemonic::ARR, Addressing::Immediate),
            (Mnemonic::JMP, Addressing::AbsoluteIndirect),
            (Mnemonic::ADC, Addressing::Absolute),
            (Mnemonic::ROR, Addressing::Absolute),
            (Mnemonic::RRA, Addressing::Absolute),
            // 0x70-0x7F
            (Mnemonic::BVS, Addressing::Relative),
            (Mnemonic::ADC, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::RRA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::ADC, Addressing::XIndexedZeroPage),
            (Mnemonic::ROR, Addressing::XIndexedZeroPage),
            (Mnemonic::RRA, Addressing::XIndexedZeroPage),
            (Mnemonic::SEI, Addressing::Implied),
            (Mnemonic::ADC, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::RRA, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::ADC, Addressing::XIndexedAbsolute),
            (Mnemonic::ROR, Addressing::XIndexedAbsolute),
            (Mnemonic::RRA, Addressing::XIndexedAbsolute),
            // 0x80-0x8F
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::STA, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::SAX, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::STA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::SHA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::STY, Addressing::XIndexedZeroPage),
            (Mnemonic::STA, Addressing::XIndexedZeroPage),
            (Mnemonic::STX, Addressing::YIndexedZeroPage),
            (Mnemonic::SAX, Addressing::YIndexedZeroPage),
            (Mnemonic::TYA, Addressing::Implied),
            (Mnemonic::STA, Addressing::YIndexedAbsolute),
            (Mnemonic::TXS, Addressing::Implied),
            (Mnemonic::SHS, Addressing::YIndexedAbsolute),
            (Mnemonic::SHY, Addressing::XIndexedAbsolute),
            (Mnemonic::STA, Addressing::XIndexedAbsolute),
            (Mnemonic::SHX, Addressing::YIndexedAbsolute),
            (Mnemonic::SHA, Addressing::YIndexedAbsolute),
            // 0xA0-0xAF
            (Mnemonic::LDY, Addressing::Immediate),
            (Mnemonic::LDA, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::LDX, Addressing::Immediate),
            (Mnemonic::LAX, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::LDA, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::LAX, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::LDY, Addressing::XIndexedZeroPage),
            (Mnemonic::LDA, Addressing::XIndexedZeroPage),
            (Mnemonic::LDX, Addressing::YIndexedZeroPage),
            (Mnemonic::LAX, Addressing::YIndexedZeroPage),
            (Mnemonic::CLV, Addressing::Implied),
            (Mnemonic::LDA, Addressing::YIndexedAbsolute),
            (Mnemonic::TSX, Addressing::Implied),
            (Mnemonic::LAS, Addressing::YIndexedAbsolute),
            (Mnemonic::LDY, Addressing::XIndexedAbsolute),
            (Mnemonic::LDA, Addressing::XIndexedAbsolute),
            (Mnemonic::LDX, Addressing::YIndexedAbsolute),
            (Mnemonic::LAX, Addressing::YIndexedAbsolute),
            // 0xC0-0xCF
            (Mnemonic::CPY, Addressing::Immediate),
            (Mnemonic::CMP, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::DCP, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::CMP, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::DCP, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::CMP, Addressing::XIndexedZeroPage),
            (Mnemonic::DEC, Addressing::XIndexedZeroPage),
            (Mnemonic::DCP, Addressing::XIndexedZeroPage),
            (Mnemonic::CLD, Addressing::Implied),
            (Mnemonic::CMP, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::DCP, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::CMP, Addressing::XIndexedAbsolute),
            (Mnemonic::DEC, Addressing::XIndexedAbsolute),
            (Mnemonic::DCP, Addressing::XIndexedAbsolute),
            // 0xE0-0xEF
            (Mnemonic::CPX, Addressing::Immediate),
            (Mnemonic::SBC, Addressing::XIndexedZeroPageIndirect),
            (Mnemonic::NOP, Addressing::Immediate),
            (Mnemonic::ISC, Addressing::XIndexedZeroPageIndirect),
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
            (Mnemonic::SBC, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::JAM, Addressing::Implied),
            (Mnemonic::ISC, Addressing::ZeroPageIndirectYIndexed),
            (Mnemonic::NOP, Addressing::XIndexedZeroPage),
            (Mnemonic::SBC, Addressing::XIndexedZeroPage),
            (Mnemonic::INC, Addressing::XIndexedZeroPage),
            (Mnemonic::ISC, Addressing::XIndexedZeroPage),
            (Mnemonic::SED, Addressing::Implied),
            (Mnemonic::SBC, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::Implied),
            (Mnemonic::ISC, Addressing::YIndexedAbsolute),
            (Mnemonic::NOP, Addressing::XIndexedAbsolute),
            (Mnemonic::SBC, Addressing::XIndexedAbsolute),
            (Mnemonic::INC, Addressing::XIndexedAbsolute),
            (Mnemonic::ISC, Addressing::XIndexedAbsolute),
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
