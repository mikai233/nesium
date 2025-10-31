use crate::cpu::{addressing::Addressing, micro_op::MicroOp, mnemonic::Mnemonic};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Instruction {
    pub(crate) opcode: Mnemonic,
    pub(crate) addressing: Addressing,
    pub(crate) micro_ops: &'static [MicroOp],
}

impl Instruction {
    pub(crate) const fn ldx(addr: Addressing) -> Self {
        Self {
            opcode: Mnemonic::LDX,
            addressing: addr,
            micro_ops: &[],
        }
    }
}
