use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
// 1. Immediate: CPY #$nn $C0 2 bytes, 2 cycles
// ================================================================
pub const fn cpy_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::cpy(ReadFrom::Immediate);
    Instruction {
        opcode: Mnemonic::CPY,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: CPY $nn $C4 2 bytes, 3 cycles
// ================================================================
pub const fn cpy_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::cpy(ReadFrom::ZeroPage);
    Instruction {
        opcode: Mnemonic::CPY,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Absolute: CPY $nnnn $CC 3 bytes, 4 cycles
// ================================================================
pub const fn cpy_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp::cpy(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::CPY,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
