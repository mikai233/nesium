use crate::cpu::{addressing::Addressing, micro_op::MicroOp, status::Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Instruction {
    //load
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
    //trans
    SHS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    //stack
    PHA,
    PHP,
    PLA,
    PLP,
    //shift
    ASL,
    LSR,
    ROL,
    ROR,
    //logic
    AND,
    BIT,
    EOR,
    ORA,
    //arith
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
    //inc
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    //ctrl
    BRK,
    JMP,
    JSR,
    RTI,
    RTS,
    //bra
    BCC,
    BCS,
    BEQ,
    BMI,
    BNE,
    BPL,
    BVC,
    BVS,
    //flags
    CLC,
    CLD,
    CLI,
    CLV,
    SEC,
    SED,
    SEI,
    //kill
    JAM,
    //nop
    NOP,
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
            //load
            Instruction::LAS
            | Instruction::LAX
            | Instruction::LDA
            | Instruction::LDX
            | Instruction::LDY => {
                status.update_negative(result);
                status.update_zero(result);
            }
            Instruction::SAX
            | Instruction::SHA
            | Instruction::SHX
            | Instruction::SHY
            | Instruction::STA
            | Instruction::STX
            | Instruction::STY => {}
            //
            Instruction::SHS => {}
            Instruction::TAX | Instruction::TAY | Instruction::TSX | Instruction::TXA => {
                status.update_negative(result);
                status.update_zero(result);
            }
            Instruction::TXS => {}
            Instruction::TYA => {
                status.update_negative(result);
                status.update_zero(result);
            }
            //
            Instruction::PHA | Instruction::PHP => {}
            Instruction::PLA => {
                status.update_negative(result);
                status.update_zero(result);
            }
            Instruction::PLP => {
                // Restore all flags from stack value
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            //
            Instruction::ASL => {
                status.update_negative(result);
                status.update_zero(result);
                if let Some(c) = carry {
                    status.set(Status::CARRY, c);
                }
            }
            Instruction::LSR => {
                status.remove(Status::NEGATIVE);
                status.update_zero(result);
                if let Some(c) = carry {
                    status.set(Status::CARRY, c);
                }
            }
            Instruction::ROL | Self::ROR => {
                status.update_negative(result);
                status.update_zero(result);
                if let Some(c) = carry {
                    status.set(Status::CARRY, c);
                }
            }
            Instruction::AND => {
                status.update_negative(result);
                status.update_zero(result);
            }
            Instruction::BIT => {
                status.update_negative(result);
                status.update_zero(result);
                if let Some(o) = overflow {
                    status.set(Status::OVERFLOW, o);
                }
            }
            Instruction::EOR | Instruction::ORA => {
                status.update_negative(result);
                status.update_zero(result);
            }
            //
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InstructionTemplate {
    name: &'static Instruction,
    addr: &'static Addressing,
    ops: &'static [MicroOp],
}

impl InstructionTemplate {
    fn ldx(addr: &'static Addressing) -> Self {
        todo!()
    }
}
