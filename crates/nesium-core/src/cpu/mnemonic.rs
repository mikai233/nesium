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
    use std::collections::HashSet;

    use rand::SeedableRng;
    use tracing::{debug, info};

    use crate::{
        bus::{Bus, BusImpl, mock::MockBus},
        cpu::{
            Cpu, addressing::Addressing, instruction::Instruction, lookup::LOOKUP_TABLE,
            mnemonic::Mnemonic, status::Status,
        },
    };

    #[derive(Debug)]
    pub(crate) struct Verification {
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

        pub(crate) fn run<F>(&self, seed: u64, verify: F)
        where
            F: Fn(&Instruction, &Verification, &Cpu, &mut BusImpl),
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    debug!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verification, bus, crossed_page) =
                        Self::build_mock(&instr, &mut cpu, &mut rng);
                    let mut bus = BusImpl::Dynamic(Box::new(bus));
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let expected = instr.cycle().total_cycle(crossed_page, false);
                    assert_eq!(
                        executed, expected,
                        "instruction: {} cycle not match on {}",
                        instr.mnemonic, instr.addressing
                    );
                    verify(&instr, &verification, &cpu, &mut bus);
                }
            }
        }

        pub(crate) fn run_branch<F>(&self, seed: u64, verify: F)
        where
            F: Fn(&Instruction, &Verification, &Cpu, &mut BusImpl) -> bool,
        {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            for instr in LOOKUP_TABLE {
                if instr.mnemonic == self.mnemonic {
                    debug!("test instruction: {}", instr);
                    let mut cpu = Self::rand_cpu(&mut rng);
                    let (verification, bus, crossed_page) =
                        Self::build_mock(&instr, &cpu, &mut rng);
                    let mut bus = BusImpl::Dynamic(Box::new(bus));
                    let executed = cpu.test_clock(&mut bus, &instr);
                    let branch_taken = verify(&instr, &verification, &cpu, &mut bus);
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
            mock.write(cpu.pc, instr.opcode());

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
            mock.write(cpu.pc + 1, data);
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
            mock.write(cpu.pc + 1, addr);
            mock.write(addr as u16, data);
            debug!("ZeroPage: addr={:02X}", addr);
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
            mock.write(cpu.pc + 1, base);
            mock.write(effective as u16, data);
            debug!(
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
            mock.write(cpu.pc + 1, base);
            mock.write(effective as u16, data);
            debug!(
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
            mock.write(cpu.pc + 1, (addr & 0xFF) as u8);
            mock.write(cpu.pc + 2, (addr >> 8) as u8);
            mock.write(addr, data);
            debug!("Absolute: addr={:04X}", addr);
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
            mock.write(cpu.pc + 1, (base & 0xFF) as u8);
            mock.write(cpu.pc + 2, (base >> 8) as u8);
            mock.write(effective, data);
            debug!(
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
            mock.write(cpu.pc + 1, (base & 0xFF) as u8);
            mock.write(cpu.pc + 2, (base >> 8) as u8);
            mock.write(effective, data);
            debug!(
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
            mock.write(cpu.pc + 1, (ptr & 0xFF) as u8);
            mock.write(cpu.pc + 2, (ptr >> 8) as u8);
            mock.write(ptr, (target & 0xFF) as u8);
            mock.write(hi_addr, (target >> 8) as u8);
            debug!(
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
                let ptr = zp_candidate.wrapping_add(cpu.x) & 0xFF;
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
            let ptr = zp.wrapping_add(cpu.x) & 0xFF;
            mock.write(cpu.pc + 1, zp);
            mock.write(ptr as u16, (target & 0xFF) as u8);
            mock.write(ptr.wrapping_add(1) as u16 & 0xFF, (target >> 8) as u8);
            mock.write(target, data);
            debug!("IndirectX: target={:04X}", target);
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
                let zp_hi = zp_candidate.wrapping_add(1) as u16 & 0xFF;

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

            mock.write(cpu.pc + 1, zp);
            mock.write(zp as u16, lo);
            mock.write(zp.wrapping_add(1) as u16 & 0xFF, hi);
            mock.write(effective, data);

            debug!(
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
            mock.write(cpu.pc + 1, offset as u8);
            mock.write(target, data);

            debug!(
                "Relative: offset={}, base={:04X}, target={:04X}, crossed={}",
                offset,
                cpu.pc.wrapping_add(2),
                target,
                crossed_page
            );

            (target, crossed_page)
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
