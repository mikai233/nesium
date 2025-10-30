use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::{MicroOp, ReadFrom},
        status::Status,
    },
};

// ================================================================
// ISC – Increment memory and Subtract with Carry (undocumented)
// ================================================================

/// Helper: read-modify-write increment (used by all RMW modes)
const fn isc_rmw() -> MicroOp {
    MicroOp {
        name: "isc_rmw_increment",
        micro_fn: |cpu, bus| {
            // 1) read old value (already in cpu.base_lo)
            let old = cpu.base_lo;

            // 2) write old value back (mandatory for RMW timing)
            bus.write(cpu.effective_addr, old);

            // 3) increment
            let new = old.wrapping_add(1);
            cpu.base_lo = new; // keep new value for subsequent SBC

            // 4) write new value
            bus.write(cpu.effective_addr, new);
        },
    }
}

/// Helper: final SBC using the value left in cpu.base_lo (with carry)
const fn isc_sbc() -> MicroOp {
    MicroOp {
        name: "isc_subtract_with_carry",
        micro_fn: |cpu, _| {
            let mem = cpu.base_lo;
            let a = cpu.a;
            let c = if cpu.p.contains(Status::CARRY) { 0 } else { 1 };
            let result = a.wrapping_sub(mem).wrapping_sub(c);

            // Carry: NOT borrow (i.e., A >= (mem + !C))
            let borrow = (a as u16) < (mem as u16 + c as u16);
            cpu.p.set_c(!borrow);

            // Overflow: signed overflow detection
            let overflow = ((a ^ result) & (a ^ mem) & 0x80) != 0;
            cpu.p.set_v(overflow);

            cpu.a = result;
            cpu.p.set_zn(result);
        },
    }
}

// ================================================================
// 1. Zero Page: ISC $nn   $E7   2 bytes, 5 cycles
// ================================================================
pub const fn isc_zero_page() -> Instruction {
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
    const OP4: MicroOp = isc_rmw(); // Cycle 4: write old, write new
    const OP5: MicroOp = isc_sbc(); // Cycle 5: SBC A, new

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 2. Zero Page,X: ISC $nn,X   $F7   2 bytes, 6 cycles
// ================================================================
pub const fn isc_zero_page_x() -> Instruction {
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
    const OP5: MicroOp = isc_rmw(); // Cycle 5
    const OP6: MicroOp = isc_sbc(); // Cycle 6

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 3. Absolute: ISC $nnnn   $EF   3 bytes, 6 cycles
// ================================================================
pub const fn isc_absolute() -> Instruction {
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
    const OP5: MicroOp = isc_rmw(); // Cycle 5
    const OP6: MicroOp = isc_sbc(); // Cycle 6

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 4. Absolute,X: ISC $nnnn,X   $FF   3 bytes, 7 cycles
// ================================================================
pub const fn isc_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x(); // sets effective_addr + crossed_page
    const OP4: MicroOp = MicroOp::dummy_read_cross_x(); // +1 cycle if page crossed
    // Cycle 5: read target byte
    const OP5: MicroOp = MicroOp {
        name: "read_abs_x_for_rmw",
        micro_fn: |cpu, bus| {
            cpu.base_lo = bus.read(cpu.effective_addr);
        },
    };
    const OP6: MicroOp = isc_rmw(); // Cycle 6
    const OP7: MicroOp = isc_sbc(); // Cycle 7

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 5. Absolute,Y: ISC $nnnn,Y   $FB   3 bytes, 7 cycles
// ================================================================
pub const fn isc_absolute_y() -> Instruction {
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
    const OP6: MicroOp = isc_rmw();
    const OP7: MicroOp = isc_sbc();

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 6. (Indirect,X): ISC ($nn,X)   $E3   2 bytes, 8 cycles
// ================================================================
pub const fn isc_indirect_x() -> Instruction {
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
    const OP7: MicroOp = isc_rmw(); // Cycle 7
    const OP8: MicroOp = isc_sbc(); // Cycle 8

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}

// ================================================================
// 7. (Indirect),Y: ISC ($nn),Y   $F3   2 bytes, 8 cycles
// ================================================================
pub const fn isc_indirect_y() -> Instruction {
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
    const OP7: MicroOp = isc_rmw(); // Cycle 7
    const OP8: MicroOp = isc_sbc(); // Cycle 8

    Instruction {
        opcode: Mnemonic::ISC,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}
