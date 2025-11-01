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
    use rand::{Rng, SeedableRng};

    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::{
            Cpu,
            addressing::Addressing,
            instruction::Instruction,
            lookup::LOOKUP_TABLE,
            mnemonic::{self, Mnemonic},
            status::Status,
        },
    };

    #[derive(Debug)]
    pub(crate) struct InstrTest {
        mnemonic: Mnemonic,
    }

    impl InstrTest {
        pub(crate) fn new(mnemonic: Mnemonic) -> Self {
            Self { mnemonic }
        }

        pub(crate) fn run<F>(&self, seed: u64, f: F)
        where
            F: Fn(&Instruction, &Cpu, &Cpu, &mut BusImpl),
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let mut cpu = Cpu::new();
            cpu.a = rng.random();
            cpu.x = rng.random();
            cpu.y = rng.random();
            cpu.s = rng.random();
            cpu.p = Status::from_bits_truncate(rng.random());
            cpu.pc = 0x8000;
            let mut mock = MockBus::default();

            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    let mut crossed_page = false;
                    mock.mem[cpu.pc as usize] = instr.opcode();
                    match instr.addressing {
                        Addressing::Implied | Addressing::Accumulator => {}

                        Addressing::Immediate => {
                            let value: u8 = rng.random();
                            mock.mem[(cpu.pc + 1) as usize] = value;
                        }

                        Addressing::ZeroPage => {
                            let addr: u8 = rng.random();
                            mock.mem[(cpu.pc + 1) as usize] = addr;
                            mock.mem[addr as usize] = rng.random();
                        }

                        Addressing::ZeroPageX => {
                            let base: u8 = rng.random();
                            let effective = base.wrapping_add(cpu.x);
                            mock.mem[(cpu.pc + 1) as usize] = base;
                            mock.mem[effective as usize] = rng.random();
                        }

                        Addressing::ZeroPageY => {
                            let base: u8 = rng.random();
                            let effective = base.wrapping_add(cpu.y);
                            mock.mem[(cpu.pc + 1) as usize] = base;
                            mock.mem[effective as usize] = rng.random();
                        }

                        Addressing::Absolute => {
                            let addr: u16 = rng.random_range(0x0000..=0xFFFF);
                            mock.mem[(cpu.pc + 1) as usize] = (addr & 0xFF) as u8;
                            mock.mem[(cpu.pc + 2) as usize] = (addr >> 8) as u8;
                            mock.mem[addr as usize] = rng.random();
                        }

                        Addressing::AbsoluteX => {
                            let base: u16 = rng.random_range(0x0000..=0xFFFF);
                            cpu.base = (base >> 8) as u8; // Store high byte useful for some unofficial instructions
                            let effective = base.wrapping_add(cpu.x as u16);
                            if (base & 0xFF00) != (effective & 0xFF00) {
                                crossed_page = true;
                            }
                            mock.mem[(cpu.pc + 1) as usize] = (base & 0xFF) as u8;
                            mock.mem[(cpu.pc + 2) as usize] = (base >> 8) as u8;
                            mock.mem[effective as usize] = rng.random();
                        }

                        Addressing::AbsoluteY => {
                            let base: u16 = rng.random_range(0x0000..=0xFFFF);
                            cpu.base = (base >> 8) as u8; // Store high byte useful for some unofficial instructions
                            let effective = base.wrapping_add(cpu.y as u16);
                            if (base & 0xFF00) != (effective & 0xFF00) {
                                crossed_page = true;
                            }
                            mock.mem[(cpu.pc + 1) as usize] = (base & 0xFF) as u8;
                            mock.mem[(cpu.pc + 2) as usize] = (base >> 8) as u8;
                            mock.mem[effective as usize] = rng.random();
                        }

                        Addressing::Indirect => {
                            let ptr: u16 = rng.random_range(0x0000..=0xFFFF);
                            let target: u16 = rng.random();
                            mock.mem[(cpu.pc + 1) as usize] = (ptr & 0xFF) as u8;
                            mock.mem[(cpu.pc + 2) as usize] = (ptr >> 8) as u8;
                            mock.mem[ptr as usize] = (target & 0xFF) as u8;
                            mock.mem[(ptr + 1) as usize] = (target >> 8) as u8;
                        }

                        Addressing::IndirectX => {
                            let zp: u8 = rng.random();
                            let ptr = zp.wrapping_add(cpu.x);
                            let target: u16 = rng.random_range(0x0000..=0xFFFF);
                            mock.mem[(cpu.pc + 1) as usize] = zp;
                            mock.mem[ptr as usize] = (target & 0xFF) as u8;
                            mock.mem[(ptr.wrapping_add(1)) as usize] = (target >> 8) as u8;
                            mock.mem[target as usize] = rng.random();
                        }

                        Addressing::IndirectY => {
                            let zp: u8 = rng.random();
                            let base: u16 = rng.random_range(0x0000..=0xFFFF);
                            cpu.base = (base >> 8) as u8; // Store high byte useful for some unofficial instructions
                            let effective = base.wrapping_add(cpu.y as u16);
                            mock.mem[(cpu.pc + 1) as usize] = zp;
                            mock.mem[zp as usize] = (base & 0xFF) as u8;
                            mock.mem[(zp.wrapping_add(1)) as usize] = (base >> 8) as u8;
                            mock.mem[effective as usize] = rng.random();
                        }

                        Addressing::Relative => {
                            let offset: i8 = rng.random_range(-128..=127);
                            mock.mem[(cpu.pc + 1) as usize] = offset as u8;
                        }
                    }
                }
            }
        }
    }

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
