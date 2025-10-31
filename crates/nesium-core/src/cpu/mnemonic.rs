use std::fmt::Display;

use crate::cpu::micro_op::MicroOp;

pub mod arith;
pub mod bra;
pub mod ctrl;
pub mod flags;
pub mod inc;
pub mod kil;
pub mod load;
pub mod logic;
pub mod nop;
pub mod shift;
pub mod stack;
pub mod trans;

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

impl Mnemonic {
    pub(crate) const fn micro_ops(&self) -> &'static [MicroOp] {
        match self {
            Mnemonic::LAS => todo!(),
            Mnemonic::LAX => todo!(),
            Mnemonic::LDA => Self::lda(),
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
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self).to_lowercase())
    }
}
