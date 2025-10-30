use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Immediate: LDY #$nn     $A0    2 bytes, 2 cycles
// ================================================================
pub const fn ldy_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_and_ldy",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.pc);
            cpu.y = data;
            cpu.p.set_zn(data);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::LDY,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: LDY $nn      $A4    2 bytes, 3 cycles
// ================================================================
pub const fn ldy_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_addr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "read_and_ldy",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.tmp as u16);
            cpu.y = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDY,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,X: LDY $nn,X  $B4    2 bytes, 4 cycles
// ================================================================
pub const fn ldy_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_base",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x_zp_wrap",
        micro_fn: |cpu, _| {
            cpu.effective_addr = (cpu.tmp as u16 + cpu.x as u16) & 0x00FF;
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_and_ldy",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.y = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDY,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: LDY $nnnn     $AC    3 bytes, 4 cycles
// ================================================================
pub const fn ldy_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_and_ldy",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.y = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDY,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,X: LDY $nnnn,X $BC    3 bytes, 4(+p) cycles
// ================================================================
pub const fn ldy_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_add_x",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.x as u16);
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
            cpu.check_cross_page = true;
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            if cpu.crossed_page {
                let wrong = (cpu.effective_addr & 0xFF)
                    | ((cpu.effective_addr.wrapping_sub(cpu.x as u16)) & 0xFF00);
                let _ = bus.read(wrong);
            }
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "read_and_ldy",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.y = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDY,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

#[cfg(test)]
mod ldy_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::micro_op::load::ldy::*, // Import LDY instruction functions
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
    // 1. Immediate: LDY #$nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_immediate() {
        let instr = ldy_immediate();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xB001] = 0b1101_0011; // Operand: 0xD3
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0xD3);
        assert_eq!(cpu.a, 0x00); // A unchanged
        assert_eq!(cpu.x, 0x00); // X unchanged
        assert_eq!(cpu.pc, 0xB002);
        assert!(cpu.p.contains(Status::NEGATIVE)); // Bit7 set
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 2. Zero Page: LDY $nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_zero_page() {
        let instr = ldy_zero_page();
        let (mut cpu, mut bus) = setup(0xB000, 0x12, 0x34, 0x00, |mock| {
            mock.mem[0xB001] = 0x66; // ZP address: $66
            mock.mem[0x0066] = 0b0011_0101; // Value: 0x35
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x35);
        assert_eq!(cpu.a, 0x12); // A preserved
        assert_eq!(cpu.x, 0x34); // X preserved
        assert_eq!(cpu.pc, 0xB002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 3. Zero Page,X: LDY $nn,X (normal, no wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_zero_page_x_normal() {
        let instr = ldy_zero_page_x();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x09, 0x00, |mock| {
            mock.mem[0xB001] = 0x20; // Base ZP: $20
            mock.mem[0x0029] = 0b0111_0000; // $20 + X=9 = $29
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x70);
        assert_eq!(cpu.x, 0x09); // X unchanged
        assert_eq!(cpu.pc, 0xB002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 4. Zero Page,X: LDY $nn,X (zero-page wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_zero_page_x_wrap() {
        let instr = ldy_zero_page_x();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x15, 0x00, |mock| {
            mock.mem[0xB001] = 0xEE; // Base ZP: $EE
            mock.mem[0x0009] = 0x8C; // $EE + X=0x15 = 0x103 → wraps to $03? Wait: 0xEE + 0x15 = 0x103 → 0x03? No: 0xEE is 238, 0x15 is 21 → 238+21=259 → 259-256=3 → $03. Fix mock:
            mock.mem[0x0003] = 0x8C; // Correct wrapped address: $03
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x8C);
        assert_eq!(cpu.pc, 0xB002);
        assert!(cpu.p.contains(Status::NEGATIVE)); // 0x8C has bit7 set
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 5. Absolute: LDY $nnnn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_absolute() {
        let instr = ldy_absolute();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xB001] = 0x9A; // Low byte: $9A
            mock.mem[0xB002] = 0xBC; // High byte: $BC → address $BC9A
            mock.mem[0xBC9A] = 0x07; // Value: 0x07
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x07);
        assert_eq!(cpu.pc, 0xB003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 6. Absolute,X: LDY $nnnn,X (no page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_absolute_x_no_cross() {
        let instr = ldy_absolute_x();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x0E, 0x00, |mock| {
            mock.mem[0xB001] = 0x40; // Low byte: $40
            mock.mem[0xB002] = 0x22; // High byte: $22 → base $2240
            mock.mem[0x224E] = 0x6D; // $2240 + X=0xE = $224E (same page)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x6D);
        assert_eq!(cpu.pc, 0xB003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 7. Absolute,X: LDY $nnnn,X (page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_absolute_x_cross() {
        let instr = ldy_absolute_x();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x03, 0x00, |mock| {
            mock.mem[0xB001] = 0xFD; // Low byte: $FD
            mock.mem[0xB002] = 0x55; // High byte: $55 → base $55FD
            mock.mem[0x5600] = 0xE1; // $55FD + X=3 = $5600 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0xE1);
        assert_eq!(cpu.pc, 0xB003);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 8. Flag: Zero result (Z set)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_zero_flag() {
        let instr = ldy_immediate();
        let (mut cpu, mut bus) = setup(0xB000, 0xAB, 0xCD, 0xEF, |mock| {
            mock.mem[0xB001] = 0x00; // Operand: 0x00
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x00);
        assert!(cpu.p.contains(Status::ZERO));
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert_eq!(cpu.a, 0xAB); // A preserved
        assert_eq!(cpu.x, 0xCD); // X preserved
    }

    // -----------------------------------------------------------------------
    // 9. Flag: Negative result (N set)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_negative_flag() {
        let instr = ldy_zero_page();
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xB001] = 0x99; // ZP address: $99
            mock.mem[0x0099] = 0b1001_0000; // Value: 0x90 (bit7 set)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0x90);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 10. Register preservation: A and X unchanged
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldy_preserve_registers() {
        let instr = ldy_absolute_x();
        let initial_a = 0x77;
        let initial_x = 0x88;
        let (mut cpu, mut bus) = setup(0xB000, initial_a, initial_x, 0x00, |mock| {
            mock.mem[0xB001] = 0x10; // Low byte: $10
            mock.mem[0xB002] = 0x30; // High byte: $30 → base $3010
            mock.mem[0x3010 + initial_x as usize] = 0xCC; // Value at $3010 + X=0x88
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.y, 0xCC);
        assert_eq!(cpu.a, initial_a); // A preserved
        assert_eq!(cpu.x, initial_x); // X preserved
    }
}
