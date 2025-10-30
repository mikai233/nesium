use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// SHY $nnnn,X $9C 3 bytes, 5 cycles
// M = Y & (high_byte_of_base + 1)
// (base = $nnnn, not including X offset)
// ================================================================
pub const fn shy_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2: fetch low byte of base

    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_calc_v",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base address
            let v = hi.wrapping_add(1); // V = H + 1
            cpu.tmp = v; // store V for Y & V
            let base = ((hi as u16) << 8) | cpu.base_lo as u16;
            cpu.effective_addr = base.wrapping_add(cpu.x as u16);
            cpu.incr_pc();
            // Note: SHY does NOT add +1 cycle on page cross, but we still need to
            // consume the cycle for timing accuracy with a dummy read from base.
        },
    };

    const OP4: MicroOp = MicroOp::dummy_read_cross_x(); // Cycle 4: dummy read from base (without X)

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
