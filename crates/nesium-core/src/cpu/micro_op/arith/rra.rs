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
// RRA – Rotate Right memory and Add with Carry (undocumented)
// ================================================================

/// Helper: read-modify-write rotate right (ROR on memory)
const fn rra_rmw() -> MicroOp {
    MicroOp {
        name: "rra_rmw_rotate_right",
        micro_fn: |cpu, bus| {
            // 1) old value already in cpu.base_lo
            let old = cpu.base_lo;
            let carry_in = if cpu.p.contains(Status::CARRY) {
                0x80
            } else {
                0
            };

            // 2) write old value back (RMW timing)
            bus.write(cpu.effective_addr, old);

            // 3) rotate: new_bit7 = old_carry, new_bit0 -> carry_out
            let new = (old >> 1) | carry_in;
            let carry_out = old & 0x01 != 0;

            cpu.base_lo = new; // store for ADC
            cpu.p.set_c(carry_out); // new carry = bit0 of old

            // 4) write new value
            bus.write(cpu.effective_addr, new);
        },
    }
}

/// Helper: final ADC using the value left in cpu.base_lo (with carry)
const fn rra_adc() -> MicroOp {
    MicroOp {
        name: "rra_add_with_carry",
        micro_fn: |cpu, _| {
            let mem = cpu.base_lo;
            let a = cpu.a;
            let c = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
            let sum = (a as u16) + (mem as u16) + (c as u16);

            let result = sum as u8;
            let carry_out = sum > 0xFF;

            // Overflow: signed overflow if both operands same sign and result different
            let overflow = ((a ^ result) & (a ^ mem) & 0x80) != 0;

            cpu.a = result;
            cpu.p.set_c(carry_out);
            cpu.p.set_v(overflow);
            cpu.p.set_zn(result);
        },
    }
}

// ================================================================
// 1. Zero Page: RRA $nn   $67   2 bytes, 5 cycles
// ================================================================
pub const fn rra_zero_page() -> Instruction {
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
    const OP4: MicroOp = rra_rmw(); // Cycle 4: write old, write new
    const OP5: MicroOp = rra_adc(); // Cycle 5: ADC A, new

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 2. Zero Page,X: RRA $nn,X   $77   2 bytes, 6 cycles
// ================================================================
pub const fn rra_zero_page_x() -> Instruction {
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
    const OP5: MicroOp = rra_rmw(); // Cycle 5
    const OP6: MicroOp = rra_adc(); // Cycle 6

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 3. Absolute: RRA $nnnn   $6F   3 bytes, 6 cycles
// ================================================================
pub const fn rra_absolute() -> Instruction {
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
    const OP5: MicroOp = rra_rmw(); // Cycle 5
    const OP6: MicroOp = rra_adc(); // Cycle 6

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 4. Absolute,X: RRA $nnnn,X   $7F   3 bytes, 7 cycles
// ================================================================
pub const fn rra_absolute_x() -> Instruction {
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
    const OP6: MicroOp = rra_rmw(); // Cycle 6
    const OP7: MicroOp = rra_adc(); // Cycle 7

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 5. Absolute,Y: RRA $nnnn,Y   $7B   3 bytes, 7 cycles
// ================================================================
pub const fn rra_absolute_y() -> Instruction {
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
    const OP6: MicroOp = rra_rmw();
    const OP7: MicroOp = rra_adc();

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}

// ================================================================
// 6. (Indirect,X): RRA ($nn,X)   $63   2 bytes, 8 cycles
// ================================================================
pub const fn rra_indirect_x() -> Instruction {
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
    const OP7: MicroOp = rra_rmw(); // Cycle 7
    const OP8: MicroOp = rra_adc(); // Cycle 8

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}

// ================================================================
// 7. (Indirect),Y: RRA ($nn),Y   $73   2 bytes, 8 cycles
// ================================================================
pub const fn rra_indirect_y() -> Instruction {
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
    const OP7: MicroOp = rra_rmw(); // Cycle 7
    const OP8: MicroOp = rra_adc(); // Cycle 8

    Instruction {
        opcode: Mnemonic::RRA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7, OP8],
    }
}
