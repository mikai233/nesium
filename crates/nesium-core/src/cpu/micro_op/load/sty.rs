use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Absolute: STY $nnnn $8C 3 bytes, 4 cycles
// ================================================================
pub const fn sty_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi(); // Cycle 3
    const OP4: MicroOp = MicroOp {
        name: "write_y",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.y);
        },
    };
    Instruction {
        opcode: Mnemonic::STY,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 2. Zero Page: STY $nn $84 2 bytes, 3 cycles
// ================================================================
pub const fn sty_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp {
        name: "write_y",
        micro_fn: |cpu, bus| {
            bus.write(cpu.zp_addr as u16, cpu.y);
        },
    };
    Instruction {
        opcode: Mnemonic::STY,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,X: STY $nn,X $94 2 bytes, 4 cycles
// ================================================================
pub const fn sty_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy(); // Cycle 3: wrap + dummy read
    const OP4: MicroOp = MicroOp {
        name: "write_y",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.y);
        },
    };
    Instruction {
        opcode: Mnemonic::STY,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
