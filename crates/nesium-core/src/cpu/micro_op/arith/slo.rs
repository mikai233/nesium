use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// SLO – Arithmetic Shift Left memory and OR with A (undocumented)
// ================================================================

/// Helper: read-modify-write arithmetic shift left (ASL on memory)
const fn slo_rmw() -> MicroOp {
    MicroOp {
        name: "slo_rmw_shift_left",
        micro_fn: |cpu, bus| {
            // 1) old value already in cpu.base_lo
            let old = cpu.base_lo;

            // 2) write old value back (RMW timing)
            bus.write(cpu.effective_addr, old);

            // 3) shift left: bit7 -> carry, bit0 = 0
            let new = old << 1;
            let carry_out = old & 0x80 != 0;

            cpu.base_lo = new; // store for ORA
            cpu.p.set_c(carry_out); // new carry = bit7 of old

            // 4) write new value
            bus.write(cpu.effective_addr, new);
        },
    }
}

/// Helper: final ORA using the value left in cpu.base_lo
const fn slo_ora() -> MicroOp {
    MicroOp {
        name: "slo_or_with_a",
        micro_fn: |cpu, _| {
            let result = cpu.a | cpu.base_lo;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    }
}

// ================================================================
// 1. Zero Page: SLO $nn   $07   2 bytes, 5 cycles
// ================================================================
pub const fn slo_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    // Cycle 3: read target byte
    const OP3: MicroOp = MicroOp {
        name: "read_zp_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.zp_addr as u16);
            cpu.effective_addr = cpu.zp_addr as u16;
        },
    };
    const OP4: MicroOp = slo_rmw(); // Cycle 4: write old, write new
    const OP5: MicroOp = slo_ora(); // Cycle 5: ORA A, new

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 2. Zero Page,X: SLO $nn,X   $17   2 bytes, 6 cycles
// ================================================================
pub const fn slo_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy(); // dummy + set effective_addr
    // Cycle 4: read target
    const OP4: MicroOp = MicroOp {
        name: "read_zp_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = slo_rmw(); // Cycle 5
    const OP6: MicroOp = slo_ora(); // Cycle 6

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 3. Absolute: SLO $nnnn   $0F   3 bytes, 6 cycles
// ================================================================
pub const fn slo_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    // Cycle 4: read target
    const OP4: MicroOp = MicroOp {
        name: "read_abs_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = slo_rmw(); // Cycle 5
    const OP6: MicroOp = slo_ora(); // Cycle 6

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 4. Absolute,X: SLO $nnnn,X   $1F   3 bytes, 7 cycles
// ================================================================
pub const fn slo_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x();
    const OP4: MicroOp = MicroOp::dummy_read_cross_x(); // +1 if page crossed
    // Cycle 5: read target
    const OP5: MicroOp = MicroOp {
        name: "read_abs_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP6: MicroOp = slo_rmw(); // Cycle 6
    const OP7: MicroOp = slo_ora(); // Cycle 7

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 5. Absolute,Y: SLO $nnnn,Y   $1B   3 bytes, 7 cycles
// ================================================================
pub const fn slo_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y();
    const OP4: MicroOp = MicroOp::dummy_read_cross_y();
    const OP5: MicroOp = MicroOp {
        name: "read_abs_y_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP6: MicroOp = slo_rmw();
    const OP7: MicroOp = slo_ora();

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 6. (Indirect,X): SLO ($nn,X)   $03   2 bytes, 8 cycles
// ================================================================
pub const fn slo_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    // Cycle 6: read target
    const OP6: MicroOp = MicroOp {
        name: "read_ind_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP7: MicroOp = slo_rmw(); // Cycle 7
    const OP8: MicroOp = slo_ora(); // Cycle 8

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}

// ================================================================
// 7. (Indirect),Y: SLO ($nn),Y   $13   2 bytes, 8 cycles
// ================================================================
pub const fn slo_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page(); // reads lo
    const OP4: MicroOp = MicroOp::read_indirect_y_hi(); // reads hi, adds Y
    const OP5: MicroOp = MicroOp::dummy_read_cross_y(); // +1 if cross
    // Cycle 6: read target
    const OP6: MicroOp = MicroOp {
        name: "read_ind_y_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP7: MicroOp = slo_rmw(); // Cycle 7
    const OP8: MicroOp = slo_ora(); // Cycle 8

    Instruction {
        opcode: Mnemonic::SLO,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}
