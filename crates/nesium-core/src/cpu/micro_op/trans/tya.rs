use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::MicroOp,
};

// ================================================================
//  TYA           $98    1 byte, 2 cycles
//  A = Y
//  Sets N and Z based on A
// ================================================================
pub const fn tya_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "transfer_and_flags",
        micro_fn: |cpu, _| {
            cpu.a = cpu.y;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::TYA,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2],
    }
}
