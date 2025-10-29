use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::MicroOp,
};

// ================================================================
//  TXS           $9A    1 byte, 2 cycles
//  S = X
//  Does NOT set any flags
// ================================================================
pub const fn txs_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "transfer_to_s",
        micro_fn: |cpu, _| {
            cpu.s = cpu.x;
            // No flags are affected
        },
    };
    Instruction {
        opcode: Mnemonic::TXS,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2],
    }
}
