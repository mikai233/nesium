use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  PHP           $08    1 byte, 3 cycles
//  Push P (Processor Status) onto stack:
//  memory[0x0100 + S] = P, then S -= 1
//  No flags or registers affected
// ================================================================
pub const fn php_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "push_p",
        micro_fn: |cpu, bus| {
            // Stack address: 0x0100 + S
            let addr = 0x0100u16 + cpu.s as u16;
            bus.write(addr, cpu.p.bits());
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "decrement_s",
        micro_fn: |cpu, _| {
            cpu.s = cpu.s.wrapping_sub(1);
            // No flags or registers are affected
        },
    };
    Instruction {
        opcode: Mnemonic::PHP,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2, OP3],
    }
}
