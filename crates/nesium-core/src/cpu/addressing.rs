use crate::cpu::micro_op::MicroOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn micro_ops(&self) -> &'static [MicroOp] {
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
