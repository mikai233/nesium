use crate::cpu::addressing::Addressing as A;
use crate::cpu::instruction::Instruction as I;

// Short aliases for addressing modes (to keep the 16x16 table readable)
const IMP: A = A::Implied;
const ACC: A = A::Accumulator;
const IMM: A = A::Immediate;
const REL: A = A::Relative;
const ZP: A = A::ZeroPage;
const ZPX: A = A::ZeroPageX;
const ZPY: A = A::ZeroPageY;
const ABS: A = A::Absolute;
const ABX: A = A::AbsoluteX;
const ABY: A = A::AbsoluteY;
const IND: A = A::Indirect;
const INX: A = A::IndirectX;
const INY: A = A::IndirectY;

macro_rules! op {
    ($ins:ident, $addr:ident) => {
        I::$ins($addr)
    };
}

#[rustfmt::skip]
pub(crate) static LOOKUP_TABLE: [I; 256] = [
    // 0               1               2               3               4               5               6               7
    // 8               9               A               B               C               D               E               F

    // 0x00
    op!(brk, IMP), op!(ora, INX), op!(jam, IMP), op!(slo, INX), op!(nop, ZP),  op!(ora, ZP),  op!(asl, ZP),  op!(slo, ZP),
    op!(php, IMP), op!(ora, IMM), op!(asl, ACC), op!(anc, IMM), op!(nop, ABS), op!(ora, ABS), op!(asl, ABS), op!(slo, ABS),

    // 0x10
    op!(bpl, REL), op!(ora, INY), op!(jam, IMP), op!(slo, INY), op!(nop, ZPX), op!(ora, ZPX), op!(asl, ZPX), op!(slo, ZPX),
    op!(clc, IMP), op!(ora, ABY), op!(nop, IMP), op!(slo, ABY), op!(nop, ABX), op!(ora, ABX), op!(asl, ABX), op!(slo, ABX),

    // 0x20
    op!(jsr, ABS), op!(and, INX), op!(jam, IMP), op!(rla, INX), op!(bit, ZP),  op!(and, ZP),  op!(rol, ZP),  op!(rla, ZP),
    op!(plp, IMP), op!(and, IMM), op!(rol, ACC), op!(anc, IMM), op!(bit, ABS), op!(and, ABS), op!(rol, ABS), op!(rla, ABS),

    // 0x30
    op!(bmi, REL), op!(and, INY), op!(jam, IMP), op!(rla, INY), op!(nop, ZPX), op!(and, ZPX), op!(rol, ZPX), op!(rla, ZPX),
    op!(sec, IMP), op!(and, ABY), op!(nop, IMP), op!(rla, ABY), op!(nop, ABX), op!(and, ABX), op!(rol, ABX), op!(rla, ABX),

    // 0x40
    op!(rti, IMP), op!(eor, INX), op!(jam, IMP), op!(sre, INX), op!(nop, ZP),  op!(eor, ZP),  op!(lsr, ZP),  op!(sre, ZP),
    op!(pha, IMP), op!(eor, IMM), op!(lsr, ACC), op!(asr, IMM), op!(jmp, ABS), op!(eor, ABS), op!(lsr, ABS), op!(sre, ABS),

    // 0x50
    op!(bvc, REL), op!(eor, INY), op!(jam, IMP), op!(sre, INY), op!(nop, ZPX), op!(eor, ZPX), op!(lsr, ZPX), op!(sre, ZPX),
    op!(cli, IMP), op!(eor, ABY), op!(nop, IMP), op!(sre, ABY), op!(nop, ABX), op!(eor, ABX), op!(lsr, ABX), op!(sre, ABX),

    // 0x60
    op!(rts, IMP), op!(adc, INX), op!(jam, IMP), op!(rra, INX), op!(nop, ZP),  op!(adc, ZP),  op!(ror, ZP),  op!(rra, ZP),
    op!(pla, IMP), op!(adc, IMM), op!(ror, ACC), op!(arr, IMM), op!(jmp, IND), op!(adc, ABS), op!(ror, ABS), op!(rra, ABS),

    // 0x70
    op!(bvs, REL), op!(adc, INY), op!(jam, IMP), op!(rra, INY), op!(nop, ZPX), op!(adc, ZPX), op!(ror, ZPX), op!(rra, ZPX),
    op!(sei, IMP), op!(adc, ABY), op!(nop, IMP), op!(rra, ABY), op!(nop, ABX), op!(adc, ABX), op!(ror, ABX), op!(rra, ABX),

    // 0x80
    op!(nop, IMM), op!(sta, INX), op!(nop, IMM), op!(sax, INX), op!(sty, ZP),  op!(sta, ZP),  op!(stx, ZP),  op!(sax, ZP),
    op!(dey, IMP), op!(nop, IMM), op!(txa, IMP), op!(xaa, IMM), op!(sty, ABS), op!(sta, ABS), op!(stx, ABS), op!(sax, ABS),

    // 0x90
    op!(bcc, REL), op!(sta, INY), op!(jam, IMP), op!(sha, INY), op!(sty, ZPX), op!(sta, ZPX), op!(stx, ZPY), op!(sax, ZPY),
    op!(tya, IMP), op!(sta, ABY), op!(txs, IMP), op!(shs, ABY), op!(shy, ABX), op!(sta, ABX), op!(shx, ABY), op!(sha, ABY),

    // 0xA0
    op!(ldy, IMM), op!(lda, INX), op!(ldx, IMM), op!(lax, INX), op!(ldy, ZP),  op!(lda, ZP),  op!(ldx, ZP),  op!(lax, ZP),
    op!(tay, IMP), op!(lda, IMM), op!(tax, IMP), op!(lax, IMM), op!(ldy, ABS), op!(lda, ABS), op!(ldx, ABS), op!(lax, ABS),

    // 0xB0
    op!(bcs, REL), op!(lda, INY), op!(jam, IMP), op!(lax, INY), op!(ldy, ZPX), op!(lda, ZPX), op!(ldx, ZPY), op!(lax, ZPY),
    op!(clv, IMP), op!(lda, ABY), op!(tsx, IMP), op!(las, ABY), op!(ldy, ABX), op!(lda, ABX), op!(ldx, ABY), op!(lax, ABY),

    // 0xC0
    op!(cpy, IMM), op!(cmp, INX), op!(nop, IMM), op!(dcp, INX), op!(cpy, ZP),  op!(cmp, ZP),  op!(dec, ZP),  op!(dcp, ZP),
    op!(iny, IMP), op!(cmp, IMM), op!(dex, IMP), op!(sbx, IMM), op!(cpy, ABS), op!(cmp, ABS), op!(dec, ABS), op!(dcp, ABS),

    // 0xD0
    op!(bne, REL), op!(cmp, INY), op!(jam, IMP), op!(dcp, INY), op!(nop, ZPX), op!(cmp, ZPX), op!(dec, ZPX), op!(dcp, ZPX),
    op!(cld, IMP), op!(cmp, ABY), op!(nop, IMP), op!(dcp, ABY), op!(nop, ABX), op!(cmp, ABX), op!(dec, ABX), op!(dcp, ABX),

    // 0xE0
    op!(cpx, IMM), op!(sbc, INX), op!(nop, IMM), op!(isc, INX), op!(cpx, ZP),  op!(sbc, ZP),  op!(inc, ZP),  op!(isc, ZP),
    op!(inx, IMP), op!(sbc, IMM), op!(nop, IMP), op!(sbc, IMM), op!(cpx, ABS), op!(sbc, ABS), op!(inc, ABS), op!(isc, ABS),

    // 0xF0
    op!(beq, REL), op!(sbc, INY), op!(jam, IMP), op!(isc, INY), op!(nop, ZPX), op!(sbc, ZPX), op!(inc, ZPX), op!(isc, ZPX),
    op!(sed, IMP), op!(sbc, ABY), op!(nop, IMP), op!(isc, ABY), op!(nop, ABX), op!(sbc, ABX), op!(inc, ABX), op!(isc, ABX),
];
