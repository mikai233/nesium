use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Absolute: STX $nnnn $8E 3 bytes, 4 cycles
// ================================================================
pub const fn stx_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi(); // Cycle 3
    const OP4: MicroOp = MicroOp {
        name: "write_x",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.x);
        },
    };
    Instruction {
        opcode: Mnemonic::STX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 2. Zero Page: STX $nn $86 2 bytes, 3 cycles
// ================================================================
pub const fn stx_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp {
        name: "write_x",
        micro_fn: |cpu, bus| {
            bus.write(cpu.zp_addr as u16, cpu.x);
        },
    };
    Instruction {
        opcode: Mnemonic::STX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,Y: STX $nn,Y $96 2 bytes, 4 cycles
// ================================================================
pub const fn stx_zero_page_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page_add_y_dummy(); // Cycle 3: wrap + dummy read
    const OP4: MicroOp = MicroOp {
        name: "write_x",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.x);
        },
    };
    Instruction {
        opcode: Mnemonic::STX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
