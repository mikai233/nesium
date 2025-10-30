use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Immediate: LAX #$nn     $AB    2 bytes, 2 cycles
// ================================================================
pub const fn lax_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp {
        name: "fetch_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.pc);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: LAX $nn      $A7    2 bytes, 3 cycles
// ================================================================
pub const fn lax_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.zp_addr as u16);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,Y: LAX $nn,Y  $B7    2 bytes, 4 cycles
// ================================================================
pub const fn lax_zero_page_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page_add_y_dummy();
    const OP4: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: LAX $nnnn     $AF    3 bytes, 4 cycles
// ================================================================
pub const fn lax_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,Y: LAX $nnnn,Y $BF    3 bytes, 4(+p) cycles
// ================================================================
pub const fn lax_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y();
    const OP4: MicroOp = MicroOp::dummy_read_cross_y();
    const OP5: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  6. (Indirect,X): LAX ($nn,X) $A3   2 bytes, 6 cycles
// ================================================================
pub const fn lax_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy();
    const OP4: MicroOp = MicroOp::read_indirect_x_lo();
    const OP5: MicroOp = MicroOp::read_indirect_x_hi();
    const OP6: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            // Cycle 6: Read data from final address and load A/X
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  7. (Indirect),Y: LAX ($nn),Y $B3   2 bytes, 5(+p) cycles
// ================================================================
pub const fn lax_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::read_zero_page();
    const OP4: MicroOp = MicroOp::read_indirect_y_hi();
    const OP5: MicroOp = MicroOp::dummy_read_cross_y();
    const OP6: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            // Read data from final effective address
            let data = bus.read(cpu.effective_addr);
            // Load accumulator and X register with same data
            cpu.a = data;
            cpu.x = data;
            // Update zero/negative flags based on loaded value
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

#[cfg(test)]
mod lax_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::micro_op::load::lax::{
            lax_absolute, lax_absolute_y, lax_immediate, lax_indirect_x, lax_indirect_y,
            lax_zero_page, lax_zero_page_y,
        },
        cpu::{Cpu, status::Status},
    };

    // -----------------------------------------------------------------------
    // Helper: create CPU + Bus with initial state
    // -----------------------------------------------------------------------
    fn setup(pc: u16, a: u8, x: u8, y: u8, mem_setup: impl FnOnce(&mut MockBus)) -> (Cpu, BusImpl) {
        let mut mock = MockBus::default();
        mem_setup(&mut mock);

        let mut cpu = Cpu::new();
        cpu.pc = pc;
        cpu.a = a;
        cpu.x = x;
        cpu.y = y;
        cpu.p = Status::empty();

        let bus = BusImpl::Dynamic(Box::new(mock));
        (cpu, bus)
    }

    // -----------------------------------------------------------------------
    // 1. Immediate: LAX #$nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_immediate() {
        let instr = lax_immediate();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xC001] = 0b1010_1010; // immediate operand
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b1010_1010);
        assert_eq!(cpu.x, 0b1010_1010);
        assert_eq!(cpu.pc, 0xC002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 2. Zero Page: LAX $nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_zero_page() {
        let instr = lax_zero_page();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xC001] = 0x34; // ZP address
            mock.mem[0x0034] = 0b1100_1100; // memory value
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b1100_1100);
        assert_eq!(cpu.x, 0b1100_1100);
        assert_eq!(cpu.pc, 0xC002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 3. Zero Page,Y: LAX $nn,Y
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_zero_page_y() {
        let instr = lax_zero_page_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x05, |mock| {
            mock.mem[0xC001] = 0x30; // base ZP address
            mock.mem[0x0035] = 0b0101_0101; // $30 + Y=5 = $35
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b0101_0101);
        assert_eq!(cpu.x, 0b0101_0101);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 4. Absolute: LAX $nnnn
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_absolute() {
        let instr = lax_absolute();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xC001] = 0x34;
            mock.mem[0xC002] = 0x12; // address $1234
            mock.mem[0x1234] = 0b1111_0000;
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b1111_0000);
        assert_eq!(cpu.x, 0b1111_0000);
        assert_eq!(cpu.pc, 0xC003);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 5. Absolute,Y - No Page Cross: LAX $nnnn,Y
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_absolute_y_no_page_cross() {
        let instr = lax_absolute_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x10, |mock| {
            mock.mem[0xC001] = 0x34;
            mock.mem[0xC002] = 0x12; // base $1234
            mock.mem[0x1244] = 0b0011_1100; // $1234 + Y=$10 = $1244
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b0011_1100);
        assert_eq!(cpu.x, 0b0011_1100);
        assert_eq!(cpu.pc, 0xC003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 6. Absolute,Y - Page Cross: LAX $nnnn,Y
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_absolute_y_page_cross() {
        let instr = lax_absolute_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x01, |mock| {
            mock.mem[0xC001] = 0xFF;
            mock.mem[0xC002] = 0x10; // base $10FF
            mock.mem[0x1100] = 0b1000_0001; // $10FF + Y=1 = $1100 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b1000_0001);
        assert_eq!(cpu.x, 0b1000_0001);
        assert_eq!(cpu.pc, 0xC003);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 7. (Indirect,X): LAX ($nn,X)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_indirect_x() {
        let instr = lax_indirect_x();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x03, 0x00, |mock| {
            mock.mem[0xC001] = 0x40; // ZP pointer base
            // Pointer at $40 + X=3 = $43
            mock.mem[0x0043] = 0x78; // target low byte
            mock.mem[0x0044] = 0x9A; // target high byte -> $9A78
            mock.mem[0x9A78] = 0b0101_1010; // final value
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b0101_1010);
        assert_eq!(cpu.x, 0b0101_1010);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 8. (Indirect),Y - No Page Cross: LAX ($nn),Y
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_indirect_y_no_page_cross() {
        let instr = lax_indirect_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x10, |mock| {
            mock.mem[0xC001] = 0x50; // ZP pointer
            mock.mem[0x0050] = 0x00; // base low
            mock.mem[0x0051] = 0x80; // base high -> $8000
            mock.mem[0x8010] = 0b0011_0011; // $8000 + Y=$10 = $8010
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b0011_0011);
        assert_eq!(cpu.x, 0b0011_0011);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 9. (Indirect),Y - Page Cross: LAX ($nn),Y
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_indirect_y_page_cross() {
        let instr = lax_indirect_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x01, |mock| {
            mock.mem[0xC001] = 0xFF; // ZP pointer (wraps)
            mock.mem[0x00FF] = 0xFF; // base low
            mock.mem[0x0000] = 0x00; // base high (wraps) -> $00FF
            mock.mem[0x0100] = 0b1100_0011; // $00FF + Y=1 = $0100 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0b1100_0011);
        assert_eq!(cpu.x, 0b1100_0011);
        assert_eq!(cpu.pc, 0xC002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 10. Zero Flag Test: LAX result is zero
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_zero_flag() {
        let instr = lax_immediate();
        let (mut cpu, mut bus) = setup(0xC000, 0xFF, 0xFF, 0x00, |mock| {
            mock.mem[0xC001] = 0x00; // immediate zero
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 11. Registers Preserved Test: Verify Y unchanged, only A and X loaded
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_preserves_y_register() {
        let instr = lax_zero_page();
        let initial_y = 0x55;
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, initial_y, |mock| {
            mock.mem[0xC001] = 0x60;
            mock.mem[0x0060] = 0x77;
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x77);
        assert_eq!(cpu.x, 0x77);
        assert_eq!(cpu.y, initial_y); // Y should be unchanged
        assert_eq!(cpu.pc, 0xC002);
    }

    // -----------------------------------------------------------------------
    // 12. (Indirect,X) - Zero Page Wrap: LAX ($nn,X) with $nn+X crossing ZP boundary
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_indirect_x_zp_wrap() {
        let instr = lax_indirect_x();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x01, 0x00, |mock| {
            mock.mem[0xC001] = 0xFF; // ZP base pointer ($FF)
            // $FF + X=1 wraps to $00 (ZP boundary wrap)
            mock.mem[0x0000] = 0x20; // Target address low byte
            mock.mem[0x0001] = 0x30; // Target address high byte -> $3020
            mock.mem[0x3020] = 0xAA; // Value to load
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0xAA);
        assert_eq!(cpu.x, 0xAA);
        assert_eq!(cpu.pc, 0xC002);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 13. Absolute,Y - Zero Y Value: LAX $nnnn,Y with Y=0 (no offset)
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_absolute_y_zero_y() {
        let instr = lax_absolute_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xC001] = 0x34; // Absolute address low byte
            mock.mem[0xC002] = 0x12; // Absolute address high byte -> $1234
            mock.mem[0x1234] = 0x33; // Value at base address (Y=0 means no offset)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x33);
        assert_eq!(cpu.x, 0x33);
        assert_eq!(cpu.pc, 0xC003);
        assert!(!cpu.p.contains(Status::NEGATIVE)); // 0x33 is positive
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page); // No offset, so no page cross
    }

    // -----------------------------------------------------------------------
    // 14. Zero Page,Y - Zero Page Wrap: LAX $nn,Y with $nn+Y crossing ZP boundary
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_zero_page_y_zp_wrap() {
        let instr = lax_zero_page_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x00, 0x20, |mock| {
            mock.mem[0xC001] = 0xF0; // ZP base address ($F0)
            // $F0 + Y=0x20 = 0x110 -> wraps to $10 in ZP
            mock.mem[0x0010] = 0x55; // Value at wrapped address
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x55);
        assert_eq!(cpu.x, 0x55);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE)); // 0x55 is positive
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 15. (Indirect,X) - Uninitialized Pointer High Byte: LAX ($nn,X) with arbitrary high byte
    // -----------------------------------------------------------------------
    #[test]
    fn test_lax_indirect_x_uninit_ptr() {
        let instr = lax_indirect_x();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0x02, 0x00, |mock| {
            mock.mem[0xC001] = 0x7F; // ZP base pointer ($7F)
            // $7F + X=2 = $81 (no wrap)
            mock.mem[0x0081] = 0x00; // Target address low byte
            mock.mem[0x0082] = 0xFF; // Arbitrary high byte (uninitialized-like) -> $FF00
            mock.mem[0xFF00] = 0x00; // Zero value to test flags
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.pc, 0xC002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(cpu.p.contains(Status::ZERO)); // Zero value should set Z flag
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
            mock.mem[0x0000] = 0x11; // $FA + X=5 = $FF + 1 = $00 → low byte: $11
            mock.mem[0x0001] = 0x22; // $00 + 1 = $01 → high byte: $22 → address $2211
            mock.mem[0x2211] = 0x33; // Value: 0x33
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x33);
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
