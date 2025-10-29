use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  PHA           $48    1 byte, 3 cycles
//  Push A onto stack: memory[0x0100 + S] = A, then S -= 1
//  No flags affected
// ================================================================
pub const fn pha_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "push_a",
        micro_fn: |cpu, bus| {
            // Stack address: 0x0100 + S
            let addr = 0x0100u16 + cpu.s as u16;
            bus.write(addr, cpu.a);
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "decrement_s",
        micro_fn: |cpu, _| {
            cpu.s = cpu.s.wrapping_sub(1);
            // No flags are affected
        },
    };
    Instruction {
        opcode: Mnemonic::PHA,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2, OP3],
    }
}
