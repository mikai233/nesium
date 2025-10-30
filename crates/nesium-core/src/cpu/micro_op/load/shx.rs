use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// SHX $nnnn,Y $9E 3 bytes, 5 cycles
// M = X & (high_byte_of_base + 1)
// (base = $nnnn, not including Y offset)
// ================================================================
pub const fn shx_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2: fetch low byte of base

    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_calc_v",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base address
            let v = hi.wrapping_add(1); // V = H + 1
            cpu.tmp = v; // store V for X & V
            let base = ((hi as u16) << 8) | cpu.base_lo as u16;
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);
            cpu.incr_pc();
            // Note: SHX does NOT add +1 cycle on page cross, but we still need to
            // consume the cycle for timing accuracy with a dummy read from base.
        },
    };

    const OP4: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 4: dummy read from base (without Y)

    const OP5: MicroOp = MicroOp {
        name: "write_shx",
        micro_fn: |cpu, bus| {
            let result = cpu.x & cpu.tmp; // X & (H + 1)
            bus.write(cpu.effective_addr, result);
        },
    };

    Instruction {
        opcode: Mnemonic::SHX,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}
