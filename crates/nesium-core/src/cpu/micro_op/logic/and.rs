use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Immediate: AND #$nn     $29    2 bytes, 2 cycles
// ================================================================
pub const fn and_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "and_imm",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Read immediate operand into tmp
            cpu.a &= cpu.tmp;
            cpu.p.set_zn(cpu.a);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: AND $nn      $25    2 bytes, 3 cycles
// ================================================================
pub const fn and_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_addr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Fetch zero-page address into tmp
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.tmp as u16);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,X: AND $nn,X  $35    2 bytes, 4 cycles
// ================================================================
pub const fn and_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_base",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Fetch base address into tmp
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x",
        micro_fn: |cpu, _| {
            cpu.effective_addr = (cpu.tmp as u16).wrapping_add(cpu.x as u16);
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: AND $nnnn     $2D    3 bytes, 4 cycles
// ================================================================
pub const fn and_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Fetch low byte of address
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
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,X: AND $nnnn,X $3D    3 bytes, 4(+1 if page crossed)
// ================================================================
pub const fn and_absolute_x() -> Instruction {
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
        name: "fetch_hi_calc_addr",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.x as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let _ = bus.read(base); // Dummy read from incorrect page
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  6. Absolute,Y: AND $nnnn,Y $39    3 bytes, 4(+1 if page crossed)
// ================================================================
pub const fn and_absolute_y() -> Instruction {
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
        name: "fetch_hi_calc_addr",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base);
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  7. (Indirect,X): AND ($nn,X) $21   2 bytes, 6 cycles
// ================================================================
pub const fn and_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Zero-page pointer address
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x_dummy",
        micro_fn: |cpu, _| {
            let _ = cpu.tmp.wrapping_add(cpu.x); // Dummy cycle for indexing
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_lo",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16);
            cpu.tmp = bus.read(ptr); // Read low byte of effective address
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "read_hi",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16).wrapping_add(1);
            let hi = bus.read(ptr);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  8. (Indirect),Y: AND ($nn),Y $31   2 bytes, 5(+1 if page crossed)
// ================================================================
pub const fn and_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // Zero-page pointer address
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "read_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.tmp as u16); // Read low byte of base address
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_hi_calc_addr",
        micro_fn: |cpu, bus| {
            let hi = bus.read((cpu.tmp as u16).wrapping_add(1));
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "and_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a &= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base);
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::AND,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

#[cfg(test)]
mod and_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::{Cpu, status::Status},
    };

    use super::*;

    // -----------------------------------------------------------------------
    // Helper: create a fresh CPU + Bus with given initial state
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
    // 1. test_and_immediate
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_immediate() {
        let instr = and_immediate();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000, // A init
            0,
            0,
            |mock| {
                mock.mem[0xC001] = 0b1010_1010; // immediate operand
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 2. test_and_zeropage
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_zeropage() {
        let instr = and_zero_page();
        let (mut cpu, mut bus) = setup(0xC000, 0b1111_0000, 0, 0, |mock| {
            mock.mem[0xC001] = 0x34; // ZP address
            mock.mem[0x0034] = 0b1010_1010; // operand
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 3. test_and_zeropage_x
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_zeropage_x() {
        let instr = and_zero_page_x();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0x05, // X = 5
            0,
            |mock| {
                mock.mem[0xC001] = 0x34; // base ZP
                mock.mem[0x0039] = 0b1010_1010; // 0x34 + 0x05 = 0x39
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 4. test_and_absolute
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_absolute() {
        let instr = and_absolute();
        let (mut cpu, mut bus) = setup(0xC000, 0b1111_0000, 0, 0, |mock| {
            mock.mem[0xC001] = 0x34;
            mock.mem[0xC002] = 0x12; // $1234
            mock.mem[0x1234] = 0b1010_1010;
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 5. test_and_absolute_x_no_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_absolute_x_no_page_cross() {
        let instr = and_absolute_x();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0x10, // X = 0x10
            0,
            |mock| {
                mock.mem[0xC001] = 0x34;
                mock.mem[0xC002] = 0x12; // base $1234
                mock.mem[0x1244] = 0b1010_1010; // $1234 + $10 = $1244
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 6. test_and_absolute_x_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_absolute_x_page_cross() {
        let instr = and_absolute_x();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0x01, // X = 1
            0,
            |mock| {
                mock.mem[0xC001] = 0xFF;
                mock.mem[0xC002] = 0x00; // base $00FF
                mock.mem[0x0100] = 0b1010_1010; // $00FF + 1 = $0100 (crosses page)
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, 0b1010_1010);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page); // Page crossed
    }

    // -----------------------------------------------------------------------
    // 7. test_and_absolute_y_no_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_absolute_y_no_page_cross() {
        let instr = and_absolute_y();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0,
            0x20, // Y = 0x20
            |mock| {
                mock.mem[0xC001] = 0x34;
                mock.mem[0xC002] = 0x12; // $1234
                mock.mem[0x1254] = 0b1010_1010; // $1234 + $20 = $1254
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 8. test_and_absolute_y_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_absolute_y_page_cross() {
        let instr = and_absolute_y();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0,
            0x01, // Y = 1
            |mock| {
                mock.mem[0xC001] = 0xFF;
                mock.mem[0xC002] = 0x00; // $00FF
                mock.mem[0x0100] = 0b1010_1010; // $00FF + 1 = $0100
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, 0b1010_1010);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 9. test_and_indirect_x
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_indirect_x() {
        let instr = and_indirect_x();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0x05, // X = 5
            0,
            |mock| {
                mock.mem[0xC001] = 0x34; // ZP pointer base
                let ptr = 0x34u8.wrapping_add(0x05); // 0x39
                mock.mem[ptr as usize] = 0x78; // low  byte of target
                mock.mem[ptr.wrapping_add(1) as usize] = 0x9A; // high byte -> $9A78
                mock.mem[0x9A78] = 0b1010_1010; // operand
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 10. test_and_indirect_y_no_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_indirect_y_no_page_cross() {
        let instr = and_indirect_y();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0,
            0x20, // Y = 0x20
            |mock| {
                mock.mem[0xC001] = 0x50; // ZP pointer
                mock.mem[0x50] = 0x00; // base low
                mock.mem[0x51] = 0x80; // base high -> $8000
                mock.mem[0x8020] = 0b1010_1010; // $8000 + $20 = $8020
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_0000);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 11. test_and_indirect_y_page_cross
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_indirect_y_page_cross() {
        let instr = and_indirect_y();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b1111_0000,
            0,
            0x01, // Y = 1
            |mock| {
                mock.mem[0xC001] = 0xFF; // ZP pointer
                mock.mem[0xFF] = 0xFF; // base low
                mock.mem[0x00] = 0x00; // wraps: high from $00 -> $00FF
                mock.mem[0x0100] = 0b1010_1010; // $00FF + 1 = $0100
            },
        );

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0b1010_1010);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 12. test_and_result_zero_sets_z_flag
    // -----------------------------------------------------------------------
    #[test]
    fn test_and_result_zero_sets_z_flag() {
        let instr = and_immediate();
        let (mut cpu, mut bus) = setup(0xC000, 0x00, 0, 0, |mock| {
            mock.mem[0xC001] = 0xFF;
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(cpu.p.contains(Status::ZERO));
    }
}
