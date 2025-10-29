use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  SHS $nnnn,Y   $9B    3 bytes, 5 cycles
//  1. S = A & X
//  2. M = S & (high_byte_of_base + 1)
//  (base = $nnnn, not including Y offset)
// ================================================================
pub const fn shs_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // low byte of base address
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_and_s",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base address
            let v = hi.wrapping_add(1); // V = H + 1
            cpu.tmp = v; // store V in tmp

            // Step 1: S = A & X
            cpu.s = cpu.a & cpu.x;

            // Calculate effective address
            let base = ((hi as u16) << 8) | (cpu.tmp as u16).wrapping_sub(1);
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);

            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_base",
        micro_fn: |cpu, bus| {
            // Dummy read from base address (without Y) to consume cycle
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let _ = bus.read(base);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_shs",
        micro_fn: |cpu, bus| {
            // Step 2: M = S & (H + 1)
            let result = cpu.s & cpu.tmp;
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHS,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}
