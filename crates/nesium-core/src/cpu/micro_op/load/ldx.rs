use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Immediate: LDX #$nn $A2 2 bytes, 2 cycles
// ================================================================
pub const fn ldx_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp {
        name: "fetch_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.pc);
            cpu.x = data;
            cpu.p.set_zn(data);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: LDX $nn $A6 2 bytes, 3 cycles
// ================================================================
pub const fn ldx_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.zp_addr as u16);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,Y: LDX $nn,Y $B6 2 bytes, 4 cycles
// ================================================================
pub const fn ldx_zero_page_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page_add_y_dummy(); // Cycle 3 (wrap + dummy read)
    const OP4: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 4. Absolute: LDX $nnnn $AE 3 bytes, 4 cycles
// ================================================================
pub const fn ldx_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi(); // Cycle 3
    const OP4: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 5. Absolute,Y: LDX $nnnn,Y $BE 3 bytes, 4(+p) cycles
// ================================================================
pub const fn ldx_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi_add_y(); // Cycle 3 (add Y, detect page cross)
    const OP4: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 4 (dummy read if crossed)
    const OP5: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

#[cfg(test)]
mod ldx_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::micro_op::load::ldx::*, // Import LDX instruction functions
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
    // 1. Immediate: LDX #$nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_immediate() {
        let instr = ldx_immediate();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xA001] = 0b1010_0101; // Operand: 0xA5
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0xA5);
        assert_eq!(cpu.a, 0x00); // A unchanged
        assert_eq!(cpu.pc, 0xA002);
        assert!(cpu.p.contains(Status::NEGATIVE)); // Bit7 set
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 2. Zero Page: LDX $nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_zero_page() {
        let instr = ldx_zero_page();
        let (mut cpu, mut bus) = setup(0xA000, 0xAB, 0x00, 0xCD, |mock| {
            mock.mem[0xA001] = 0x22; // ZP address: $22
            mock.mem[0x0022] = 0b0010_1100; // Value: 0x2C
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x2C);
        assert_eq!(cpu.a, 0xAB); // A preserved
        assert_eq!(cpu.y, 0xCD); // Y preserved
        assert_eq!(cpu.pc, 0xA002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 3. Zero Page,Y: LDX $nn,Y (normal, no wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_zero_page_y_normal() {
        let instr = ldx_zero_page_y();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x07, |mock| {
            mock.mem[0xA001] = 0x40; // Base ZP: $40
            mock.mem[0x0047] = 0b0100_0011; // $40 + Y=7 = $47
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x43);
        assert_eq!(cpu.y, 0x07); // Y unchanged
        assert_eq!(cpu.pc, 0xA002);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 4. Zero Page,Y: LDX $nn,Y (zero-page wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_zero_page_y_wrap() {
        let instr = ldx_zero_page_y();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x10, |mock| {
            mock.mem[0xA001] = 0xF5; // Base ZP: $F5
            mock.mem[0x0005] = 0xBC; // $F5 + Y=0x10 = 0x105 → wraps to $05
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0xBC);
        assert_eq!(cpu.pc, 0xA002);
        assert!(cpu.p.contains(Status::NEGATIVE)); // 0xBC has bit7 set
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 5. Absolute: LDX $nnnn
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_absolute() {
        let instr = ldx_absolute();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xA001] = 0x78; // Low byte: $78
            mock.mem[0xA002] = 0x12; // High byte: $12 → address $1278
            mock.mem[0x1278] = 0x0F; // Value: 0x0F
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x0F);
        assert_eq!(cpu.pc, 0xA003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 6. Absolute,Y: LDX $nnnn,Y (no page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_absolute_y_no_cross() {
        let instr = ldx_absolute_y();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x0C, |mock| {
            mock.mem[0xA001] = 0x30; // Low byte: $30
            mock.mem[0xA002] = 0x33; // High byte: $33 → base $3330
            mock.mem[0x333C] = 0x55; // $3330 + Y=0xC = $333C (same page)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x55);
        assert_eq!(cpu.pc, 0xA003);
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 7. Absolute,Y: LDX $nnnn,Y (page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_absolute_y_cross() {
        let instr = ldx_absolute_y();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x05, |mock| {
            mock.mem[0xA001] = 0xFB; // Low byte: $FB
            mock.mem[0xA002] = 0x44; // High byte: $44 → base $44FB
            mock.mem[0x4500] = 0xD7; // $44FB + Y=5 = $4500 (page cross)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0xD7);
        assert_eq!(cpu.pc, 0xA003);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 8. Flag: Zero result (Z set)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_zero_flag() {
        let instr = ldx_immediate();
        let (mut cpu, mut bus) = setup(0xA000, 0xAA, 0xBB, 0xCC, |mock| {
            mock.mem[0xA001] = 0x00; // Operand: 0x00
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x00);
        assert!(cpu.p.contains(Status::ZERO));
        assert!(!cpu.p.contains(Status::NEGATIVE));
        assert_eq!(cpu.a, 0xAA); // A preserved
    }

    // -----------------------------------------------------------------------
    // 9. Flag: Negative result (N set)
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_negative_flag() {
        let instr = ldx_zero_page();
        let (mut cpu, mut bus) = setup(0xA000, 0x00, 0x00, 0x00, |mock| {
            mock.mem[0xA001] = 0x80; // ZP address: $80
            mock.mem[0x0080] = 0b1000_0000; // Value: 0x80 (bit7 set)
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(!cpu.p.contains(Status::ZERO));
    }

    // -----------------------------------------------------------------------
    // 10. Register preservation: A and Y unchanged
    // -----------------------------------------------------------------------
    #[test]
    fn test_ldx_preserve_registers() {
        let instr = ldx_absolute_y();
        let initial_a = 0x11;
        let initial_y = 0x22;
        let (mut cpu, mut bus) = setup(0xA000, initial_a, 0x00, initial_y, |mock| {
            mock.mem[0xA001] = 0x60; // Low byte: $60
            mock.mem[0xA002] = 0x70; // High byte: $70 → base $7060
            mock.mem[0x7060 + initial_y as usize] = 0x99; // Value at $7060 + Y=0x22
        });

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.x, 0x99);
        assert_eq!(cpu.a, initial_a); // A preserved
        assert_eq!(cpu.y, initial_y); // Y preserved
    }
}
