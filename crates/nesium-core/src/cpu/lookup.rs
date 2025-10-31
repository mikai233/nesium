use crate::cpu::addressing::Addressing as A;
use crate::cpu::instruction::Instruction as I;

pub(crate) type Table = [I; 256];

pub(crate) static LOOKUP_TABLE: Table = [
    //  0x00          0x01          0x02          0x03          0x04          0x05          0x06          0x07
    I::brk(A::Implied),
    I::ora(A::IndirectX),
    I::jam(A::Implied),
    I::slo(A::IndirectX),
    I::nop(A::ZeroPage),
    I::ora(A::ZeroPage),
    I::asl(A::ZeroPage),
    I::slo(A::ZeroPage),
    I::php(A::Implied),
    I::ora(A::Immediate),
    I::asl(A::Accumulator),
    I::anc(A::Immediate),
    I::nop(A::Absolute),
    I::ora(A::Absolute),
    I::asl(A::Absolute),
    I::slo(A::Absolute),
    //  0x08          0x09          0x0A          0x0B          0x0C          0x0D          0x0E          0x0F
    I::bpl(A::Relative),
    I::ora(A::IndirectY),
    I::jam(A::Implied),
    I::slo(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::ora(A::ZeroPageX),
    I::asl(A::ZeroPageX),
    I::slo(A::ZeroPageX),
    I::clc(A::Implied),
    I::ora(A::AbsoluteY),
    I::nop(A::Implied),
    I::slo(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::ora(A::AbsoluteX),
    I::asl(A::AbsoluteX),
    I::slo(A::AbsoluteX),
    //  0x10          0x11          0x12          0x13          0x14          0x15          0x16          0x17
    I::jsr(A::Absolute),
    I::and(A::IndirectX),
    I::jam(A::Implied),
    I::rla(A::IndirectX),
    I::bit(A::ZeroPage),
    I::and(A::ZeroPage),
    I::rol(A::ZeroPage),
    I::rla(A::ZeroPage),
    I::plp(A::Implied),
    I::and(A::Immediate),
    I::rol(A::Accumulator),
    I::anc(A::Immediate),
    I::bit(A::Absolute),
    I::and(A::Absolute),
    I::rol(A::Absolute),
    I::rla(A::Absolute),
    //  0x18          0x19          0x1A          0x1B          0x1C          0x1D          0x1E          0x1F
    I::bmi(A::Relative),
    I::and(A::IndirectY),
    I::jam(A::Implied),
    I::rla(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::and(A::ZeroPageX),
    I::rol(A::ZeroPageX),
    I::rla(A::ZeroPageX),
    I::sec(A::Implied),
    I::and(A::AbsoluteY),
    I::nop(A::Implied),
    I::rla(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::and(A::AbsoluteX),
    I::rol(A::AbsoluteX),
    I::rla(A::AbsoluteX),
    //  0x20          0x21          0x22          0x23          0x24          0x25          0x26          0x27
    I::rti(A::Implied),
    I::eor(A::IndirectX),
    I::jam(A::Implied),
    I::sre(A::IndirectX),
    I::nop(A::ZeroPage),
    I::eor(A::ZeroPage),
    I::lsr(A::ZeroPage),
    I::sre(A::ZeroPage),
    I::pha(A::Implied),
    I::eor(A::Immediate),
    I::lsr(A::Accumulator),
    I::asr(A::Immediate),
    I::jmp(A::Absolute),
    I::eor(A::Absolute),
    I::lsr(A::Absolute),
    I::sre(A::Absolute),
    //  0x28          0x29          0x2A          0x2B          0x2C          0x2D          0x2E          0x2F
    I::bvc(A::Relative),
    I::eor(A::IndirectY),
    I::jam(A::Implied),
    I::sre(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::eor(A::ZeroPageX),
    I::lsr(A::ZeroPageX),
    I::sre(A::ZeroPageX),
    I::cli(A::Implied),
    I::eor(A::AbsoluteY),
    I::nop(A::Implied),
    I::sre(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::eor(A::AbsoluteX),
    I::lsr(A::AbsoluteX),
    I::sre(A::AbsoluteX),
    //  0x30          0x31          0x32          0x33          0x34          0x35          0x36          0x37
    I::rts(A::Implied),
    I::adc(A::IndirectX),
    I::jam(A::Implied),
    I::rra(A::IndirectX),
    I::nop(A::ZeroPage),
    I::adc(A::ZeroPage),
    I::ror(A::ZeroPage),
    I::rra(A::ZeroPage),
    I::pla(A::Implied),
    I::adc(A::Immediate),
    I::ror(A::Accumulator),
    I::arr(A::Immediate),
    I::jmp(A::Indirect),
    I::adc(A::Absolute),
    I::ror(A::Absolute),
    I::rra(A::Absolute),
    //  0x38          0x39          0x3A          0x3B          0x3C          0x3D          0x3E          0x3F
    I::bvs(A::Relative),
    I::adc(A::IndirectY),
    I::jam(A::Implied),
    I::rra(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::adc(A::ZeroPageX),
    I::ror(A::ZeroPageX),
    I::rra(A::ZeroPageX),
    I::sei(A::Implied),
    I::adc(A::AbsoluteY),
    I::nop(A::Implied),
    I::rra(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::adc(A::AbsoluteX),
    I::ror(A::AbsoluteX),
    I::rra(A::AbsoluteX),
    //  0x40          0x41          0x42          0x43          0x44          0x45          0x46          0x47
    I::nop(A::Immediate),
    I::sta(A::IndirectX),
    I::nop(A::Immediate),
    I::sax(A::IndirectX),
    I::sty(A::ZeroPage),
    I::sta(A::ZeroPage),
    I::stx(A::ZeroPage),
    I::sax(A::ZeroPage),
    I::dey(A::Implied),
    I::nop(A::Immediate),
    I::txa(A::Implied),
    I::xaa(A::Immediate),
    I::sty(A::Absolute),
    I::sta(A::Absolute),
    I::stx(A::Absolute),
    I::sax(A::Absolute),
    //  0x48          0x49          0x4A          0x4B          0x4C          0x4D          0x4E          0x4F
    I::bcc(A::Relative),
    I::sta(A::IndirectY),
    I::jam(A::Implied),
    I::sha(A::IndirectY),
    I::sty(A::ZeroPageX),
    I::sta(A::ZeroPageX),
    I::stx(A::ZeroPageY),
    I::sax(A::ZeroPageY),
    I::tya(A::Implied),
    I::sta(A::AbsoluteY),
    I::txs(A::Implied),
    I::shs(A::AbsoluteY),
    I::shy(A::AbsoluteX),
    I::sta(A::AbsoluteX),
    I::shx(A::AbsoluteY),
    I::sha(A::AbsoluteY),
    //  0x50          0x51          0x52          0x53          0x54          0x55          0x56          0x57
    I::ldy(A::Immediate),
    I::lda(A::IndirectX),
    I::ldx(A::Immediate),
    I::lax(A::IndirectX),
    I::ldy(A::ZeroPage),
    I::lda(A::ZeroPage),
    I::ldx(A::ZeroPage),
    I::lax(A::ZeroPage),
    I::tay(A::Implied),
    I::lda(A::Immediate),
    I::tax(A::Implied),
    I::lax(A::Immediate),
    I::ldy(A::Absolute),
    I::lda(A::Absolute),
    I::ldx(A::Absolute),
    I::lax(A::Absolute),
    //  0x58          0x59          0x5A          0x5B          0x5C          0x5D          0x5E          0x5F
    I::bcs(A::Relative),
    I::lda(A::IndirectY),
    I::jam(A::Implied),
    I::lax(A::IndirectY),
    I::ldy(A::ZeroPageX),
    I::lda(A::ZeroPageX),
    I::ldx(A::ZeroPageY),
    I::lax(A::ZeroPageY),
    I::clv(A::Implied),
    I::lda(A::AbsoluteY),
    I::tsx(A::Implied),
    I::las(A::AbsoluteY),
    I::ldy(A::AbsoluteX),
    I::lda(A::AbsoluteX),
    I::ldx(A::AbsoluteY),
    I::lax(A::AbsoluteY),
    //  0x60          0x61          0x62          0x63          0x64          0x65          0x66          0x67
    I::cpy(A::Immediate),
    I::cmp(A::IndirectX),
    I::nop(A::Immediate),
    I::dcp(A::IndirectX),
    I::cpy(A::ZeroPage),
    I::cmp(A::ZeroPage),
    I::dec(A::ZeroPage),
    I::dcp(A::ZeroPage),
    I::iny(A::Implied),
    I::cmp(A::Immediate),
    I::dex(A::Implied),
    I::sbx(A::Immediate),
    I::cpy(A::Absolute),
    I::cmp(A::Absolute),
    I::dec(A::Absolute),
    I::dcp(A::Absolute),
    //  0x68          0x69          0x6A          0x6B          0x6C          0x6D          0x6E          0x6F
    I::bne(A::Relative),
    I::cmp(A::IndirectY),
    I::jam(A::Implied),
    I::dcp(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::cmp(A::ZeroPageX),
    I::dec(A::ZeroPageX),
    I::dcp(A::ZeroPageX),
    I::cld(A::Implied),
    I::cmp(A::AbsoluteY),
    I::nop(A::Implied),
    I::dcp(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::cmp(A::AbsoluteX),
    I::dec(A::AbsoluteX),
    I::dcp(A::AbsoluteX),
    //  0x70          0x71          0x72          0x73          0x74          0x75          0x76          0x77
    I::cpx(A::Immediate),
    I::sbc(A::IndirectX),
    I::nop(A::Immediate),
    I::isc(A::IndirectX),
    I::cpx(A::ZeroPage),
    I::sbc(A::ZeroPage),
    I::inc(A::ZeroPage),
    I::isc(A::ZeroPage),
    I::inx(A::Implied),
    I::sbc(A::Immediate),
    I::nop(A::Implied),
    I::sbc(A::Immediate),
    I::cpx(A::Absolute),
    I::sbc(A::Absolute),
    I::inc(A::Absolute),
    I::isc(A::Absolute),
    //  0x78          0x79          0x7A          0x7B          0x7C          0x7D          0x7E          0x7F
    I::beq(A::Relative),
    I::sbc(A::IndirectY),
    I::jam(A::Implied),
    I::isc(A::IndirectY),
    I::nop(A::ZeroPageX),
    I::sbc(A::ZeroPageX),
    I::inc(A::ZeroPageX),
    I::isc(A::ZeroPageX),
    I::sed(A::Implied),
    I::sbc(A::AbsoluteY),
    I::nop(A::Implied),
    I::isc(A::AbsoluteY),
    I::nop(A::AbsoluteX),
    I::sbc(A::AbsoluteX),
    I::inc(A::AbsoluteX),
    I::isc(A::AbsoluteX),
];

#[cfg(test)]
mod tests {
    use crate::cpu::lookup::LOOKUP_TABLE;

    #[rustfmt::skip]
    pub(crate) static CYCLE_TABLE: [u8; 256] = [
        7, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        6, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        2, 6, 0, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
        2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        2, 5, 0, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
        2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    ];

    #[cfg(test)]
    pub(crate) fn total_cycles(opcode: u8, crossed: bool, taken: bool) -> u8 {
        use crate::cpu::{addressing::Addressing as A, mnemonic::Mnemonic};

        let base = CYCLE_TABLE[opcode as usize];
        let inst = LOOKUP_TABLE[opcode as usize];

        let mut extra = 0;

        if matches!(inst.addressing, A::AbsoluteX | A::AbsoluteY | A::IndirectY) && crossed {
            extra += 1;
        }

        if matches!(
            inst.opcode,
            Mnemonic::BCC
                | Mnemonic::BCS
                | Mnemonic::BNE
                | Mnemonic::BEQ
                | Mnemonic::BPL
                | Mnemonic::BMI
                | Mnemonic::BVC
                | Mnemonic::BVS
        ) {
            if taken {
                extra += 1;
                if crossed {
                    extra += 1;
                }
            }
        }

        base + extra
    }

    #[test]
    fn test_cycle() {
        for instr in &LOOKUP_TABLE {
            
        }
    }
}
