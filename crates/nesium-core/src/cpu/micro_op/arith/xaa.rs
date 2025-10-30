use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// XAA – ANd X with (A | 0xEE) and memory, store in A (undocumented, unstable)
// ================================================================

/// Helper: final XAA operation using value in cpu.base_lo
const fn xaa_final() -> MicroOp {
    MicroOp {
        name: "xaa_and_magic",
        micro_fn: |cpu, _| {
            // XAA = X & (A | 0xEE) & operand
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & cpu.base_lo;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    }
}

// ================================================================
// 1. Immediate: XAA #$nn   $8B   2 bytes, 2 cycles
// ================================================================
pub const fn xaa_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    // Cycle 2: read immediate and perform XAA
    const OP2: MicroOp = MicroOp {
        name: "xaa_immediate",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.pc);
            cpu.base_lo = operand; // reuse base_lo for consistency
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
            cpu.incr_pc();
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: XAA $nn   $87   2 bytes, 3 cycles
// ================================================================
pub const fn xaa_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    // Cycle 3: read operand and XAA
    const OP3: MicroOp = MicroOp {
        name: "xaa_zp",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.zp_addr as u16);
            cpu.base_lo = operand;
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,Y: XAA $nn,Y   $97   2 bytes, 4 cycles
// ================================================================
pub const fn xaa_zero_page_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_y_dummy(); // sets effective_addr
    // Cycle 4: read operand and XAA
    const OP4: MicroOp = MicroOp {
        name: "xaa_zp_y",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.base_lo = operand;
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 4. Absolute: XAA $nnnn   $8F   3 bytes, 4 cycles
// ================================================================
pub const fn xaa_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    // Cycle 4: read operand and XAA
    const OP4: MicroOp = MicroOp {
        name: "xaa_abs",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.base_lo = operand;
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 5. Absolute,Y: XAA $nnnn,Y   $9B   3 bytes, 4(+p) cycles
// ================================================================
pub const fn xaa_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y();
    const OP4: MicroOp = MicroOp::dummy_read_cross_y(); // +1 if page crossed
    // Cycle 5: read operand and XAA
    const OP5: MicroOp = MicroOp {
        name: "xaa_abs_y",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.base_lo = operand;
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 6. (Indirect,X): XAA ($nn,X)   $83   2 bytes, 6 cycles
// ================================================================
pub const fn xaa_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    // Cycle 6: read operand and XAA
    const OP6: MicroOp = MicroOp {
        name: "xaa_ind_x",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.base_lo = operand;
            let magic = cpu.a | 0xEE;
            let result = cpu.x & magic & operand;
            cpu.a = result;
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::XAA,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
