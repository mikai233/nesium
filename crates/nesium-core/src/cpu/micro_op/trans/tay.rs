use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::MicroOp,
};

// ================================================================
//  TAY           $A8    1 byte, 2 cycles
//  Y = A
//  Sets N and Z based on Y
// ================================================================
pub const fn tay_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "transfer_and_flags",
        micro_fn: |cpu, _| {
            cpu.y = cpu.a;
            cpu.p.set_zn(cpu.y);
        },
    };
    Instruction {
        opcode: Mnemonic::TAY,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2],
    }
}
