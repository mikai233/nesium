use std::fmt::Display;

use crate::bus::CpuBus;
use crate::context::Context;
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

/// TODO:: Not handle dma yet
#[inline]
pub(crate) fn hi_byte_store_final(
    cpu: &mut Cpu,
    bus: &mut CpuBus,
    ctx: &mut Context,
    value_reg: u8,
) {
    let base_hi = cpu.tmp;
    let addr = cpu.effective_addr;

    let crossed = ((addr >> 8) as u8) != base_hi;
    let mut hi = (addr >> 8) as u8;
    let lo = (addr & 0x00FF) as u8;

    if crossed {
        hi &= value_reg;
    }

    let value = value_reg & base_hi.wrapping_add(1);
    let final_addr = ((hi as u16) << 8) | (lo as u16);

    bus.mem_write(final_addr, value, cpu, ctx);
}

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
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
        match self {
            // Load / Store
            Mnemonic::LAS => load::exec_las(cpu, bus, ctx, step),
            Mnemonic::LAX => load::exec_lax(cpu, bus, ctx, step),
            Mnemonic::LDA => load::exec_lda(cpu, bus, ctx, step),
            Mnemonic::LDX => load::exec_ldx(cpu, bus, ctx, step),
            Mnemonic::LDY => load::exec_ldy(cpu, bus, ctx, step),
            Mnemonic::SAX => load::exec_sax(cpu, bus, ctx, step),
            Mnemonic::SHA => load::exec_sha(cpu, bus, ctx, step),
            Mnemonic::SHX => load::exec_shx(cpu, bus, ctx, step),
            Mnemonic::SHY => load::exec_shy(cpu, bus, ctx, step),
            Mnemonic::STA => load::exec_sta(cpu, bus, ctx, step),
            Mnemonic::STX => load::exec_stx(cpu, bus, ctx, step),
            Mnemonic::STY => load::exec_sty(cpu, bus, ctx, step),

            // Transfer
            Mnemonic::SHS => trans::exec_shs(cpu, bus, ctx, step),
            Mnemonic::TAX => trans::exec_tax(cpu, bus, ctx, step),
            Mnemonic::TAY => trans::exec_tay(cpu, bus, ctx, step),
            Mnemonic::TSX => trans::exec_tsx(cpu, bus, ctx, step),
            Mnemonic::TXA => trans::exec_txa(cpu, bus, ctx, step),
            Mnemonic::TXS => trans::exec_txs(cpu, bus, ctx, step),
            Mnemonic::TYA => trans::exec_tya(cpu, bus, ctx, step),

            // Stack
            Mnemonic::PHA => stack::exec_pha(cpu, bus, ctx, step),
            Mnemonic::PHP => stack::exec_php(cpu, bus, ctx, step),
            Mnemonic::PLA => stack::exec_pla(cpu, bus, ctx, step),
            Mnemonic::PLP => stack::exec_plp(cpu, bus, ctx, step),

            // Shift / Rotate
            Mnemonic::ASL => shift::exec_asl(cpu, bus, ctx, step),
            Mnemonic::LSR => shift::exec_lsr(cpu, bus, ctx, step),
            Mnemonic::ROL => shift::exec_rol(cpu, bus, ctx, step),
            Mnemonic::ROR => shift::exec_ror(cpu, bus, ctx, step),

            // Logical
            Mnemonic::AND => logic::exec_and(cpu, bus, ctx, step),
            Mnemonic::BIT => logic::exec_bit(cpu, bus, ctx, step),
            Mnemonic::EOR => logic::exec_eor(cpu, bus, ctx, step),
            Mnemonic::ORA => logic::exec_ora(cpu, bus, ctx, step),

            // Arithmetic
            Mnemonic::ADC => arith::exec_adc(cpu, bus, ctx, step),
            Mnemonic::ANC => arith::exec_anc(cpu, bus, ctx, step),
            Mnemonic::ARR => arith::exec_arr(cpu, bus, ctx, step),
            Mnemonic::ASR => arith::exec_asr(cpu, bus, ctx, step),
            Mnemonic::CMP => arith::exec_cmp(cpu, bus, ctx, step),
            Mnemonic::CPX => arith::exec_cpx(cpu, bus, ctx, step),
            Mnemonic::CPY => arith::exec_cpy(cpu, bus, ctx, step),
            Mnemonic::DCP => arith::exec_dcp(cpu, bus, ctx, step),
            Mnemonic::ISC => arith::exec_isc(cpu, bus, ctx, step),
            Mnemonic::RLA => arith::exec_rla(cpu, bus, ctx, step),
            Mnemonic::RRA => arith::exec_rra(cpu, bus, ctx, step),
            Mnemonic::SBC => arith::exec_sbc(cpu, bus, ctx, step),
            Mnemonic::SBX => arith::exec_sbx(cpu, bus, ctx, step),
            Mnemonic::SLO => arith::exec_slo(cpu, bus, ctx, step),
            Mnemonic::SRE => arith::exec_sre(cpu, bus, ctx, step),
            Mnemonic::XAA => arith::exec_xaa(cpu, bus, ctx, step),

            // Increment / Decrement
            Mnemonic::DEC => inc::exec_dec(cpu, bus, ctx, step),
            Mnemonic::DEX => inc::exec_dex(cpu, bus, ctx, step),
            Mnemonic::DEY => inc::exec_dey(cpu, bus, ctx, step),
            Mnemonic::INC => inc::exec_inc(cpu, bus, ctx, step),
            Mnemonic::INX => inc::exec_inx(cpu, bus, ctx, step),
            Mnemonic::INY => inc::exec_iny(cpu, bus, ctx, step),

            // Control Flow
            Mnemonic::BRK => ctrl::exec_brk(cpu, bus, ctx, step),
            Mnemonic::JMP => ctrl::exec_jmp(cpu, bus, ctx, step),
            Mnemonic::JSR => ctrl::exec_jsr(cpu, bus, ctx, step),
            Mnemonic::RTI => ctrl::exec_rti(cpu, bus, ctx, step),
            Mnemonic::RTS => ctrl::exec_rts(cpu, bus, ctx, step),

            // Branches
            Mnemonic::BCC => bra::exec_bcc(cpu, bus, ctx, step),
            Mnemonic::BCS => bra::exec_bcs(cpu, bus, ctx, step),
            Mnemonic::BEQ => bra::exec_beq(cpu, bus, ctx, step),
            Mnemonic::BMI => bra::exec_bmi(cpu, bus, ctx, step),
            Mnemonic::BNE => bra::exec_bne(cpu, bus, ctx, step),
            Mnemonic::BPL => bra::exec_bpl(cpu, bus, ctx, step),
            Mnemonic::BVC => bra::exec_bvc(cpu, bus, ctx, step),
            Mnemonic::BVS => bra::exec_bvs(cpu, bus, ctx, step),

            // Flags
            Mnemonic::CLC => flags::exec_clc(cpu, bus, ctx, step),
            Mnemonic::CLD => flags::exec_cld(cpu, bus, ctx, step),
            Mnemonic::CLI => flags::exec_cli(cpu, bus, ctx, step),
            Mnemonic::CLV => flags::exec_clv(cpu, bus, ctx, step),
            Mnemonic::SEC => flags::exec_sec(cpu, bus, ctx, step),
            Mnemonic::SED => flags::exec_sed(cpu, bus, ctx, step),
            Mnemonic::SEI => flags::exec_sei(cpu, bus, ctx, step),

            // Halt
            Mnemonic::JAM => kil::exec_jam(cpu, bus, ctx, step),

            // NOP
            Mnemonic::NOP => nop::exec_nop(cpu, bus, ctx, step),
        }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self).to_lowercase())
    }
}
