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
            //load
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
            //
            Instruction::SHS => {}
            Instruction::TAX | Instruction::TAY | Instruction::TSX | Instruction::TXA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::TXS => {}
            Instruction::TYA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            //
            Instruction::PHA | Instruction::PHP => {}
            Instruction::PLA => {
                status!(status, result, carry, overflow; N:*, Z:*);
            }
            Instruction::PLP => {
                // Restore all flags from stack value
                *status = Status::from_bits_truncate(result | Status::UNUSED.bits());
            }
            //
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
            //
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
            //
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
            //
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
