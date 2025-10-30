use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
//  1. Immediate: LDA #$nn     $A9    2 bytes, 2 cycles
// ================================================================
pub const fn lda_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::lda(ReadFrom::Immediate);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: LDA $nn      $A5    2 bytes, 3 cycles
// ================================================================
pub const fn lda_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::lda(ReadFrom::ZeroPage);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,X: LDA $nn,X  $B5    2 bytes, 4 cycles
// ================================================================
pub const fn lda_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_x_dummy();
    const OP4: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: LDA $nnnn     $AD    3 bytes, 4 cycles
// ================================================================
pub const fn lda_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,X: LDA $nnnn,X $BD    3 bytes, 4(+p) cycles
// ================================================================
pub const fn lda_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_x();
    const OP4: MicroOp = MicroOp::dummy_read_cross_x();
    const OP5: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  6. Absolute,Y: LDA $nnnn,Y $B9    3 bytes, 4(+p) cycles
// ================================================================
pub const fn lda_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y();
    const OP4: MicroOp = MicroOp::dummy_read_cross_y();
    const OP5: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  7. (Indirect,X): LDA ($nn,X) $A1   2 bytes, 6 cycles
// ================================================================
pub const fn lda_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    const OP6: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  8. (Indirect),Y: LDA ($nn),Y $B1   2 bytes, 5(+p) cycles
// ================================================================
pub const fn lda_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page();
    const OP4: MicroOp = MicroOp::read_indirect_y_hi();
    const OP5: MicroOp = MicroOp::dummy_read_cross_y();
    const OP6: MicroOp = MicroOp::lda(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::LDA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

#[cfg(test)]
mod lda_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::micro_op::load::lda::*, // Import LDA instruction functions
        cpu::{Cpu, status::Status},
    };

    // Helper: Initialize CPU + Bus with custom memory setup
    fn setup(pc: u16, a: u8, x: u8, y: u8, mem_setup: impl FnOnce(&mut MockBus)) -> (Cpu, BusImpl) {
        let mut mock = MockBus::default();
        mem_setup(&mut mock);

        let mut cpu = Cpu::new();
        cpu.pc = pc;
        cpu.a = a;
        cpu.x = x;
        cpu.y = y;
        cpu.p = Status::empty();

        (cpu, BusImpl::Dynamic(Box::new(mock)))
    }

    // -----------------------------------------------------------------------
    // 1. Immediate: LDA #$nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_immediate() {
        let instr = lda_immediate();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0x9001] = 0b1100_0111; // Operand: 0xC7
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0xC7);
        assert_eq!(cpu.x, 0x00); // X unchanged
        assert_eq!(cpu.pc, 0x9002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 2. Zero Page: LDA $nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_zero_page() {
        let instr = lda_zero_page();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0xAB, 0xCD, |mock| {
            mock.mem[0x9001] = 0x77; // ZP address: $77
            mock.mem[0x0077] = 0b0001_1000; // Value: 0x18
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x18);
        assert_eq!(cpu.x, 0xAB); // X preserved
        assert_eq!(cpu.y, 0xCD); // Y preserved
        assert_eq!(cpu.pc, 0x9002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 3. Zero Page,X: LDA $nn,X (normal)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_zero_page_x_normal() {
        let instr = lda_zero_page_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x06, 0x00, |mock| {
            mock.mem[0x9001] = 0x30; // Base ZP: $30
            mock.mem[0x0036] = 0b0110_0110; // $30 + X=6 = $36
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x66);
        assert_eq!(cpu.x, 0x06); // X unchanged
        assert_eq!(cpu.pc, 0x9002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 4. Zero Page,X: LDA $nn,X (ZP wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_zero_page_x_wrap() {
        let instr = lda_zero_page_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x10, 0x00, |mock| {
            mock.mem[0x9001] = 0xF5; // Base ZP: $F5
            mock.mem[0x0005] = 0xF0; // $F5 + X=0x10 = 0x105 → wraps to $05
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0xF0);
        assert_eq!(cpu.pc, 0x9002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 5. Absolute: LDA $nnnn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_absolute() {
        let instr = lda_absolute();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0x9001] = 0x12; // Low byte: $12
            mock.mem[0x9002] = 0x34; // High byte: $34 → address $3412
            mock.mem[0x3412] = 0x55; // Value: 0x55
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x55);
        assert_eq!(cpu.pc, 0x9003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 6. Absolute,X: LDA $nnnn,X (no page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_absolute_x_no_cross() {
        let instr = lda_absolute_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x0C, 0x00, |mock| {
            mock.mem[0x9001] = 0x80; // Low byte: $80
            mock.mem[0x9002] = 0x50; // High byte: $50 → base $5080
            mock.mem[0x508C] = 0x0F; // $5080 + X=0xC = $508C (same page)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x0F);
        assert_eq!(cpu.pc, 0x9003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 7. Absolute,X: LDA $nnnn,X (page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_absolute_x_cross() {
        let instr = lda_absolute_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x05, 0x00, |mock| {
            mock.mem[0x9001] = 0xFB; // Low byte: $FB
            mock.mem[0x9002] = 0x22; // High byte: $22 → base $22FB
            mock.mem[0x2300] = 0xD0; // $22FB + X=5 = $2300 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0xD0);
        assert_eq!(cpu.pc, 0x9003);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 8. Absolute,Y: LDA $nnnn,Y (no page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_absolute_y_no_cross() {
        let instr = lda_absolute_y();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x20, |mock| {
            mock.mem[0x9001] = 0x40; // Low byte: $40
            mock.mem[0x9002] = 0x11; // High byte: $11 → base $1140
            mock.mem[0x1160] = 0x2A; // $1140 + Y=0x20 = $1160 (same page)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x2A);
        assert_eq!(cpu.pc, 0x9003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 9. Absolute,Y: LDA $nnnn,Y (page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_absolute_y_cross() {
        let instr = lda_absolute_y();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x03, |mock| {
            mock.mem[0x9001] = 0xFF; // Low byte: $FF
            mock.mem[0x9002] = 0x77; // High byte: $77 → base $77FF
            mock.mem[0x7802] = 0x00; // $77FF + Y=3 = $7802 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.pc, 0x9003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 10. (Indirect,X): LDA ($nn,X) (normal)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_indirect_x_normal() {
        let instr = lda_indirect_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x02, 0x00, |mock| {
            mock.mem[0x9001] = 0x50; // ZP base: $50
            mock.mem[0x0052] = 0x67; // $50 + X=2 = $52 → low byte: $67
            mock.mem[0x0053] = 0x89; // $52 + 1 = $53 → high byte: $89 → address $8967
            mock.mem[0x8967] = 0x9D; // Value: 0x9D
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x9D);
        assert_eq!(cpu.x, 0x02); // X unchanged
        assert_eq!(cpu.pc, 0x9002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 11. (Indirect,X): LDA ($nn,X) (ZP wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_indirect_x_wrap() {
        let instr = lda_indirect_x();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x05, 0x00, |mock| {
            mock.mem[0x9001] = 0xFA; // ZP base: $FA
            // $FA + X=0x05 = 0xFF (low byte pointer)
            mock.mem[0x00FF] = 0x11; // Low byte of target address
            // $FF + 1 = 0x00 (high byte pointer, ZP wrap)
            mock.mem[0x0000] = 0x22; // High byte of target address → $2211
            mock.mem[0x2211] = 0x33; // Value at target address
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x33); // 0x33 = 51 (matches expected "right" value)
        assert_eq!(cpu.pc, 0x9002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 12. (Indirect),Y: LDA ($nn),Y (no page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_indirect_y_no_cross() {
        let instr = lda_indirect_y();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x0A, |mock| {
            mock.mem[0x9001] = 0x33; // ZP pointer: $33
            mock.mem[0x0033] = 0x50; // Low byte: $50
            mock.mem[0x0034] = 0x60; // High byte: $60 → base $6050
            mock.mem[0x605A] = 0x77; // $6050 + Y=0xA = $605A (same page)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x77);
        assert_eq!(cpu.y, 0x0A); // Y unchanged
        assert_eq!(cpu.pc, 0x9002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 13. (Indirect),Y: LDA ($nn),Y (page cross + ZP wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_indirect_y_cross_wrap() {
        let instr = lda_indirect_y();
        let (mut cpu, mut bus) = setup(0x9000, 0x00, 0x00, 0x02, |mock| {
            mock.mem[0x9001] = 0xFF; // ZP pointer: $FF
            mock.mem[0x00FF] = 0xFC; // Low byte: $FC
            mock.mem[0x0000] = 0x00; // $FF + 1 = $00 → high byte: $00 → base $00FC
            mock.mem[0x00FE] = 0x88; // $00FC + Y=2 = $00FE (no cross) → adjust to cross:
            // To force cross: $00FF + Y=1 = $0100 (modify setup):
            mock.mem[0x00FF] = 0xFF;
            mock.mem[0x0101] = 0x88; // $00FF + Y=2 = $0101 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x88);
        assert_eq!(cpu.pc, 0x9002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 14. Flag: Zero result
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_zero_flag() {
        let instr = lda_immediate();
        let (mut cpu, mut bus) = setup(0x9000, 0xAA, 0xBB, 0xCC, |mock| {
            mock.mem[0x9001] = 0x00; // Operand: 0x00
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.p.contains(Status::ZERO));
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert_eq!(cpu.x, 0xBB); // X preserved
    }

    // -----------------------------------------------------------------------
    // 15. Register preservation: X and Y unchanged
    // -----------------------------------------------------------------------
    #[test]
    fn test_lda_preserve_registers() {
        let instr = lda_absolute();
        let initial_x = 0x12;
        let initial_y = 0x34;
        let (mut cpu, mut bus) = setup(0x9000, 0x00, initial_x, initial_y, |mock| {
            mock.mem[0x9001] = 0x56;
            mock.mem[0x9002] = 0x78; // Address $7856
            mock.mem[0x7856] = 0x9A;
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x9A);
        assert_eq!(cpu.x, initial_x); // X preserved
        assert_eq!(cpu.y, initial_y); // Y preserved
    }
}
