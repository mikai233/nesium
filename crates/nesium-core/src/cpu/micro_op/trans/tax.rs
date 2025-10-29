use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::MicroOp,
};

// ================================================================
//  TAX           $AA    1 byte, 2 cycles
//  X = A
//  Sets N and Z based on X
// ================================================================
pub const fn tax_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "transfer_and_flags",
        micro_fn: |cpu, _| {
            cpu.x = cpu.a;
            cpu.p.set_zn(cpu.x);
        },
    };
    Instruction {
        opcode: Mnemonic::TAX,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2],
    }
}
