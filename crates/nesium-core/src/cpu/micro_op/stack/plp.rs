use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
        status::Status,
    },
};

// ================================================================
//  PLP           $28    1 byte, 4 cycles
//  S += 1
//  P = memory[0x0100 + S]
//  Updates all flags in P (including N, Z, C, V, D, I, B)
// ================================================================
pub const fn plp_implied() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "increment_s",
        micro_fn: |cpu, _| {
            cpu.s = cpu.s.wrapping_add(1);
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "read_from_stack",
        micro_fn: |cpu, bus| {
            let addr = 0x0100u16 + cpu.s as u16;
            cpu.tmp = bus.read(addr); // store raw P byte in tmp
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "load_p",
        micro_fn: |cpu, _| {
            // Restore full processor status from stack
            // Note: B flag is typically cleared on PLP (except in IRQ/NMI context)
            // But we load exactly what was pushed.
            cpu.p = Status::from_bits_truncate(cpu.tmp);
        },
    };
    Instruction {
        opcode: Mnemonic::PLP,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
