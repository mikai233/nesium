use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
// 1. Immediate: ANC #$nn $0B 2 bytes, 2 cycles (undocumented)
// ================================================================
pub const fn anc_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::anc(ReadFrom::Immediate);
    Instruction {
        opcode: Mnemonic::ANC,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}
