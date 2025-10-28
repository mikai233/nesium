use std::fmt::Display;

use crate::cpu::{addressing::Addressing, micro_op::MicroOp, status::Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Instruction {
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

impl Instruction {
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
            Instruction::LAS
            | Instruction::LAX
            | Instruction::LDA
            | Instruction::LDX
            | Instruction::LDY => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::SAX
            | Instruction::SHA
            | Instruction::SHX
            | Instruction::SHY
            | Instruction::STA
            | Instruction::STX
            | Instruction::STY => {}
            //Transfer
            Instruction::SHS => {}
            Instruction::TAX | Instruction::TAY | Instruction::TSX | Instruction::TXA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::TXS => {}
            Instruction::TYA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Stack
            Instruction::PHA | Instruction::PHP => {}
            Instruction::PLA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::PLP => {
                // Restore all flags from stack value
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            //Shift
            Instruction::ASL => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::LSR => {
                status!(status, result, carry, overflow; N:0, Z:*, C:*);
            }
            Instruction::ROL | Self::ROR => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::AND => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::BIT => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*);
            }
            Instruction::EOR | Instruction::ORA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Arithmetic
            Instruction::ADC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Instruction::ANC => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::ARR => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Instruction::ASR => {
                status!(status, result, carry, overflow; N:0, Z:*, C:*);
            }
            Instruction::CMP | Instruction::CPX | Instruction::CPY | Instruction::DCP => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::ISC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Instruction::RLA => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::RRA | Instruction::SBC => {
                status!(status, result, carry, overflow; N:*, V:*, Z:*, C:*);
            }
            Instruction::SBX | Instruction::SLO | Instruction::SRE => {
                status!(status, result, carry, overflow; N:*, Z:*, C:*);
            }
            Instruction::XAA
            | Instruction::DEC
            | Instruction::DEX
            | Instruction::DEY
            | Instruction::INC
            | Instruction::INX
            | Instruction::INY => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //Control Flow
            Instruction::BRK => {
                status!(status, result, carry, overflow; I:1);
            }
            Instruction::JMP | Instruction::JSR => {}
            Instruction::RTI => {
                //TODO
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            Instruction::RTS
            | Instruction::BCC
            | Instruction::BCS
            | Instruction::BEQ
            | Instruction::BMI
            | Instruction::BNE
            | Instruction::BPL
            | Instruction::BVC
            | Instruction::BVS => {}
            //Flags
            Instruction::CLC => {
                status!(status, result, carry, overflow; C:0);
            }
            Instruction::CLD => {
                status!(status, result, carry, overflow; D:0);
            }
            Instruction::CLI => {
                status!(status, result, carry, overflow; I:0);
            }
            Instruction::CLV => {
                status!(status, result, carry, overflow; V:0);
            }
            Instruction::SEC => {
                status!(status, result, carry, overflow; C:1);
            }
            Instruction::SED => {
                status!(status, result, carry, overflow; D:1);
            }
            Instruction::SEI => {
                status!(status, result, carry, overflow; I:1);
            }
            Instruction::JAM | Instruction::NOP => {}
        }
    }

    pub(crate) const fn micro_ops(&self) -> &'static [MicroOp] {
        &[]
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::LAS => "las".fmt(f),
            Instruction::LAX => "lax".fmt(f),
            Instruction::LDA => "lda".fmt(f),
            Instruction::LDX => "ldx".fmt(f),
            Instruction::LDY => "ldy".fmt(f),
            Instruction::SAX => "sax".fmt(f),
            Instruction::SHA => "sha".fmt(f),
            Instruction::SHX => "shx".fmt(f),
            Instruction::SHY => "shy".fmt(f),
            Instruction::STA => "sta".fmt(f),
            Instruction::STX => "stx".fmt(f),
            Instruction::STY => "sty".fmt(f),
            Instruction::SHS => "shs".fmt(f),
            Instruction::TAX => "tax".fmt(f),
            Instruction::TAY => "tay".fmt(f),
            Instruction::TSX => "tsx".fmt(f),
            Instruction::TXA => "txa".fmt(f),
            Instruction::TXS => "txs".fmt(f),
            Instruction::TYA => "tya".fmt(f),
            Instruction::PHA => "pha".fmt(f),
            Instruction::PHP => "php".fmt(f),
            Instruction::PLA => "pla".fmt(f),
            Instruction::PLP => "plp".fmt(f),
            Instruction::ASL => "asl".fmt(f),
            Instruction::LSR => "lsr".fmt(f),
            Instruction::ROL => "rol".fmt(f),
            Instruction::ROR => "ror".fmt(f),
            Instruction::AND => "and".fmt(f),
            Instruction::BIT => "bit".fmt(f),
            Instruction::EOR => "eor".fmt(f),
            Instruction::ORA => "ora".fmt(f),
            Instruction::ADC => "adc".fmt(f),
            Instruction::ANC => "anc".fmt(f),
            Instruction::ARR => "arr".fmt(f),
            Instruction::ASR => "asr".fmt(f),
            Instruction::CMP => "cmp".fmt(f),
            Instruction::CPX => "cpx".fmt(f),
            Instruction::CPY => "cpy".fmt(f),
            Instruction::DCP => "dcp".fmt(f),
            Instruction::ISC => "isc".fmt(f),
            Instruction::RLA => "rla".fmt(f),
            Instruction::RRA => "rra".fmt(f),
            Instruction::SBC => "sbc".fmt(f),
            Instruction::SBX => "sbx".fmt(f),
            Instruction::SLO => "slo".fmt(f),
            Instruction::SRE => "sre".fmt(f),
            Instruction::XAA => "xaa".fmt(f),
            Instruction::DEC => "dec".fmt(f),
            Instruction::DEX => "dex".fmt(f),
            Instruction::DEY => "dey".fmt(f),
            Instruction::INC => "inc".fmt(f),
            Instruction::INX => "inx".fmt(f),
            Instruction::INY => "iny".fmt(f),
            Instruction::BRK => "brk".fmt(f),
            Instruction::JMP => "jmp".fmt(f),
            Instruction::JSR => "jsr".fmt(f),
            Instruction::RTI => "rti".fmt(f),
            Instruction::RTS => "rts".fmt(f),
            Instruction::BCC => "bcc".fmt(f),
            Instruction::BCS => "bcs".fmt(f),
            Instruction::BEQ => "beq".fmt(f),
            Instruction::BMI => "bmi".fmt(f),
            Instruction::BNE => "bne".fmt(f),
            Instruction::BPL => "bpl".fmt(f),
            Instruction::BVC => "bvc".fmt(f),
            Instruction::BVS => "bvs".fmt(f),
            Instruction::CLC => "clc".fmt(f),
            Instruction::CLD => "cld".fmt(f),
            Instruction::CLI => "cli".fmt(f),
            Instruction::CLV => "clv".fmt(f),
            Instruction::SEC => "sec".fmt(f),
            Instruction::SED => "sed".fmt(f),
            Instruction::SEI => "sei".fmt(f),
            Instruction::JAM => "jam".fmt(f),
            Instruction::NOP => "nop".fmt(f),
        }
    }
}

pub(crate) const fn table() -> &'static [(Instruction, Addressing); 256] {
    &[
        // 0x00-0x0F
        (Instruction::BRK, Addressing::Implied),
        (Instruction::ORA, Addressing::XIndexedZeroPageIndirect),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::SLO, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::ZeroPage),
        (Instruction::ORA, Addressing::ZeroPage),
        (Instruction::ASL, Addressing::ZeroPage),
        (Instruction::SLO, Addressing::ZeroPage),
        (Instruction::PHP, Addressing::Implied),
        (Instruction::ORA, Addressing::Immediate),
        (Instruction::ASL, Addressing::Accumulator),
        (Instruction::ANC, Addressing::Immediate),
        (Instruction::NOP, Addressing::Absolute),
        (Instruction::ORA, Addressing::Absolute),
        (Instruction::ASL, Addressing::Absolute),
        (Instruction::SLO, Addressing::Absolute),
        // 0x10-0x1F
        (Instruction::BPL, Addressing::Relative),
        (Instruction::ORA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::SLO, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::ORA, Addressing::XIndexedZeroPage),
        (Instruction::ASL, Addressing::XIndexedZeroPage),
        (Instruction::SLO, Addressing::XIndexedZeroPage),
        (Instruction::CLC, Addressing::Implied),
        (Instruction::ORA, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::SLO, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::ORA, Addressing::XIndexedAbsolute),
        (Instruction::ASL, Addressing::XIndexedAbsolute),
        (Instruction::SLO, Addressing::XIndexedAbsolute),
        // 0x20-0x2F
        (Instruction::JSR, Addressing::Absolute),
        (Instruction::AND, Addressing::XIndexedZeroPageIndirect),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::RLA, Addressing::XIndexedZeroPageIndirect),
        (Instruction::BIT, Addressing::ZeroPage),
        (Instruction::AND, Addressing::ZeroPage),
        (Instruction::ROL, Addressing::ZeroPage),
        (Instruction::RLA, Addressing::ZeroPage),
        (Instruction::PLP, Addressing::Implied),
        (Instruction::AND, Addressing::Immediate),
        (Instruction::ROL, Addressing::Accumulator),
        (Instruction::ANC, Addressing::Immediate),
        (Instruction::BIT, Addressing::Absolute),
        (Instruction::AND, Addressing::Absolute),
        (Instruction::ROL, Addressing::Absolute),
        (Instruction::RLA, Addressing::Absolute),
        // 0x30-0x3F
        (Instruction::BMI, Addressing::Relative),
        (Instruction::AND, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::RLA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::AND, Addressing::XIndexedZeroPage),
        (Instruction::ROL, Addressing::XIndexedZeroPage),
        (Instruction::RLA, Addressing::XIndexedZeroPage),
        (Instruction::SEC, Addressing::Implied),
        (Instruction::AND, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::RLA, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::AND, Addressing::XIndexedAbsolute),
        (Instruction::ROL, Addressing::XIndexedAbsolute),
        (Instruction::RLA, Addressing::XIndexedAbsolute),
        // 0x40-0x4F
        (Instruction::RTI, Addressing::Implied),
        (Instruction::EOR, Addressing::XIndexedZeroPageIndirect),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::SRE, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::ZeroPage),
        (Instruction::EOR, Addressing::ZeroPage),
        (Instruction::LSR, Addressing::ZeroPage),
        (Instruction::SRE, Addressing::ZeroPage),
        (Instruction::PHA, Addressing::Implied),
        (Instruction::EOR, Addressing::Immediate),
        (Instruction::LSR, Addressing::Accumulator),
        (Instruction::ASR, Addressing::Immediate),
        (Instruction::JMP, Addressing::Absolute),
        (Instruction::EOR, Addressing::Absolute),
        (Instruction::LSR, Addressing::Absolute),
        (Instruction::SRE, Addressing::Absolute),
        // 0x50-0x5F
        (Instruction::BVC, Addressing::Relative),
        (Instruction::EOR, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::SRE, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::EOR, Addressing::XIndexedZeroPage),
        (Instruction::LSR, Addressing::XIndexedZeroPage),
        (Instruction::SRE, Addressing::XIndexedZeroPage),
        (Instruction::CLI, Addressing::Implied),
        (Instruction::EOR, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::SRE, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::EOR, Addressing::XIndexedAbsolute),
        (Instruction::LSR, Addressing::XIndexedAbsolute),
        (Instruction::SRE, Addressing::XIndexedAbsolute),
        // 0x60-0x6F
        (Instruction::RTS, Addressing::Implied),
        (Instruction::ADC, Addressing::XIndexedZeroPageIndirect),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::RRA, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::ZeroPage),
        (Instruction::ADC, Addressing::ZeroPage),
        (Instruction::ROR, Addressing::ZeroPage),
        (Instruction::RRA, Addressing::ZeroPage),
        (Instruction::PLA, Addressing::Implied),
        (Instruction::ADC, Addressing::Immediate),
        (Instruction::ROR, Addressing::Accumulator),
        (Instruction::ARR, Addressing::Immediate),
        (Instruction::JMP, Addressing::AbsoluteIndirect),
        (Instruction::ADC, Addressing::Absolute),
        (Instruction::ROR, Addressing::Absolute),
        (Instruction::RRA, Addressing::Absolute),
        // 0x70-0x7F
        (Instruction::BVS, Addressing::Relative),
        (Instruction::ADC, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::RRA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::ADC, Addressing::XIndexedZeroPage),
        (Instruction::ROR, Addressing::XIndexedZeroPage),
        (Instruction::RRA, Addressing::XIndexedZeroPage),
        (Instruction::SEI, Addressing::Implied),
        (Instruction::ADC, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::RRA, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::ADC, Addressing::XIndexedAbsolute),
        (Instruction::ROR, Addressing::XIndexedAbsolute),
        (Instruction::RRA, Addressing::XIndexedAbsolute),
        // 0x80-0x8F
        (Instruction::NOP, Addressing::Immediate),
        (Instruction::STA, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::Immediate),
        (Instruction::SAX, Addressing::XIndexedZeroPageIndirect),
        (Instruction::STY, Addressing::ZeroPage),
        (Instruction::STA, Addressing::ZeroPage),
        (Instruction::STX, Addressing::ZeroPage),
        (Instruction::SAX, Addressing::ZeroPage),
        (Instruction::DEY, Addressing::Implied),
        (Instruction::NOP, Addressing::Immediate),
        (Instruction::TXA, Addressing::Implied),
        (Instruction::XAA, Addressing::Immediate),
        (Instruction::STY, Addressing::Absolute),
        (Instruction::STA, Addressing::Absolute),
        (Instruction::STX, Addressing::Absolute),
        (Instruction::SAX, Addressing::Absolute),
        // 0x90-0x9F
        (Instruction::BCC, Addressing::Relative),
        (Instruction::STA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::SHA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::STY, Addressing::XIndexedZeroPage),
        (Instruction::STA, Addressing::XIndexedZeroPage),
        (Instruction::STX, Addressing::YIndexedZeroPage),
        (Instruction::SAX, Addressing::YIndexedZeroPage),
        (Instruction::TYA, Addressing::Implied),
        (Instruction::STA, Addressing::YIndexedAbsolute),
        (Instruction::TXS, Addressing::Implied),
        (Instruction::SHS, Addressing::YIndexedAbsolute),
        (Instruction::SHY, Addressing::XIndexedAbsolute),
        (Instruction::STA, Addressing::XIndexedAbsolute),
        (Instruction::SHX, Addressing::YIndexedAbsolute),
        (Instruction::SHA, Addressing::YIndexedAbsolute),
        // 0xA0-0xAF
        (Instruction::LDY, Addressing::Immediate),
        (Instruction::LDA, Addressing::XIndexedZeroPageIndirect),
        (Instruction::LDX, Addressing::Immediate),
        (Instruction::LAX, Addressing::XIndexedZeroPageIndirect),
        (Instruction::LDY, Addressing::ZeroPage),
        (Instruction::LDA, Addressing::ZeroPage),
        (Instruction::LDX, Addressing::ZeroPage),
        (Instruction::LAX, Addressing::ZeroPage),
        (Instruction::TAY, Addressing::Implied),
        (Instruction::LDA, Addressing::Immediate),
        (Instruction::TAX, Addressing::Implied),
        (Instruction::LAX, Addressing::Immediate),
        (Instruction::LDY, Addressing::Absolute),
        (Instruction::LDA, Addressing::Absolute),
        (Instruction::LDX, Addressing::Absolute),
        (Instruction::LAX, Addressing::Absolute),
        // 0xB0-0xBF
        (Instruction::BCS, Addressing::Relative),
        (Instruction::LDA, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::LAX, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::LDY, Addressing::XIndexedZeroPage),
        (Instruction::LDA, Addressing::XIndexedZeroPage),
        (Instruction::LDX, Addressing::YIndexedZeroPage),
        (Instruction::LAX, Addressing::YIndexedZeroPage),
        (Instruction::CLV, Addressing::Implied),
        (Instruction::LDA, Addressing::YIndexedAbsolute),
        (Instruction::TSX, Addressing::Implied),
        (Instruction::LAS, Addressing::YIndexedAbsolute),
        (Instruction::LDY, Addressing::XIndexedAbsolute),
        (Instruction::LDA, Addressing::XIndexedAbsolute),
        (Instruction::LDX, Addressing::YIndexedAbsolute),
        (Instruction::LAX, Addressing::YIndexedAbsolute),
        // 0xC0-0xCF
        (Instruction::CPY, Addressing::Immediate),
        (Instruction::CMP, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::Immediate),
        (Instruction::DCP, Addressing::XIndexedZeroPageIndirect),
        (Instruction::CPY, Addressing::ZeroPage),
        (Instruction::CMP, Addressing::ZeroPage),
        (Instruction::DEC, Addressing::ZeroPage),
        (Instruction::DCP, Addressing::ZeroPage),
        (Instruction::INY, Addressing::Implied),
        (Instruction::CMP, Addressing::Immediate),
        (Instruction::DEX, Addressing::Implied),
        (Instruction::SBX, Addressing::Immediate),
        (Instruction::CPY, Addressing::Absolute),
        (Instruction::CMP, Addressing::Absolute),
        (Instruction::DEC, Addressing::Absolute),
        (Instruction::DCP, Addressing::Absolute),
        // 0xD0-0xDF
        (Instruction::BNE, Addressing::Relative),
        (Instruction::CMP, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::DCP, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::CMP, Addressing::XIndexedZeroPage),
        (Instruction::DEC, Addressing::XIndexedZeroPage),
        (Instruction::DCP, Addressing::XIndexedZeroPage),
        (Instruction::CLD, Addressing::Implied),
        (Instruction::CMP, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::DCP, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::CMP, Addressing::XIndexedAbsolute),
        (Instruction::DEC, Addressing::XIndexedAbsolute),
        (Instruction::DCP, Addressing::XIndexedAbsolute),
        // 0xE0-0xEF
        (Instruction::CPX, Addressing::Immediate),
        (Instruction::SBC, Addressing::XIndexedZeroPageIndirect),
        (Instruction::NOP, Addressing::Immediate),
        (Instruction::ISC, Addressing::XIndexedZeroPageIndirect),
        (Instruction::CPX, Addressing::ZeroPage),
        (Instruction::SBC, Addressing::ZeroPage),
        (Instruction::INC, Addressing::ZeroPage),
        (Instruction::ISC, Addressing::ZeroPage),
        (Instruction::INX, Addressing::Implied),
        (Instruction::SBC, Addressing::Immediate),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::SBC, Addressing::Immediate),
        (Instruction::CPX, Addressing::Absolute),
        (Instruction::SBC, Addressing::Absolute),
        (Instruction::INC, Addressing::Absolute),
        (Instruction::ISC, Addressing::Absolute),
        // 0xF0-0xFF
        (Instruction::BEQ, Addressing::Relative),
        (Instruction::SBC, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::JAM, Addressing::Implied),
        (Instruction::ISC, Addressing::ZeroPageIndirectYIndexed),
        (Instruction::NOP, Addressing::XIndexedZeroPage),
        (Instruction::SBC, Addressing::XIndexedZeroPage),
        (Instruction::INC, Addressing::XIndexedZeroPage),
        (Instruction::ISC, Addressing::XIndexedZeroPage),
        (Instruction::SED, Addressing::Implied),
        (Instruction::SBC, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::Implied),
        (Instruction::ISC, Addressing::YIndexedAbsolute),
        (Instruction::NOP, Addressing::XIndexedAbsolute),
        (Instruction::SBC, Addressing::XIndexedAbsolute),
        (Instruction::INC, Addressing::XIndexedAbsolute),
        (Instruction::ISC, Addressing::XIndexedAbsolute),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InstructionTemplate {
    name: Instruction,
    addr: Addressing,
    ops: &'static [MicroOp],
}

impl InstructionTemplate {
    pub(crate) const fn ldx(addr: Addressing) -> Self {
        Self {
            name: Instruction::LDX,
            addr,
            ops: &[],
        }
    }
}
