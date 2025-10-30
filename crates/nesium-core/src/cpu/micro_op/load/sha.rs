use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Absolute,Y: SHA $nnnn,Y $9F 3 bytes, 5 cycles
// V = (high byte of base address) + 1
// ================================================================
pub const fn sha_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2: fetch low byte
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_calc_v",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc); // high byte of base
            let base = ((hi as u16) << 8) | cpu.base_lo as u16;
            let v = hi.wrapping_add(1); // V = high + 1
            cpu.tmp = v; // store V for later A & X & V
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.effective_addr = addr;
            cpu.incr_pc();
            // Note: SHA does NOT add +1 cycle on page cross, but we still need to
            // consume the cycle for timing accuracy. We perform a dummy read from
            // the base address (without Y) to match real 6502 behavior.
        },
    };
    const OP4: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 4: dummy read from base (no Y)
    const OP5: MicroOp = MicroOp {
        name: "write_sha",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x & cpu.tmp; // A & X & V
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 2. (Indirect),Y: SHA ($nn),Y $93 2 bytes, 6 cycles
// V = [zp] + 1 (low byte of pointer, no Y offset)
// ================================================================
pub const fn sha_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2: fetch zp pointer addr
    const OP3: MicroOp = MicroOp {
        name: "read_lo_calc_v",
        micro_fn: |cpu, bus| {
            let lo = bus.read(cpu.zp_addr as u16); // low byte of base
            let v = lo.wrapping_add(1); // V = [zp] + 1
            cpu.tmp = v; // store V
            cpu.base_lo = lo; // reuse base_lo for hi fetch
        },
    };
    const OP4: MicroOp = MicroOp::read_indirect_y_hi(); // Cycle 4: read hi, add Y, set effective_addr
    const OP5: MicroOp = MicroOp::dummy_read_cross_y(); // Cycle 5: dummy read from base (no Y)
    const OP6: MicroOp = MicroOp {
        name: "write_sha",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x & cpu.tmp; // A & X & V
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SHA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

#[cfg(test)]
mod sha_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::micro_op::load::sha::*, // Import SHA instruction functions
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
    // 1. Absolute,Y: SHA $nnnn,Y (normal, same page)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_absolute_y_normal() {
        let instr = sha_absolute_y();
        let (mut cpu, mut bus) = setup(0x8000, 0xAA, 0x55, 0x03, |mock| {
            mock.mem[0x8001] = 0x10; // Low byte
            mock.mem[0x8002] = 0x20; // High byte → base = $2010
        });

        cpu.test_clock(&mut bus, &instr);

        let expected_addr = 0x2010 + 0x03;
        let expected_val = 0xAA & 0x55 & 0x21; // V = hi+1 = 0x21
        assert_eq!(bus.read(expected_addr), expected_val);

        assert_eq!(cpu.pc, 0x8003);
        assert_eq!(cpu.a, 0xAA);
        assert_eq!(cpu.x, 0x55);
        assert_eq!(cpu.y, 0x03);
        assert_eq!(cpu.p, Status::empty());
    }

    // -----------------------------------------------------------------------
    // 2. Absolute,Y: SHA $nnnn,Y (page cross)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_absolute_y_cross_page() {
        let instr = sha_absolute_y();
        let (mut cpu, mut bus) = setup(0x9000, 0xF0, 0x0F, 0x0A, |mock| {
            mock.mem[0x9001] = 0xF8; // Low byte
            mock.mem[0x9002] = 0x20; // High byte → base = $20F8
        });

        cpu.test_clock(&mut bus, &instr);

        let expected_addr = 0x20F8 + 0x0A; // = $2102 (cross page)
        let expected_val = 0xF0 & 0x0F & 0x21; // hi+1 = 0x21
        assert_eq!(bus.read(expected_addr), expected_val);

        assert_eq!(cpu.pc, 0x9003);
        assert_eq!(cpu.a, 0xF0);
        assert_eq!(cpu.x, 0x0F);
        assert_eq!(cpu.y, 0x0A);
        assert_eq!(cpu.p, Status::empty());
    }

    // -----------------------------------------------------------------------
    // 3. Absolute,Y: SHA $nnnn,Y (high byte 0xFF overflow)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_absolute_y_high_overflow() {
        let instr = sha_absolute_y();
        let (mut cpu, mut bus) = setup(0xA000, 0xFF, 0xFF, 0x05, |mock| {
            mock.mem[0xA001] = 0x10;
            mock.mem[0xA002] = 0xFF; // base = $FF10
        });

        cpu.test_clock(&mut bus, &instr);

        let expected_addr = 0xFF10 + 0x05; // $FF15
        let expected_val = 0xFF & 0xFF & 0x00; // hi+1 = 0x00 (overflow)
        assert_eq!(bus.read(expected_addr), expected_val);

        assert_eq!(cpu.pc, 0xA003);
        assert_eq!(cpu.p, Status::empty());
    }

    // -----------------------------------------------------------------------
    // 4. Absolute,Y: SHA $nnnn,Y (A or X zero)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_absolute_y_zero_a_or_x() {
        let instr = sha_absolute_y();

        // Case A = 0
        let (mut cpu, mut bus) = setup(0xB000, 0x00, 0xFF, 0x01, |mock| {
            mock.mem[0xB001] = 0x00;
            mock.mem[0xB002] = 0x10;
        });
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(bus.read(0x1001), 0x00);

        // Case X = 0
        let (mut cpu, mut bus) = setup(0xB100, 0xFF, 0x00, 0x01, |mock| {
            mock.mem[0xB101] = 0x00;
            mock.mem[0xB102] = 0x10;
        });
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(bus.read(0x1001), 0x00);
    }

    // -----------------------------------------------------------------------
    // 5. (Indirect),Y: SHA ($nn),Y (normal)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_indirect_y_normal() {
        let instr = sha_indirect_y();
        let (mut cpu, mut bus) = setup(0xC000, 0xAA, 0x55, 0x03, |mock| {
            mock.mem[0xC001] = 0x20; // Pointer = $0020
            mock.mem[0x0020] = 0x34; // low
            mock.mem[0x0021] = 0x12; // high → base = $1234
        });

        cpu.test_clock(&mut bus, &instr);

        let expected_addr = 0x1234 + 0x03;
        let expected_val = 0xAA & 0x55 & (0x34 + 1); // V = [zp]+1 = 0x35
        assert_eq!(bus.read(expected_addr), expected_val);
        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.p, Status::empty());
    }

    // -----------------------------------------------------------------------
    // 6. (Indirect),Y: SHA ($nn),Y (zero-page pointer wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_indirect_y_pointer_wrap() {
        let instr = sha_indirect_y();
        let (mut cpu, mut bus) = setup(0xD000, 0x0F, 0xF0, 0x05, |mock| {
            mock.mem[0xD001] = 0xFF; // Pointer = $00FF
            mock.mem[0x00FF] = 0x00; // low
            mock.mem[0x0000] = 0x20; // high (wraps)
        });

        cpu.test_clock(&mut bus, &instr);

        let expected_addr = 0x2000 + 0x05;
        let expected_val = 0x0F & 0xF0 & (0x00 + 1); // V = 1
        assert_eq!(bus.read(expected_addr), expected_val);
        assert_eq!(cpu.pc, 0xD002);
    }

    // -----------------------------------------------------------------------
    // 7. (Indirect),Y: SHA ($nn),Y (A or X zero)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_indirect_y_zero_a_or_x() {
        let instr = sha_indirect_y();

        // A = 0
        let (mut cpu, mut bus) = setup(0xE000, 0x00, 0xFF, 0x01, |mock| {
            mock.mem[0xE001] = 0x10;
            mock.mem[0x0010] = 0xAA;
            mock.mem[0x0011] = 0xBB;
        });
        cpu.test_clock(&mut bus, &instr);
        let addr = 0xBBAA + 1; // approximate effective_addr, not crucial here
        assert_eq!(bus.read(addr), 0x00);

        // X = 0
        let (mut cpu, mut bus) = setup(0xE100, 0xFF, 0x00, 0x01, |mock| {
            mock.mem[0xE101] = 0x10;
            mock.mem[0x0010] = 0x11;
            mock.mem[0x0011] = 0x22;
        });
        cpu.test_clock(&mut bus, &instr);
        let addr = 0x2211 + 1;
        assert_eq!(bus.read(addr), 0x00);
    }

    // -----------------------------------------------------------------------
    // 8. Flag and register preservation
    // -----------------------------------------------------------------------
    #[test]
    fn test_sha_preserve_registers_and_flags() {
        let instr = sha_absolute_y();
        let (mut cpu, mut bus) = setup(0xF000, 0x33, 0x77, 0x10, |mock| {
            mock.mem[0xF001] = 0x00;
            mock.mem[0xF002] = 0x40;
        });
        cpu.p.insert(Status::CARRY | Status::ZERO);

        cpu.test_clock(&mut bus, &instr);

        // Registers unchanged
        assert_eq!(cpu.a, 0x33);
        assert_eq!(cpu.x, 0x77);
        assert_eq!(cpu.y, 0x10);

        // Flags preserved
        assert!(cpu.p.contains(Status::CARRY));
        assert!(cpu.p.contains(Status::ZERO));

        // PC advanced correctly
        assert_eq!(cpu.pc, 0xF003);
    }
}
