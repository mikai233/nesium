use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  PLA           $68    1 byte, 4 cycles
//  S += 1
//  A = memory[0x0100 + S]
//  Sets N and Z based on A
// ================================================================
pub const fn pla_implied() -> Instruction {
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
            cpu.tmp = bus.read(addr); // store data in tmp temporarily
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "load_a_and_flags",
        micro_fn: |cpu, _| {
            cpu.a = cpu.tmp;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::PLA,
        addressing: Addressing::Implied,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
