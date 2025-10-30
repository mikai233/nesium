use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

pub const fn las_absolute_y() -> Instruction {
    // Cycle 1: opcode already fetched, increment PC to point to low byte
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();

    // Cycle 2: fetch low byte of address, increment PC
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();

    // Cycle 3: fetch high byte of address, add Y, check for page cross
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();

    // Cycle 4: dummy read (only if page boundary was crossed)
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            // When the address crosses a page boundary, the 6502 performs
            // a dummy read from the wrong page before correcting the high byte.
            let addr = (cpu.effective_addr & 0xFF)
                | ((cpu.effective_addr.wrapping_sub(cpu.y as u16)) & 0xFF00);
            let _ = bus.read(addr); // dummy read, result discarded
        },
    };

    // Cycle 5: read final byte from effective address and execute LAS operation
    const OP5: MicroOp = MicroOp {
        name: "read_and_las",
        micro_fn: |cpu, bus| {
            // Read the actual data from the computed effective address
            let data = bus.read(cpu.effective_addr);

            // Perform LAS operation: A, X, S = M & S
            let result = data & cpu.s;
            cpu.a = result;
            cpu.x = result;
            cpu.s = result;

            // Update processor status flags (Z, N)
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::LAS,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
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
}
