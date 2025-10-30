use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Absolute: STA $nnnn $8D 3 bytes, 4 cycles
// ================================================================
pub const fn sta_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi(); // Cycle 3
    const OP4: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 2. Absolute,X: STA $nnnn,X $9D 3 bytes, 5 cycles
// ================================================================
pub const fn sta_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x(); // Cycle 3: add X, no page cross penalty
    const OP4: MicroOp = MicroOp::dummy_read_cross_x(); // Cycle 4: dummy read from base
    const OP5: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 3. Absolute,Y: STA $nnnn,Y $99 3 bytes, 5 cycles
// ================================================================
pub const fn sta_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y(); // Cycle 3: add Y, no page cross penalty
    const OP4: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 4: dummy read from base
    const OP5: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 4. Zero Page: STA $nn $85 2 bytes, 3 cycles
// ================================================================
pub const fn sta_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.zp_addr as u16, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 5. Zero Page,X: STA $nn,X $95 2 bytes, 4 cycles
// ================================================================
pub const fn sta_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy(); // Cycle 3: wrap + dummy read
    const OP4: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 6. (Indirect,X): STA ($nn,X) $81 2 bytes, 6 cycles
// ================================================================
pub const fn sta_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy(); // Cycle 3: dummy read
    const OP4: MicroOp = MicroOp::read_indirect_x_lo(); // Cycle 4: read low byte
    const OP5: MicroOp = MicroOp::read_indirect_x_hi(); // Cycle 5: read high byte
    const OP6: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 7. (Indirect),Y: STA ($nn),Y $91 2 bytes, 6 cycles
// ================================================================
pub const fn sta_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page(); // Cycle 3: read low byte from zp
    const OP4: MicroOp = MicroOp::read_indirect_y_hi(); // Cycle 4: read high, add Y, detect cross
    const OP5: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 5: dummy read from base (no Y)
    const OP6: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
