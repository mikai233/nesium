use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
// 1. Immediate: SBC #$nn $E9 2 bytes, 2 cycles
// ================================================================
pub const fn sbc_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::sbc(ReadFrom::Immediate);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: SBC $nn $E5 2 bytes, 3 cycles
// ================================================================
pub const fn sbc_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::sbc(ReadFrom::ZeroPage);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,X: SBC $nn,X $F5 2 bytes, 4 cycles
// ================================================================
pub const fn sbc_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy();
    const OP4: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 4. Absolute: SBC $nnnn $ED 3 bytes, 4 cycles
// ================================================================
pub const fn sbc_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 5. Absolute,X: SBC $nnnn,X $FD 3 bytes, 4(+p) cycles
// ================================================================
pub const fn sbc_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x();
    const OP4: MicroOp = MicroOp::dummy_read_cross_x();
    const OP5: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 6. Absolute,Y: SBC $nnnn,Y $F9 3 bytes, 4(+p) cycles
// ================================================================
pub const fn sbc_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y();
    const OP4: MicroOp = MicroOp::dummy_read_cross_y();
    const OP5: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 7. (Indirect,X): SBC ($nn,X) $E1 2 bytes, 6 cycles
// ================================================================
pub const fn sbc_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    const OP6: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 8. (Indirect),Y: SBC ($nn),Y $F1 2 bytes, 5(+p) cycles
// ================================================================
pub const fn sbc_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page();
    const OP4: MicroOp = MicroOp::read_indirect_y_hi();
    const OP5: MicroOp = MicroOp::dummy_read_cross_y();
    const OP6: MicroOp = MicroOp::sbc(ReadFrom::Effective);
    Instruction {
        opcode: Mnemonic::SBC,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
