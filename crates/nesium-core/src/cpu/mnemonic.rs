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
    use rand::SeedableRng;
    use tracing::debug;

    use crate::{
        bus::{Bus, BusImpl, mock::MockBus},
        cpu::{
            Cpu, addressing::Addressing, instruction::Instruction, lookup::LOOKUP_TABLE,
            mnemonic::Mnemonic, status::Status,
        },
    };

    #[derive(Debug)]
    pub(crate) struct Verify {
        pub(crate) cpu: Cpu,
        pub(crate) addr_hi: u8,
        pub(crate) addr: u16,
        pub(crate) m: u8,
    }

    #[derive(Debug)]
    pub(crate) struct InstrTest {
        mnemonic: Mnemonic,
    }

    impl InstrTest {
        pub(crate) fn new(mnemonic: Mnemonic) -> Self {
            Self { mnemonic }
        }

        pub(crate) fn rand_cpu<R>(rng: &mut R) -> Cpu
        where
            R: rand::Rng,
        {
            let mut cpu = Cpu::new();
            cpu.a = rng.random();
            cpu.x = rng.random();
            cpu.y = rng.random();
            cpu.s = rng.random();
            cpu.p = Status::from_bits_truncate(rng.random());
            cpu.pc = 0x8000;
            cpu
        }

        pub(crate) fn run<F>(&self, seed: u64, verify_fn: F)
        where
            F: Fn(&Instruction, &Verify, &Cpu, &mut BusImpl),
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    debug!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verify, bus, crossed_page) = Self::build_mock(&instr, &mut cpu, &mut rng);
                    let mut bus = BusImpl::Dynamic(Box::new(bus));
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let expected = instr.cycle().total_cycle(crossed_page, false);
                    assert_eq!(
                        executed, expected,
                        "instruction: {} cycle not match on {}",
                        instr.mnemonic, instr.addressing
                    );
                    verify_fn(&instr, &verify, &cpu, &mut bus);
                }
            }
        }

        pub(crate) fn run_branch<F>(&self, seed: u64, verify_fn: F)
        where
            F: Fn(&Instruction, &Verify, &Cpu, &mut BusImpl) -> bool,
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    debug!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verify, bus, crossed_page) = Self::build_mock(&instr, &cpu, &mut rng);
                    let mut bus = BusImpl::Dynamic(Box::new(bus));
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let branch_taken = verify_fn(&instr, &verify, &cpu, &mut bus);
                    let expected = instr.cycle().total_cycle(crossed_page, branch_taken);
                    assert_eq!(executed, expected, "instruction cycle not match");
                }
            }
        }

        fn build_mock<R>(instr: &Instruction, cpu: &Cpu, rng: &mut R) -> (Verify, MockBus, bool)
        where
            R: rand::Rng,
        {
            let mut mock = MockBus::default();
            let mut crossed_page = false;
            let data = rng.random();
            let mut addr_hi = 0;

            // 写入 opcode 到 PC
            mock.write(cpu.pc, instr.opcode());

            let addr = match instr.addressing {
                // ------------------------------------------------------------
                // Implied / Accumulator
                // ------------------------------------------------------------
                Addressing::Implied | Addressing::Accumulator => {
                    // 无操作数，指令仅使用寄存器或状态
                    0
                }

                // ------------------------------------------------------------
                // Immediate (#$nn)
                // ------------------------------------------------------------
                Addressing::Immediate => {
                    mock.write(cpu.pc + 1, data);
                    cpu.pc + 1
                }

                // ------------------------------------------------------------
                // Zero Page ($nn)
                // ------------------------------------------------------------
                Addressing::ZeroPage => {
                    let addr: u8 = rng.random();
                    mock.write(cpu.pc + 1, addr);
                    mock.write(addr as u16, data);
                    addr as u16
                }

                // ------------------------------------------------------------
                // Zero Page,X ($nn,X)
                // ------------------------------------------------------------
                Addressing::ZeroPageX => {
                    let base: u8 = rng.random();
                    let effective = base.wrapping_add(cpu.x);
                    mock.write(cpu.pc + 1, base);
                    mock.write(effective as u16, data);
                    effective as u16
                }

                // ------------------------------------------------------------
                // Zero Page,Y ($nn,Y)
                // ------------------------------------------------------------
                Addressing::ZeroPageY => {
                    let base: u8 = rng.random();
                    let effective = base.wrapping_add(cpu.y);
                    mock.write(cpu.pc + 1, base);
                    mock.write(effective as u16, data);
                    effective as u16
                }

                // ------------------------------------------------------------
                // Absolute ($nnnn)
                // ------------------------------------------------------------
                Addressing::Absolute => {
                    let addr: u16 = rng.random_range(0x0000..=0xFFFF);
                    mock.write(cpu.pc + 1, (addr & 0xFF) as u8);
                    mock.write(cpu.pc + 2, (addr >> 8) as u8);
                    mock.write(addr, data);
                    addr
                }

                // ------------------------------------------------------------
                // Absolute,X ($nnnn,X)
                // ------------------------------------------------------------
                Addressing::AbsoluteX => {
                    let base: u16 = rng.random_range(0x0000..=0xFFFF);
                    let effective = base.wrapping_add(cpu.x as u16);

                    if (base & 0xFF00) != (effective & 0xFF00) {
                        crossed_page = true;
                    }

                    // 仅非官方指令（如 SHS/SHY/SHX）使用 base 高字节
                    if matches!(
                        instr.mnemonic,
                        Mnemonic::SHS | Mnemonic::SHY | Mnemonic::SHX
                    ) {
                        addr_hi = (base >> 8) as u8;
                    }

                    mock.write(cpu.pc + 1, (base & 0xFF) as u8);
                    mock.write(cpu.pc + 2, (base >> 8) as u8);
                    mock.write(effective, data);
                    effective
                }

                // ------------------------------------------------------------
                // Absolute,Y ($nnnn,Y)
                // ------------------------------------------------------------
                Addressing::AbsoluteY => {
                    let base: u16 = rng.random_range(0x0000..=0xFFFF);
                    let effective = base.wrapping_add(cpu.y as u16);

                    if (base & 0xFF00) != (effective & 0xFF00) {
                        crossed_page = true;
                    }

                    if matches!(
                        instr.mnemonic,
                        Mnemonic::SHS | Mnemonic::SHY | Mnemonic::SHX
                    ) {
                        addr_hi = (base >> 8) as u8;
                    }

                    mock.write(cpu.pc + 1, (base & 0xFF) as u8);
                    mock.write(cpu.pc + 2, (base >> 8) as u8);
                    mock.write(effective, data);
                    effective
                }

                // ------------------------------------------------------------
                // Indirect ($nnnn) — typically used only by JMP
                // ------------------------------------------------------------
                Addressing::Indirect => {
                    let ptr: u16 = rng.random_range(0x0000..=0xFFFE); // 避免溢出
                    let target: u16 = rng.random_range(0x0000..=0xFFFF);

                    mock.write(cpu.pc + 1, (ptr & 0xFF) as u8);
                    mock.write(cpu.pc + 2, (ptr >> 8) as u8);
                    mock.write(ptr, (target & 0xFF) as u8);

                    // 模拟 JMP ($xxFF) 硬件 bug：高字节 wrap-around
                    let hi_addr = (ptr & 0xFF00) | ((ptr + 1) & 0x00FF);
                    mock.write(hi_addr, (target >> 8) as u8);
                    target
                }

                // ------------------------------------------------------------
                // (Indirect,X) - ($nn,X)
                // ------------------------------------------------------------
                Addressing::IndirectX => {
                    let zp: u8 = rng.random();
                    let ptr = zp.wrapping_add(cpu.x) & 0xFF;
                    let target: u16 = rng.random_range(0x0000..=0xFFFF);

                    mock.write(cpu.pc + 1, zp);
                    mock.write(ptr as u16, (target & 0xFF) as u8);
                    mock.write(ptr.wrapping_add(1) as u16 & 0xFF, (target >> 8) as u8);
                    mock.write(target, data);
                    target
                }

                // ------------------------------------------------------------
                // (Indirect),Y - ($nn),Y
                // ------------------------------------------------------------
                Addressing::IndirectY => {
                    let zp: u8 = rng.random();
                    let lo: u8 = rng.random();
                    let hi: u8 = rng.random();
                    let base = ((hi as u16) << 8) | lo as u16;
                    let effective = base.wrapping_add(cpu.y as u16);

                    if (base & 0xFF00) != (effective & 0xFF00) {
                        crossed_page = true;
                    }

                    mock.write(cpu.pc + 1, zp);
                    mock.write(zp as u16, lo);
                    mock.write(zp.wrapping_add(1) as u16 & 0xFF, hi);
                    mock.write(effective, data);
                    debug!("indirect y addr: {:04x}",effective);
                    effective
                }

                // ------------------------------------------------------------
                // Relative (branch offset)
                // ------------------------------------------------------------
                Addressing::Relative => {
                    let offset: i8 = rng.random_range(-128..=127);
                    mock.write(cpu.pc + 1, offset as u8);
                    let base = cpu.pc.wrapping_add(2);
                    let target = base.wrapping_add(offset as i16 as u16);
                    if (base & 0xFF00) != (target & 0xFF00) {
                        crossed_page = true;
                    }
                    mock.write(target, data);
                    target
                }
            };

            let verify = Verify {
                cpu: *cpu,
                addr_hi,
                addr,
                m: data,
            };

            (verify, mock, crossed_page)
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
