use std::ops::Index;

use crate::cpu::{addressing::Addressing, micro_op::MicroOp, mnemonic::Mnemonic};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Instruction {
    pub(crate) opcode: Mnemonic,
    pub(crate) addressing: Addressing,
}

impl Instruction {
    pub(crate) const fn len(&self) -> usize {
        self.opcode.micro_ops().len() + self.addr_len()
    }

    pub(crate) const fn addr_len(&self) -> usize {
        self.addressing.micro_ops().len()
    }

    // Load/Store
    pub(crate) const fn las(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LAS,
            addressing: addr,
        }
    }
    pub(crate) const fn lax(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LAX,
            addressing: addr,
        }
    }
    pub(crate) const fn lda(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LDA,
            addressing: addr,
        }
    }
    pub(crate) const fn ldx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LDX,
            addressing: addr,
        }
    }
    pub(crate) const fn ldy(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LDY,
            addressing: addr,
        }
    }
    pub(crate) const fn sax(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SAX,
            addressing: addr,
        }
    }
    pub(crate) const fn sha(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SHA,
            addressing: addr,
        }
    }
    pub(crate) const fn shx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SHX,
            addressing: addr,
        }
    }
    pub(crate) const fn shy(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SHY,
            addressing: addr,
        }
    }
    pub(crate) const fn sta(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::STA,
            addressing: addr,
        }
    }
    pub(crate) const fn stx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::STX,
            addressing: addr,
        }
    }
    pub(crate) const fn sty(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::STY,
            addressing: addr,
        }
    }

    // Transfer
    pub(crate) const fn shs(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SHS,
            addressing: addr,
        }
    }
    pub(crate) const fn tax(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TAX,
            addressing: addr,
        }
    }
    pub(crate) const fn tay(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TAY,
            addressing: addr,
        }
    }
    pub(crate) const fn tsx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TSX,
            addressing: addr,
        }
    }
    pub(crate) const fn txa(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TXA,
            addressing: addr,
        }
    }
    pub(crate) const fn txs(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TXS,
            addressing: addr,
        }
    }
    pub(crate) const fn tya(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::TYA,
            addressing: addr,
        }
    }

    // Stack
    pub(crate) const fn pha(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::PHA,
            addressing: addr,
        }
    }
    pub(crate) const fn php(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::PHP,
            addressing: addr,
        }
    }
    pub(crate) const fn pla(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::PLA,
            addressing: addr,
        }
    }
    pub(crate) const fn plp(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::PLP,
            addressing: addr,
        }
    }

    // Shift
    pub(crate) const fn asl(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ASL,
            addressing: addr,
        }
    }
    pub(crate) const fn lsr(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LSR,
            addressing: addr,
        }
    }
    pub(crate) const fn rol(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ROL,
            addressing: addr,
        }
    }
    pub(crate) const fn ror(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ROR,
            addressing: addr,
        }
    }

    // Logic
    pub(crate) const fn and(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::AND,
            addressing: addr,
        }
    }
    pub(crate) const fn bit(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BIT,
            addressing: addr,
        }
    }
    pub(crate) const fn eor(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::EOR,
            addressing: addr,
        }
    }
    pub(crate) const fn ora(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ORA,
            addressing: addr,
        }
    }

    // Arithmetic
    pub(crate) const fn adc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ADC,
            addressing: addr,
        }
    }
    pub(crate) const fn anc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ANC,
            addressing: addr,
        }
    }
    pub(crate) const fn arr(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ARR,
            addressing: addr,
        }
    }
    pub(crate) const fn asr(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ASR,
            addressing: addr,
        }
    }
    pub(crate) const fn cmp(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CMP,
            addressing: addr,
        }
    }
    pub(crate) const fn cpx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CPX,
            addressing: addr,
        }
    }
    pub(crate) const fn cpy(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CPY,
            addressing: addr,
        }
    }
    pub(crate) const fn dcp(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::DCP,
            addressing: addr,
        }
    }
    pub(crate) const fn isc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::ISC,
            addressing: addr,
        }
    }
    pub(crate) const fn rla(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::RLA,
            addressing: addr,
        }
    }
    pub(crate) const fn rra(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::RRA,
            addressing: addr,
        }
    }
    pub(crate) const fn sbc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SBC,
            addressing: addr,
        }
    }
    pub(crate) const fn sbx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SBX,
            addressing: addr,
        }
    }
    pub(crate) const fn slo(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SLO,
            addressing: addr,
        }
    }
    pub(crate) const fn sre(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SRE,
            addressing: addr,
        }
    }
    pub(crate) const fn xaa(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::XAA,
            addressing: addr,
        }
    }

    // Arithmetic: Inc/Dec
    pub(crate) const fn dec(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::DEC,
            addressing: addr,
        }
    }
    pub(crate) const fn dex(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::DEX,
            addressing: addr,
        }
    }
    pub(crate) const fn dey(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::DEY,
            addressing: addr,
        }
    }
    pub(crate) const fn inc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::INC,
            addressing: addr,
        }
    }
    pub(crate) const fn inx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::INX,
            addressing: addr,
        }
    }
    pub(crate) const fn iny(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::INY,
            addressing: addr,
        }
    }

    // Control Flow
    pub(crate) const fn brk(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BRK,
            addressing: addr,
        }
    }
    pub(crate) const fn jmp(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::JMP,
            addressing: addr,
        }
    }
    pub(crate) const fn jsr(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::JSR,
            addressing: addr,
        }
    }
    pub(crate) const fn rti(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::RTI,
            addressing: addr,
        }
    }
    pub(crate) const fn rts(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::RTS,
            addressing: addr,
        }
    }

    // Control Flow: Branch
    pub(crate) const fn bcc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BCC,
            addressing: addr,
        }
    }
    pub(crate) const fn bcs(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BCS,
            addressing: addr,
        }
    }
    pub(crate) const fn beq(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BEQ,
            addressing: addr,
        }
    }
    pub(crate) const fn bmi(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BMI,
            addressing: addr,
        }
    }
    pub(crate) const fn bne(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BNE,
            addressing: addr,
        }
    }
    pub(crate) const fn bpl(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BPL,
            addressing: addr,
        }
    }
    pub(crate) const fn bvc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BVC,
            addressing: addr,
        }
    }
    pub(crate) const fn bvs(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::BVS,
            addressing: addr,
        }
    }

    // Flags
    pub(crate) const fn clc(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CLC,
            addressing: addr,
        }
    }
    pub(crate) const fn cld(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CLD,
            addressing: addr,
        }
    }
    pub(crate) const fn cli(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CLI,
            addressing: addr,
        }
    }
    pub(crate) const fn clv(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::CLV,
            addressing: addr,
        }
    }
    pub(crate) const fn sec(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SEC,
            addressing: addr,
        }
    }
    pub(crate) const fn sed(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SED,
            addressing: addr,
        }
    }
    pub(crate) const fn sei(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::SEI,
            addressing: addr,
        }
    }

    // KIL
    pub(crate) const fn jam(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::JAM,
            addressing: addr,
        }
    }

    // NOP
    pub(crate) const fn nop(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::NOP,
            addressing: addr,
        }
    }
}

impl Index<usize> for Instruction {
    type Output = MicroOp;

    fn index(&self, index: usize) -> &Self::Output {
        let len = self.addr_len();
        if index < len {
            &self.addressing.micro_ops()[index]
        } else {
            &self.opcode.micro_ops()[index]
        }
    }
}
