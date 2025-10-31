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
            // ===============================
            // Load / Store Instructions
            // ===============================
            Mnemonic::LAS => Self::las(),
            Mnemonic::LAX => Self::lax(),
            Mnemonic::LDA => Self::lda(),
            Mnemonic::LDX => Self::ldx(),
            Mnemonic::LDY => Self::ldy(),
            Mnemonic::SAX => Self::sax(),
            Mnemonic::SHA => Self::sha(),
            Mnemonic::SHX => Self::shx(),
            Mnemonic::SHY => Self::shy(),
            Mnemonic::STA => Self::sta(),
            Mnemonic::STX => Self::stx(),
            Mnemonic::STY => Self::sty(),
            Mnemonic::SHS => Self::shs(),

            // ===============================
            // Transfer Instructions
            // ===============================
            Mnemonic::TAX => Self::tax(),
            Mnemonic::TAY => Self::tay(),
            Mnemonic::TSX => Self::tsx(),
            Mnemonic::TXA => Self::txa(),
            Mnemonic::TXS => Self::txs(),
            Mnemonic::TYA => Self::tya(),

            // ===============================
            // Stack Instructions
            // ===============================
            Mnemonic::PHA => Self::pha(),
            Mnemonic::PHP => Self::php(),
            Mnemonic::PLA => Self::pla(),
            Mnemonic::PLP => Self::plp(),

            // ===============================
            // Shift / Rotate
            // ===============================
            Mnemonic::ASL => Self::asl(),
            Mnemonic::LSR => Self::lsr(),
            Mnemonic::ROL => Self::rol(),
            Mnemonic::ROR => Self::ror(),

            // ===============================
            // Logical
            // ===============================
            Mnemonic::AND => Self::and(),
            Mnemonic::BIT => Self::bit(),
            Mnemonic::EOR => Self::eor(),
            Mnemonic::ORA => Self::ora(),

            // ===============================
            // Arithmetic
            // ===============================
            Mnemonic::ADC => Self::adc(),
            Mnemonic::ANC => Self::anc(),
            Mnemonic::ARR => Self::arr(),
            Mnemonic::ASR => Self::asr(),
            Mnemonic::CMP => Self::cmp(),
            Mnemonic::CPX => Self::cpx(),
            Mnemonic::CPY => Self::cpy(),
            Mnemonic::DCP => Self::dcp(),
            Mnemonic::ISC => Self::isc(),
            Mnemonic::RLA => Self::rla(),
            Mnemonic::RRA => Self::rra(),
            Mnemonic::SBC => Self::sbc(),
            Mnemonic::SBX => Self::sbx(),
            Mnemonic::SLO => Self::slo(),
            Mnemonic::SRE => Self::sre(),
            Mnemonic::XAA => Self::xaa(),

            // ===============================
            // Increment / Decrement
            // ===============================
            Mnemonic::DEC => Self::dec(),
            Mnemonic::DEX => Self::dex(),
            Mnemonic::DEY => Self::dey(),
            Mnemonic::INC => Self::inc(),
            Mnemonic::INX => Self::inx(),
            Mnemonic::INY => Self::iny(),

            // ===============================
            // Control Flow
            // ===============================
            Mnemonic::BRK => Self::brk(),
            Mnemonic::JMP => Self::jmp(),
            Mnemonic::JSR => Self::jsr(),
            Mnemonic::RTI => Self::rti(),
            Mnemonic::RTS => Self::rts(),

            // ===============================
            // Branches
            // ===============================
            Mnemonic::BCC => Self::bcc(),
            Mnemonic::BCS => Self::bcs(),
            Mnemonic::BEQ => Self::beq(),
            Mnemonic::BMI => Self::bmi(),
            Mnemonic::BNE => Self::bne(),
            Mnemonic::BPL => Self::bpl(),
            Mnemonic::BVC => Self::bvc(),
            Mnemonic::BVS => Self::bvs(),

            // ===============================
            // Status Flag Operations
            // ===============================
            Mnemonic::CLC => Self::clc(),
            Mnemonic::CLD => Self::cld(),
            Mnemonic::CLI => Self::cli(),
            Mnemonic::CLV => Self::clv(),
            Mnemonic::SEC => Self::sec(),
            Mnemonic::SED => Self::sed(),
            Mnemonic::SEI => Self::sei(),

            // ===============================
            // Other
            // ===============================
            Mnemonic::JAM => Self::jam(),
            Mnemonic::NOP => Self::nop(),
        }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self).to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::Cpu,
    };

    // Helper: Initialize CPU + Bus with custom memory setup
    pub(crate) fn setup(
        pc: u16,
        a: u8,
        x: u8,
        y: u8,
        s: u8,
        mem_setup: impl FnOnce(&mut MockBus),
    ) -> (Cpu, BusImpl) {
        use crate::{bus::BusImpl, cpu::status::Status};

        let mut mock = MockBus::default();
        mem_setup(&mut mock);

        let mut cpu = Cpu::new();
        cpu.pc = pc;
        cpu.a = a;
        cpu.x = x;
        cpu.y = y;
        cpu.s = s;
        cpu.p = Status::empty();

        (cpu, BusImpl::Dynamic(Box::new(mock)))
    }
}
