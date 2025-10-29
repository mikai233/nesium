use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Absolute,Y: SHA $nnnn,Y   $9F    3 bytes, 5 cycles
//  V = (high byte of base address) + 1
// ================================================================
pub const fn sha_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // low byte of base
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_calc_v",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let v = hi.wrapping_add(1); // V = high + 1
            cpu.tmp = v; // reuse tmp to store V
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            // Note: SHA does NOT add +1 cycle on page cross, but we still need to
            // consume the cycle for timing accuracy. We perform a dummy read from
            // the base address (without Y) to match real 6502 behavior.
            let _ = bus.read(cpu.effective_addr.wrapping_sub(cpu.y as u16));
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_sha",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x & cpu.tmp; // A & X & V
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  2. (Indirect),Y: SHA ($nn),Y   $93    2 bytes, 6 cycles
//  V = [zp] + 1  (low byte of pointer, no Y offset)
// ================================================================
pub const fn sha_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // zero page pointer address
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "read_lo_calc_v",
        micro_fn: |cpu, bus| {
            let lo = bus.read(cpu.tmp as u16); // low byte of base
            let v = lo.wrapping_add(1); // V = [zp] + 1
            cpu.tmp = v; // store V in tmp
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_hi",
        micro_fn: |cpu, bus| {
            let hi = bus.read((cpu.tmp as u16).wrapping_sub(1).wrapping_add(1)); // [zp+1]
            let base = ((hi as u16) << 8) | ((cpu.tmp as u16).wrapping_sub(1));
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            // Dummy read from base (without Y) to consume cycle
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let _ = bus.read(base);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "write_sha",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x & cpu.tmp; // A & X & V
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
