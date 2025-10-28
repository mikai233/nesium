use std::fmt::Display;

use crate::cpu::micro_op::MicroOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Addressing {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    XIndexedAbsolute,
    YIndexedAbsolute,
    AbsoluteIndirect,
    ZeroPage,
    XIndexedZeroPage,
    YIndexedZeroPage,
    XIndexedZeroPageIndirect,
    ZeroPageIndirectYIndexed,
    Relative,
}

impl Addressing {
    pub(crate) const fn micro_ops(&self) -> &'static [MicroOp] {
        match self {
            Addressing::Implied => &[MicroOp::Nop],
            Addressing::Immediate => &[MicroOp::FetchOpcode],
            Addressing::ZeroPage => &[MicroOp::FetchAddrLo, MicroOp::ReadZeroPage],
            Addressing::XIndexedZeroPage => &[
                MicroOp::FetchAddrLo,
                MicroOp::AddIndexToAddrLo,
                MicroOp::ReadZeroPage,
            ],
            Addressing::Absolute => &[MicroOp::FetchAddrLo, MicroOp::FetchAddrHi, MicroOp::ReadAbs],
            Addressing::XIndexedAbsolute => &[
                MicroOp::FetchAddrLo,
                MicroOp::FetchAddrHi,
                MicroOp::AddIndexToAddrLo,
                MicroOp::DummyRead, // extra cycle
                MicroOp::CorrectAddrHiOnPageCross,
                MicroOp::ReadAbs,
            ],
            Addressing::AbsoluteIndirect => &[
                MicroOp::FetchAddrLo,
                MicroOp::FetchAddrHi,
                MicroOp::ReadAbs,
                MicroOp::ReadAbs, // high byte
            ],
            Addressing::XIndexedZeroPageIndirect => &[
                MicroOp::FetchAddrLo,
                MicroOp::AddIndexToAddrLo,
                MicroOp::ReadIndirectXLo,
                MicroOp::ReadIndirectXHi,
                MicroOp::ReadAbs,
            ],
            Addressing::ZeroPageIndirectYIndexed => &[
                MicroOp::FetchAddrLo,
                MicroOp::ReadIndirectYLo,
                MicroOp::ReadIndirectYHi,
                MicroOp::AddIndexToAddrLo,
                MicroOp::CorrectAddrHiOnPageCross,
                MicroOp::ReadAbs,
            ],
            Addressing::Relative => &[
                MicroOp::FetchAddrLo,
                MicroOp::AddBranchOffset,
                MicroOp::FixBranchCross,
            ],
            Addressing::Accumulator
            | Addressing::YIndexedAbsolute
            | Addressing::YIndexedZeroPage => &[],
        }
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Addressing::Implied => "implied".fmt(f),
            Addressing::Accumulator => "accumulator".fmt(f),
            Addressing::Immediate => "immediate".fmt(f),
            Addressing::Absolute => "absolute".fmt(f),
            Addressing::XIndexedAbsolute => "x_indexed_absolute".fmt(f),
            Addressing::YIndexedAbsolute => "y_indexed_absolute".fmt(f),
            Addressing::AbsoluteIndirect => "absolute_indirect".fmt(f),
            Addressing::ZeroPage => "zero_page".fmt(f),
            Addressing::XIndexedZeroPage => "x_indexed_zero_page".fmt(f),
            Addressing::YIndexedZeroPage => "y_indexed_zero_page".fmt(f),
            Addressing::XIndexedZeroPageIndirect => "x_indexed_zero_page_indirect".fmt(f),
            Addressing::ZeroPageIndirectYIndexed => "zero_page_indirect_y_indexed".fmt(f),
            Addressing::Relative => "relative".fmt(f),
        }
    }
}
