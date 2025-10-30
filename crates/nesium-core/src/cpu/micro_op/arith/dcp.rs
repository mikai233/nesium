use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::{MicroOp, ReadFrom},
    },
};

// ================================================================
// DCP – Decrement memory and Compare with A (undocumented)
// ================================================================

/// Helper: read-modify-write decrement (used by all RMW modes)
const fn dcp_rmw() -> MicroOp {
    MicroOp {
        name: "dcp_rmw_decrement",
        micro_fn: |cpu, bus| {
            // 1) read old value (already in cpu.base_lo)
            let old = cpu.base_lo;

            // 2) write old value back (mandatory for RMW timing)
            bus.write(cpu.effective_addr, old);

            // 3) decrement
            let new = old.wrapping_sub(1);
            cpu.base_lo = new; // keep new value for subsequent CMP

            // 4) write new value
            bus.write(cpu.effective_addr, new);
        },
    }
}

/// Helper: final CMP using the value left in cpu.base_lo
const fn dcp_cmp() -> MicroOp {
    MicroOp {
        name: "dcp_compare_a",
        micro_fn: |cpu, _| {
            let mem = cpu.base_lo;
            let a = cpu.a;
            let result = a.wrapping_sub(mem);

            cpu.p.set_c(a >= mem); // Carry = NOT borrow
            cpu.p.set_zn(result);
        },
    }
}

// ================================================================
// 1. Zero Page: DCP $nn   $C7   2 bytes, 5 cycles
// ================================================================
pub const fn dcp_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    // Cycle 3: read target byte into base_lo
    const OP3: MicroOp = MicroOp {
        name: "read_zp_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.zp_addr as u16);
            cpu.effective_addr = cpu.zp_addr as u16;
        },
    };
    const OP4: MicroOp = dcp_rmw(); // Cycle 4: write old, write new
    const OP5: MicroOp = dcp_cmp(); // Cycle 5: CMP A, new

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 2. Zero Page,X: DCP $nn,X   $D7   2 bytes, 6 cycles
// ================================================================
pub const fn dcp_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy(); // dummy + set effective_addr
    // Cycle 4: read target byte
    const OP4: MicroOp = MicroOp {
        name: "read_zp_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = dcp_rmw(); // Cycle 5
    const OP6: MicroOp = dcp_cmp(); // Cycle 6

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 3. Absolute: DCP $nnnn   $CF   3 bytes, 6 cycles
// ================================================================
pub const fn dcp_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    // Cycle 4: read target byte
    const OP4: MicroOp = MicroOp {
        name: "read_abs_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = dcp_rmw(); // Cycle 5
    const OP6: MicroOp = dcp_cmp(); // Cycle 6

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 4. Absolute,X: DCP $nnnn,X   $DF   3 bytes, 7 cycles
// ================================================================
pub const fn dcp_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x(); // sets effective_addr + crossed_page
    const OP4: MicroOp = MicroOp::dummy_read_cross_x(); // +1 cycle if page crossed (already counted in base 7)
    // Cycle 5: read target byte
    const OP5: MicroOp = MicroOp {
        name: "read_abs_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP6: MicroOp = dcp_rmw(); // Cycle 6
    const OP7: MicroOp = dcp_cmp(); // Cycle 7

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 5. Absolute,Y: DCP $nnnn,Y   $DB   3 bytes, 7 cycles
// ================================================================
pub const fn dcp_absolute_y() -> Instruction {
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
    const OP6: MicroOp = dcp_rmw();
    const OP7: MicroOp = dcp_cmp();

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 6. (Indirect,X): DCP ($nn,X)   $C3   2 bytes, 8 cycles
// ================================================================
pub const fn dcp_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    // Cycle 6: read target byte
    const OP6: MicroOp = MicroOp {
        name: "read_ind_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP7: MicroOp = dcp_rmw(); // Cycle 7
    const OP8: MicroOp = dcp_cmp(); // Cycle 8

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}

// ================================================================
// 7. (Indirect),Y: DCP ($nn),Y   $D3   2 bytes, 8 cycles
// ================================================================
pub const fn dcp_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page(); // reads lo into base_lo
    const OP4: MicroOp = MicroOp::read_indirect_y_hi(); // reads hi, adds Y, sets effective_addr
    const OP5: MicroOp = MicroOp::dummy_read_cross_y(); // +1 if page cross
    // Cycle 6: read target byte
    const OP6: MicroOp = MicroOp {
        name: "read_ind_y_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP7: MicroOp = dcp_rmw(); // Cycle 7
    const OP8: MicroOp = dcp_cmp(); // Cycle 8

    Instruction {
        opcode: Mnemonic::DCP,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}
