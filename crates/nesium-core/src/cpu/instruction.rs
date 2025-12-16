use std::fmt::Display;

use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{
        Cpu,
        addressing::Addressing,
        lookup::LOOKUP_TABLE,
        mnemonic::Mnemonic,
        timing::{CYCLE_TABLE, Timing},
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Instruction {
    pub(crate) mnemonic: Mnemonic,
    pub(crate) addressing: Addressing,
}

impl Instruction {
    pub(crate) const fn len(&self) -> u8 {
        self.addr_len() + self.mnemonic.exec_len()
    }

    pub(crate) const fn addr_len(&self) -> u8 {
        self.addressing.exec_len()
    }

    /// Execute a single cycle step using static dispatch (addressing then mnemonic).
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
        let addr_len = self.addr_len();
        if step < addr_len {
            self.addressing.exec(cpu, bus, ctx, step);
        } else {
            let offset = step - addr_len;
            self.mnemonic.exec(cpu, bus, ctx, offset);
        }
    }

    // Load/Store
    pub(crate) const fn las(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LAS,
            addressing: addr,
        }
    }
    pub(crate) const fn lax(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LAX,
            addressing: addr,
        }
    }
    pub(crate) const fn lda(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LDA,
            addressing: addr,
        }
    }
    pub(crate) const fn ldx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LDX,
            addressing: addr,
        }
    }
    pub(crate) const fn ldy(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LDY,
            addressing: addr,
        }
    }
    pub(crate) const fn sax(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SAX,
            addressing: addr,
        }
    }
    pub(crate) const fn sha(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SHA,
            addressing: addr,
        }
    }
    pub(crate) const fn shx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SHX,
            addressing: addr,
        }
    }
    pub(crate) const fn shy(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SHY,
            addressing: addr,
        }
    }
    pub(crate) const fn sta(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::STA,
            addressing: addr,
        }
    }
    pub(crate) const fn stx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::STX,
            addressing: addr,
        }
    }
    pub(crate) const fn sty(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::STY,
            addressing: addr,
        }
    }

    // Transfer
    pub(crate) const fn shs(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SHS,
            addressing: addr,
        }
    }
    pub(crate) const fn tax(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TAX,
            addressing: addr,
        }
    }
    pub(crate) const fn tay(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TAY,
            addressing: addr,
        }
    }
    pub(crate) const fn tsx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TSX,
            addressing: addr,
        }
    }
    pub(crate) const fn txa(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TXA,
            addressing: addr,
        }
    }
    pub(crate) const fn txs(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TXS,
            addressing: addr,
        }
    }
    pub(crate) const fn tya(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::TYA,
            addressing: addr,
        }
    }

    // Stack
    pub(crate) const fn pha(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::PHA,
            addressing: addr,
        }
    }
    pub(crate) const fn php(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::PHP,
            addressing: addr,
        }
    }
    pub(crate) const fn pla(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::PLA,
            addressing: addr,
        }
    }
    pub(crate) const fn plp(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::PLP,
            addressing: addr,
        }
    }

    // Shift
    pub(crate) const fn asl(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ASL,
            addressing: addr,
        }
    }
    pub(crate) const fn lsr(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::LSR,
            addressing: addr,
        }
    }
    pub(crate) const fn rol(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ROL,
            addressing: addr,
        }
    }
    pub(crate) const fn ror(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ROR,
            addressing: addr,
        }
    }

    // Logic
    pub(crate) const fn and(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::AND,
            addressing: addr,
        }
    }
    pub(crate) const fn bit(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BIT,
            addressing: addr,
        }
    }
    pub(crate) const fn eor(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::EOR,
            addressing: addr,
        }
    }
    pub(crate) const fn ora(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ORA,
            addressing: addr,
        }
    }

    // Arithmetic
    pub(crate) const fn adc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ADC,
            addressing: addr,
        }
    }
    pub(crate) const fn anc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ANC,
            addressing: addr,
        }
    }
    pub(crate) const fn arr(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ARR,
            addressing: addr,
        }
    }
    pub(crate) const fn asr(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ASR,
            addressing: addr,
        }
    }
    pub(crate) const fn cmp(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CMP,
            addressing: addr,
        }
    }
    pub(crate) const fn cpx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CPX,
            addressing: addr,
        }
    }
    pub(crate) const fn cpy(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CPY,
            addressing: addr,
        }
    }
    pub(crate) const fn dcp(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::DCP,
            addressing: addr,
        }
    }
    pub(crate) const fn isc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::ISC,
            addressing: addr,
        }
    }
    pub(crate) const fn rla(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::RLA,
            addressing: addr,
        }
    }
    pub(crate) const fn rra(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::RRA,
            addressing: addr,
        }
    }
    pub(crate) const fn sbc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SBC,
            addressing: addr,
        }
    }
    pub(crate) const fn sbx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SBX,
            addressing: addr,
        }
    }
    pub(crate) const fn slo(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SLO,
            addressing: addr,
        }
    }
    pub(crate) const fn sre(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SRE,
            addressing: addr,
        }
    }
    pub(crate) const fn xaa(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::XAA,
            addressing: addr,
        }
    }

    // Arithmetic: Inc/Dec
    pub(crate) const fn dec(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::DEC,
            addressing: addr,
        }
    }
    pub(crate) const fn dex(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::DEX,
            addressing: addr,
        }
    }
    pub(crate) const fn dey(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::DEY,
            addressing: addr,
        }
    }
    pub(crate) const fn inc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::INC,
            addressing: addr,
        }
    }
    pub(crate) const fn inx(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::INX,
            addressing: addr,
        }
    }
    pub(crate) const fn iny(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::INY,
            addressing: addr,
        }
    }

    // Control Flow
    pub(crate) const fn brk(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BRK,
            addressing: addr,
        }
    }
    pub(crate) const fn jmp(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::JMP,
            addressing: addr,
        }
    }
    pub(crate) const fn jsr(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::JSR,
            addressing: addr,
        }
    }
    pub(crate) const fn rti(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::RTI,
            addressing: addr,
        }
    }
    pub(crate) const fn rts(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::RTS,
            addressing: addr,
        }
    }

    // Control Flow: Branch
    pub(crate) const fn bcc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BCC,
            addressing: addr,
        }
    }
    pub(crate) const fn bcs(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BCS,
            addressing: addr,
        }
    }
    pub(crate) const fn beq(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BEQ,
            addressing: addr,
        }
    }
    pub(crate) const fn bmi(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BMI,
            addressing: addr,
        }
    }
    pub(crate) const fn bne(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BNE,
            addressing: addr,
        }
    }
    pub(crate) const fn bpl(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BPL,
            addressing: addr,
        }
    }
    pub(crate) const fn bvc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BVC,
            addressing: addr,
        }
    }
    pub(crate) const fn bvs(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::BVS,
            addressing: addr,
        }
    }

    // Flags
    pub(crate) const fn clc(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CLC,
            addressing: addr,
        }
    }
    pub(crate) const fn cld(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CLD,
            addressing: addr,
        }
    }
    pub(crate) const fn cli(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CLI,
            addressing: addr,
        }
    }
    pub(crate) const fn clv(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::CLV,
            addressing: addr,
        }
    }
    pub(crate) const fn sec(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SEC,
            addressing: addr,
        }
    }
    pub(crate) const fn sed(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SED,
            addressing: addr,
        }
    }
    pub(crate) const fn sei(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::SEI,
            addressing: addr,
        }
    }

    // KIL
    pub(crate) const fn jam(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::JAM,
            addressing: addr,
        }
    }

    // NOP
    pub(crate) const fn nop(addr: Addressing) -> Self {
        Self {
            mnemonic: Mnemonic::NOP,
            addressing: addr,
        }
    }

    pub(crate) fn opcode(&self) -> u8 {
        LOOKUP_TABLE
            .iter()
            .position(|instr| instr == self)
            .expect("instruction not found in LOOKUP_TABLE") as u8
    }

    pub(crate) fn cycle(&self) -> Timing {
        CYCLE_TABLE[self.opcode() as usize]
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.mnemonic, self.addressing)
    }
}
