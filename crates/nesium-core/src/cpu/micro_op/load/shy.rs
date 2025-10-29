use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  SHY $nnnn,X   $9C    3 bytes, 5 cycles
//  M = Y & (high_byte_of_base + 1)
//  (base = $nnnn, not including X offset)
// ================================================================
pub const fn shy_absolute_x() -> Instruction {
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
        name: "fetch_hi_calc_v",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base address
            let v = hi.wrapping_add(1); // V = H + 1
            cpu.tmp = v; // reuse tmp to store V
            let base = ((hi as u16) << 8) | (cpu.tmp as u16).wrapping_sub(1);
            cpu.effective_addr = base.wrapping_add(cpu.x as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_base",
        micro_fn: |cpu, bus| {
            // Dummy read from base address (without X) to consume cycle
            let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
            let _ = bus.read(base);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_shy",
        micro_fn: |cpu, bus| {
            let result = cpu.y & cpu.tmp; // Y & (H + 1)
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHY,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}
