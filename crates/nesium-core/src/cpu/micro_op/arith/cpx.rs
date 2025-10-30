use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
// 1. Immediate: CPX #$nn $E0 2 bytes, 2 cycles
// ================================================================
pub const fn cpx_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::cpx(ReadFrom::Immediate);
    Instruction {
        opcode: Mnemonic::CPX,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: CPX $nn $E4 2 bytes, 3 cycles
// ================================================================
pub const fn cpx_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::cpx(ReadFrom::ZeroPage);
    Instruction {
        opcode: Mnemonic::CPX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Absolute: CPX $nnnn $EC 3 bytes, 4 cycles
// ================================================================
pub const fn cpx_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp::cpx(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::CPX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}
