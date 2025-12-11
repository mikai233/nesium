use std::fmt::Display;

use crate::bus::Bus;
use crate::cpu::{Cpu, micro_op::MicroOp};

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

#[allow(clippy::upper_case_acronyms)]
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
    /// Longest execution length (in cycles) for static-dispatch exec.
    pub const fn exec_len(&self) -> u8 {
        match self {
            // Load / Store
            Mnemonic::LAS
            | Mnemonic::LAX
            | Mnemonic::LDA
            | Mnemonic::LDX
            | Mnemonic::LDY
            | Mnemonic::SAX
            | Mnemonic::SHA
            | Mnemonic::SHX
            | Mnemonic::SHY
            | Mnemonic::STA
            | Mnemonic::STX
            | Mnemonic::STY => 1,

            // Transfer
            Mnemonic::SHS
            | Mnemonic::TAX
            | Mnemonic::TAY
            | Mnemonic::TSX
            | Mnemonic::TXA
            | Mnemonic::TXS
            | Mnemonic::TYA => 1,

            // Stack
            Mnemonic::PHA | Mnemonic::PHP => 2,
            Mnemonic::PLA | Mnemonic::PLP => 3,

            // Shift / Rotate
            Mnemonic::ASL | Mnemonic::LSR | Mnemonic::ROL | Mnemonic::ROR => 3,

            // Logical
            Mnemonic::AND | Mnemonic::BIT | Mnemonic::EOR | Mnemonic::ORA => 1,

            // Arithmetic
            Mnemonic::ADC
            | Mnemonic::ANC
            | Mnemonic::ARR
            | Mnemonic::ASR
            | Mnemonic::CMP
            | Mnemonic::CPX
            | Mnemonic::CPY
            | Mnemonic::SBC
            | Mnemonic::SBX
            | Mnemonic::XAA => 1,
            Mnemonic::DCP
            | Mnemonic::ISC
            | Mnemonic::RLA
            | Mnemonic::RRA
            | Mnemonic::SLO
            | Mnemonic::SRE => 3,

            // Increment / Decrement
            Mnemonic::DEC | Mnemonic::INC => 3,
            Mnemonic::DEX | Mnemonic::DEY | Mnemonic::INX | Mnemonic::INY => 1,

            // Control Flow
            Mnemonic::BRK => 6,
            Mnemonic::JMP => 0,
            Mnemonic::JSR => 5,
            Mnemonic::RTI => 5,
            Mnemonic::RTS => 5,

            // Branches
            Mnemonic::BCC
            | Mnemonic::BCS
            | Mnemonic::BEQ
            | Mnemonic::BMI
            | Mnemonic::BNE
            | Mnemonic::BPL
            | Mnemonic::BVC
            | Mnemonic::BVS => 3,

            // Status Flags
            Mnemonic::CLC
            | Mnemonic::CLD
            | Mnemonic::CLI
            | Mnemonic::CLV
            | Mnemonic::SEC
            | Mnemonic::SED
            | Mnemonic::SEI => 1,

            // Halt
            Mnemonic::JAM => 1,
            // NOP
            Mnemonic::NOP => 1,
        }
    }

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

    /// Static-dispatch exec entry (prototype; not yet wired into CPU).
    #[allow(dead_code)]
    pub(crate) fn exec<B: Bus>(&self, cpu: &mut Cpu, bus: &mut B, step: u8) {
        match self {
            // Load / Store
            Mnemonic::LAS => load::exec_las(cpu, bus, step),
            Mnemonic::LAX => load::exec_lax(cpu, bus, step),
            Mnemonic::LDA => load::exec_lda(cpu, bus, step),
            Mnemonic::LDX => load::exec_ldx(cpu, bus, step),
            Mnemonic::LDY => load::exec_ldy(cpu, bus, step),
            Mnemonic::SAX => load::exec_sax(cpu, bus, step),
            Mnemonic::SHA => load::exec_sha(cpu, bus, step),
            Mnemonic::SHX => load::exec_shx(cpu, bus, step),
            Mnemonic::SHY => load::exec_shy(cpu, bus, step),
            Mnemonic::STA => load::exec_sta(cpu, bus, step),
            Mnemonic::STX => load::exec_stx(cpu, bus, step),
            Mnemonic::STY => load::exec_sty(cpu, bus, step),
            // Arithmetic / Logic
            Mnemonic::ADC => arith::exec_adc(cpu, bus, step),
            Mnemonic::ANC => arith::exec_anc(cpu, bus, step),
            Mnemonic::ARR => arith::exec_arr(cpu, bus, step),
            Mnemonic::ASR => arith::exec_asr(cpu, bus, step),
            Mnemonic::CMP => arith::exec_cmp(cpu, bus, step),
            Mnemonic::CPX => arith::exec_cpx(cpu, bus, step),
            Mnemonic::CPY => arith::exec_cpy(cpu, bus, step),
            Mnemonic::DCP => arith::exec_dcp(cpu, bus, step),
            Mnemonic::ISC => arith::exec_isc(cpu, bus, step),
            Mnemonic::RLA => arith::exec_rla(cpu, bus, step),
            Mnemonic::RRA => arith::exec_rra(cpu, bus, step),
            Mnemonic::SBC => arith::exec_sbc(cpu, bus, step),
            Mnemonic::SBX => arith::exec_sbx(cpu, bus, step),
            Mnemonic::SLO => arith::exec_slo(cpu, bus, step),
            Mnemonic::SRE => arith::exec_sre(cpu, bus, step),
            Mnemonic::XAA => arith::exec_xaa(cpu, bus, step),
            // Logic
            Mnemonic::AND => logic::exec_and(cpu, bus, step),
            Mnemonic::BIT => logic::exec_bit(cpu, bus, step),
            Mnemonic::EOR => logic::exec_eor(cpu, bus, step),
            Mnemonic::ORA => logic::exec_ora(cpu, bus, step),
            // Shift / Rotate
            Mnemonic::ASL => shift::exec_asl(cpu, bus, step),
            Mnemonic::LSR => shift::exec_lsr(cpu, bus, step),
            Mnemonic::ROL => shift::exec_rol(cpu, bus, step),
            Mnemonic::ROR => shift::exec_ror(cpu, bus, step),
            // Inc/Dec
            Mnemonic::DEC => inc::exec_dec(cpu, bus, step),
            Mnemonic::DEX => inc::exec_dex(cpu, bus, step),
            Mnemonic::DEY => inc::exec_dey(cpu, bus, step),
            Mnemonic::INC => inc::exec_inc(cpu, bus, step),
            Mnemonic::INX => inc::exec_inx(cpu, bus, step),
            Mnemonic::INY => inc::exec_iny(cpu, bus, step),
            // Branches
            Mnemonic::BCC => bra::exec_bcc(cpu, bus, step),
            Mnemonic::BCS => bra::exec_bcs(cpu, bus, step),
            Mnemonic::BEQ => bra::exec_beq(cpu, bus, step),
            Mnemonic::BMI => bra::exec_bmi(cpu, bus, step),
            Mnemonic::BNE => bra::exec_bne(cpu, bus, step),
            Mnemonic::BPL => bra::exec_bpl(cpu, bus, step),
            Mnemonic::BVC => bra::exec_bvc(cpu, bus, step),
            Mnemonic::BVS => bra::exec_bvs(cpu, bus, step),
            // Control Flow
            Mnemonic::BRK => ctrl::exec_brk(cpu, bus, step),
            Mnemonic::JMP => ctrl::exec_jmp(cpu, bus, step),
            Mnemonic::JSR => ctrl::exec_jsr(cpu, bus, step),
            Mnemonic::RTI => ctrl::exec_rti(cpu, bus, step),
            Mnemonic::RTS => ctrl::exec_rts(cpu, bus, step),
            // Flags
            Mnemonic::CLC => flags::exec_clc(cpu, bus, step),
            Mnemonic::CLD => flags::exec_cld(cpu, bus, step),
            Mnemonic::CLI => flags::exec_cli(cpu, bus, step),
            Mnemonic::CLV => flags::exec_clv(cpu, bus, step),
            Mnemonic::SEC => flags::exec_sec(cpu, bus, step),
            Mnemonic::SED => flags::exec_sed(cpu, bus, step),
            Mnemonic::SEI => flags::exec_sei(cpu, bus, step),
            // Stack
            Mnemonic::PHA => stack::exec_pha(cpu, bus, step),
            Mnemonic::PHP => stack::exec_php(cpu, bus, step),
            Mnemonic::PLA => stack::exec_pla(cpu, bus, step),
            Mnemonic::PLP => stack::exec_plp(cpu, bus, step),
            // Transfer
            Mnemonic::SHS => trans::exec_shs(cpu, bus, step),
            Mnemonic::TAX => trans::exec_tax(cpu, bus, step),
            Mnemonic::TAY => trans::exec_tay(cpu, bus, step),
            Mnemonic::TSX => trans::exec_tsx(cpu, bus, step),
            Mnemonic::TXA => trans::exec_txa(cpu, bus, step),
            Mnemonic::TXS => trans::exec_txs(cpu, bus, step),
            Mnemonic::TYA => trans::exec_tya(cpu, bus, step),
            // Halt
            Mnemonic::JAM => kil::exec_jam(cpu, bus, step),
            // NOP
            Mnemonic::NOP => nop::exec_nop(cpu, bus, step),
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

    use std::panic::{self, AssertUnwindSafe};

    use rand::SeedableRng;
    use tracing::{error, trace};

    use crate::{
        bus::{Bus, mock::MockBus},
        cpu::{
            Cpu, addressing::Addressing, instruction::Instruction, lookup::LOOKUP_TABLE,
            mnemonic::Mnemonic, status::Status,
        },
        tests::TEST_COUNT,
    };

    #[derive(Debug)]
    pub(crate) struct Verification {
        pub(crate) cpu: Cpu,
        pub(crate) addr_hi: u8,
        pub(crate) addr: u16,
        pub(crate) m: u8,
    }

    impl Verification {
        pub(crate) fn check_nz(&self, status: Status, val: u8) {
            if val == 0 {
                assert!(status.z());
            } else {
                assert!(!status.z());
            }
            if val & 0x80 != 0 {
                assert!(status.n())
            } else {
                assert!(!status.n())
            }
        }
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

        pub(crate) fn test<F>(&self, verify: F)
        where
            F: Fn(&Verification, &mut Cpu, &mut dyn Bus),
        {
            for _ in 0..TEST_COUNT {
                let seed = rand::random();
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    self.run(seed, &verify);
                }));

                if let Err(err) = result {
                    error!(
                        "InstrTest for {:?} failed with seed {}",
                        self.mnemonic, seed
                    );
                    panic::resume_unwind(err);
                }
            }
        }

        pub(crate) fn test_branch<F>(&self, verify: F)
        where
            F: Fn(&Verification, &mut Cpu, &mut dyn Bus) -> bool,
        {
            for _ in 0..TEST_COUNT {
                let seed = rand::random();
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    self.run_branch(seed, &verify);
                }));

                if let Err(err) = result {
                    error!(
                        "Branch InstrTest for {:?} failed with seed {}",
                        self.mnemonic, seed
                    );
                    panic::resume_unwind(err);
                }
            }
        }

        pub(crate) fn run<F>(&self, seed: u64, verify: &F)
        where
            F: Fn(&Verification, &mut Cpu, &mut dyn Bus),
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    trace!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verification, mut bus, crossed_page) =
                        Self::build_mock(&instr, &cpu, &mut rng);
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let expected = instr.cycle().total_cycle(crossed_page, false);
                    assert_eq!(
                        executed, expected,
                        "instruction: {} cycle not match on {}",
                        instr.mnemonic, instr.addressing
                    );
                    verify(&verification, &mut cpu, &mut bus);
                }
            }
        }

        pub(crate) fn run_branch<F>(&self, seed: u64, verify: F)
        where
            F: Fn(&Verification, &mut Cpu, &mut dyn Bus) -> bool,
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    trace!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verification, mut bus, crossed_page) =
                        Self::build_mock(&instr, &cpu, &mut rng);
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let branch_taken = verify(&verification, &mut cpu, &mut bus);
                    let expected = instr.cycle().total_cycle(crossed_page, branch_taken);
                    assert_eq!(executed, expected, "instruction cycle not match");
                }
            }
        }

        /// Build a mock bus for testing 6502 instructions.
        /// Ensures addresses do not overwrite protected regions.
        fn build_mock<R>(
            instr: &Instruction,
            cpu: &Cpu,
            rng: &mut R,
        ) -> (Verification, MockBus, bool)
        where
            R: rand::Rng,
        {
            let mut mock = MockBus::default();
            let mut crossed_page = false;
            let data = rng.random();
            let mut addr_hi = 0;

            // Write opcode at PC
            mock.mem_write(cpu.pc, instr.opcode());

            // Base protected addresses to avoid collisions
            let protected_addrs = vec![cpu.pc, cpu.pc + 1, cpu.pc + 2, cpu.pc + 3];

            // Dispatch to helper functions based on addressing mode
            let addr = match instr.addressing {
                Addressing::Implied | Addressing::Accumulator => 0,
                Addressing::Immediate => Self::immediate(&mut mock, cpu, data),
                Addressing::ZeroPage => {
                    Self::zero_page(&mut mock, cpu, rng, data, &protected_addrs)
                }
                Addressing::ZeroPageX => {
                    Self::zero_page_x(&mut mock, cpu, rng, data, &protected_addrs)
                }
                Addressing::ZeroPageY => {
                    Self::zero_page_y(&mut mock, cpu, rng, data, &protected_addrs)
                }
                Addressing::Absolute => Self::absolute(&mut mock, cpu, rng, data, &protected_addrs),
                Addressing::AbsoluteX => {
                    let (hi, eff, crossed) =
                        Self::absolute_x(&mut mock, cpu, rng, data, &protected_addrs);
                    addr_hi = hi;
                    crossed_page = crossed;
                    eff
                }
                Addressing::AbsoluteY => {
                    let (hi, eff, crossed) =
                        Self::absolute_y(&mut mock, cpu, rng, data, &protected_addrs);
                    addr_hi = hi;
                    crossed_page = crossed;
                    eff
                }
                Addressing::Indirect => Self::indirect(&mut mock, cpu, rng, &protected_addrs),
                Addressing::IndirectX => {
                    Self::indirect_x(&mut mock, cpu, rng, data, &protected_addrs)
                }
                Addressing::IndirectY => {
                    let (hi, eff, crossed) =
                        Self::indirect_y(&mut mock, cpu, rng, data, &protected_addrs);
                    crossed_page = crossed;
                    addr_hi = hi;
                    eff
                }
                Addressing::Relative => {
                    let (eff, crossed) =
                        Self::relative(&mut mock, cpu, rng, data, &protected_addrs);
                    crossed_page = crossed;
                    eff
                }
            };

            let verify = Verification {
                cpu: *cpu,
                addr_hi,
                addr,
                m: data,
            };

            (verify, mock, crossed_page)
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Helper functions for each addressing mode
        ////////////////////////////////////////////////////////////////////////////////

        fn immediate(mock: &mut MockBus, cpu: &Cpu, data: u8) -> u16 {
            // Immediate mode: write operand at PC+1
            mock.mem_write(cpu.pc + 1, data);
            cpu.pc + 1
        }

        fn zero_page<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> u16
        where
            R: rand::Rng,
        {
            // Pick a zero-page address not overlapping protected addresses
            let addr = loop {
                let candidate: u8 = rng.random();
                if !protected_addrs.contains(&(candidate as u16)) {
                    break candidate;
                }
            };
            mock.mem_write(cpu.pc + 1, addr);
            mock.mem_write(addr as u16, data);
            trace!("ZeroPage: addr={:02X}", addr);
            addr as u16
        }

        fn zero_page_x<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> u16
        where
            R: rand::Rng,
        {
            let (base, effective) = loop {
                let base_candidate: u8 = rng.random();
                let effective_candidate = base_candidate.wrapping_add(cpu.x);
                if !protected_addrs.contains(&(effective_candidate as u16)) {
                    break (base_candidate, effective_candidate);
                }
            };
            mock.mem_write(cpu.pc + 1, base);
            mock.mem_write(effective as u16, data);
            trace!(
                "ZeroPageX: base={:02X}, X={:02X}, effective={:02X}",
                base, cpu.x, effective
            );
            effective as u16
        }

        fn zero_page_y<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> u16
        where
            R: rand::Rng,
        {
            let (base, effective) = loop {
                let base_candidate: u8 = rng.random();
                let effective_candidate = base_candidate.wrapping_add(cpu.y);
                if !protected_addrs.contains(&(effective_candidate as u16)) {
                    break (base_candidate, effective_candidate);
                }
            };
            mock.mem_write(cpu.pc + 1, base);
            mock.mem_write(effective as u16, data);
            trace!(
                "ZeroPageY: base={:02X}, Y={:02X}, effective={:02X}",
                base, cpu.y, effective
            );
            effective as u16
        }

        fn absolute<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> u16
        where
            R: rand::Rng,
        {
            let addr = loop {
                let candidate: u16 = rng.random_range(0x0000..=0xFFFF);
                if !protected_addrs.contains(&candidate) {
                    break candidate;
                }
            };
            mock.mem_write(cpu.pc + 1, (addr & 0xFF) as u8);
            mock.mem_write(cpu.pc + 2, (addr >> 8) as u8);
            mock.mem_write(addr, data);
            trace!("Absolute: addr={:04X}", addr);
            addr
        }

        fn absolute_x<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> (u8, u16, bool)
        where
            R: rand::Rng,
        {
            let (base, effective) = loop {
                let base_candidate: u16 = rng.random_range(0x0000..=0xFFFF);
                let effective_candidate = base_candidate.wrapping_add(cpu.x as u16);
                if !protected_addrs.contains(&base_candidate)
                    && !protected_addrs.contains(&effective_candidate)
                {
                    break (base_candidate, effective_candidate);
                }
            };
            let crossed_page = (base & 0xFF00) != (effective & 0xFF00);
            mock.mem_write(cpu.pc + 1, (base & 0xFF) as u8);
            mock.mem_write(cpu.pc + 2, (base >> 8) as u8);
            mock.mem_write(effective, data);
            trace!(
                "AbsoluteX: base={:04X}, effective={:04X}, crossed={}",
                base, effective, crossed_page
            );
            let hi = (base >> 8) as u8;
            (hi, effective, crossed_page)
        }

        fn absolute_y<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> (u8, u16, bool)
        where
            R: rand::Rng,
        {
            let (base, effective) = loop {
                let base_candidate: u16 = rng.random_range(0x0000..=0xFFFF);
                let effective_candidate = base_candidate.wrapping_add(cpu.y as u16);
                if !protected_addrs.contains(&base_candidate)
                    && !protected_addrs.contains(&effective_candidate)
                {
                    break (base_candidate, effective_candidate);
                }
            };
            let crossed_page = (base & 0xFF00) != (effective & 0xFF00);
            mock.mem_write(cpu.pc + 1, (base & 0xFF) as u8);
            mock.mem_write(cpu.pc + 2, (base >> 8) as u8);
            mock.mem_write(effective, data);
            trace!(
                "AbsoluteY: base={:04X}, effective={:04X}, crossed={}",
                base, effective, crossed_page
            );
            let hi = (base >> 8) as u8;
            (hi, effective, crossed_page)
        }

        fn indirect<R>(mock: &mut MockBus, cpu: &Cpu, rng: &mut R, protected_addrs: &[u16]) -> u16
        where
            R: rand::Rng,
        {
            let (ptr, target) = loop {
                let ptr_candidate: u16 = rng.random_range(0x0000..=0xFFFE);
                let hi_addr = (ptr_candidate & 0xFF00) | ((ptr_candidate + 1) & 0x00FF);
                let target_candidate: u16 = rng.random_range(0x0000..=0xFFFF);

                if protected_addrs.contains(&ptr_candidate)
                    || protected_addrs.contains(&hi_addr)
                    || protected_addrs.contains(&target_candidate)
                {
                    continue;
                }

                break (ptr_candidate, target_candidate);
            };
            let hi_addr = (ptr & 0xFF00) | ((ptr + 1) & 0x00FF);
            mock.mem_write(cpu.pc + 1, (ptr & 0xFF) as u8);
            mock.mem_write(cpu.pc + 2, (ptr >> 8) as u8);
            mock.mem_write(ptr, (target & 0xFF) as u8);
            mock.mem_write(hi_addr, (target >> 8) as u8);
            trace!(
                "Indirect: ptr={:04X}, hi_addr={:04X}, target={:04X}",
                ptr, hi_addr, target
            );
            target
        }

        fn indirect_x<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> u16
        where
            R: rand::Rng,
        {
            let (zp, target) = loop {
                let zp_candidate: u8 = rng.random();
                let ptr = zp_candidate.wrapping_add(cpu.x);
                let ptr_lo = ptr as u16;
                let ptr_hi = ptr.wrapping_add(1) as u16 & 0xFF;
                let target_candidate: u16 = rng.random_range(0x0000..=0xFFFF);

                if protected_addrs.contains(&ptr_lo)
                    || protected_addrs.contains(&ptr_hi)
                    || protected_addrs.contains(&target_candidate)
                    || target_candidate == ptr_lo
                    || target_candidate == ptr_hi
                {
                    continue;
                }

                break (zp_candidate, target_candidate);
            };
            let ptr = zp.wrapping_add(cpu.x);
            mock.mem_write(cpu.pc + 1, zp);
            mock.mem_write(ptr as u16, (target & 0xFF) as u8);
            mock.mem_write(ptr.wrapping_add(1) as u16 & 0xFF, (target >> 8) as u8);
            mock.mem_write(target, data);
            trace!("IndirectX: target={:04X}", target);
            target
        }

        fn indirect_y<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> (u8, u16, bool)
        where
            R: rand::Rng,
        {
            let (zp, lo, hi) = loop {
                let zp_candidate: u8 = rng.random();
                let zp_lo = zp_candidate as u16;
                let zp_hi = zp_candidate.wrapping_add(1) as u16;

                if protected_addrs.contains(&zp_lo) || protected_addrs.contains(&zp_hi) {
                    continue;
                }

                let lo_candidate: u8 = rng.random();
                let hi_candidate: u8 = rng.random();
                let base = ((hi_candidate as u16) << 8) | lo_candidate as u16;
                let effective = base.wrapping_add(cpu.y as u16);

                if protected_addrs.contains(&effective) || effective == zp_lo || effective == zp_hi
                {
                    continue;
                }

                break (zp_candidate, lo_candidate, hi_candidate);
            };
            let base = ((hi as u16) << 8) | lo as u16;
            let effective = base.wrapping_add(cpu.y as u16);
            let crossed_page = (base & 0xFF00) != (effective & 0xFF00);

            mock.mem_write(cpu.pc + 1, zp);
            mock.mem_write(zp as u16, lo);
            mock.mem_write(zp.wrapping_add(1) as u16 & 0xFF, hi);
            mock.mem_write(effective, data);

            trace!(
                "IndirectY: effective={:04X}, crossed={}",
                effective, crossed_page
            );
            (hi, effective, crossed_page)
        }

        fn relative<R>(
            mock: &mut MockBus,
            cpu: &Cpu,
            rng: &mut R,
            data: u8,
            protected_addrs: &[u16],
        ) -> (u16, bool)
        where
            R: rand::Rng,
        {
            let (offset, target) = loop {
                let offset_candidate: i8 = rng.random_range(-128..=127);
                let base = cpu.pc.wrapping_add(2);
                let target_candidate = base.wrapping_add(offset_candidate as i16 as u16);
                if !protected_addrs.contains(&target_candidate) {
                    break (offset_candidate, target_candidate);
                }
            };
            let crossed_page = (cpu.pc.wrapping_add(2) & 0xFF00) != (target & 0xFF00);
            mock.mem_write(cpu.pc + 1, offset as u8);
            mock.mem_write(target, data);

            trace!(
                "Relative: offset={}, base={:04X}, target={:04X}, crossed={}",
                offset,
                cpu.pc.wrapping_add(2),
                target,
                crossed_page
            );

            (target, crossed_page)
        }
    }

    /// Ensure `exec_len()` stays in sync with the actual `exec` step table for each mnemonic.
    #[test]
    fn exec_len_matches_steps() {
        let mut seen = std::collections::HashSet::new();
        let mut pairs = Vec::new();
        for (opcode, instr) in LOOKUP_TABLE.iter().enumerate() {
            if seen.insert(instr.mnemonic) {
                pairs.push((instr.mnemonic, opcode as u8));
            }
        }

        for (mnemonic, opcode) in pairs {
            let len = mnemonic.exec_len();
            if len == 0 {
                continue;
            }
            let mut cpu = Cpu::new();
            let mut bus = MockBus::default();
            cpu.reset(&mut bus, crate::reset_kind::ResetKind::PowerOn);
            cpu.opcode_in_flight = Some(opcode);

            let mut actual_len = 0usize;
            let max_steps = 8usize; // Longest path is BRK (6 cycles in exec phase)

            for step in 0..max_steps {
                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    mnemonic.exec(&mut cpu, &mut bus, step as u8);
                }));
                if result.is_ok() {
                    actual_len += 1;
                } else {
                    break;
                }
            }

            assert!(
                actual_len as u8 == len,
                "mnemonic {:?} (opcode 0x{:02X}) exec_len mismatch: declared {} but executed {} steps before panic",
                mnemonic,
                opcode,
                len,
                actual_len
            );
        }
    }
}
