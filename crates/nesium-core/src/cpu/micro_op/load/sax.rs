use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
// 1. Absolute: SAX $nnnn $8F 3 bytes, 4 cycles
// ================================================================
pub const fn sax_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi(); // Cycle 3
    const OP4: MicroOp = MicroOp {
        name: "write_and",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x;
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SAX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 2. Zero Page: SAX $nn $87 2 bytes, 3 cycles
// ================================================================
pub const fn sax_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp {
        name: "write_and",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x;
            bus.write(cpu.zp_addr as u16, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SAX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,Y: SAX $nn,Y $97 2 bytes, 4 cycles
// ================================================================
pub const fn sax_zero_page_y() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_zero_page_add_y_dummy(); // Cycle 3 (dummy read + wrap)
    const OP4: MicroOp = MicroOp {
        name: "write_and",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x;
            bus.write(cpu.effective_addr, result); // Write to wrapped address
        },
    };
    Instruction {
        opcode: Mnemonic::SAX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 4. (Indirect,X): SAX ($nn,X) $83 2 bytes, 6 cycles
// ================================================================
pub const fn sax_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode(); // Cycle 1
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo(); // Cycle 2
    const OP3: MicroOp = MicroOp::read_indirect_x_dummy(); // Cycle 3 (dummy read)
    const OP4: MicroOp = MicroOp::read_indirect_x_lo(); // Cycle 4
    const OP5: MicroOp = MicroOp::read_indirect_x_hi(); // Cycle 5
    const OP6: MicroOp = MicroOp {
        name: "write_and",
        micro_fn: |cpu, bus| {
            let result = cpu.a & cpu.x;
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::SAX,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

#[cfg(test)]
mod sax_tests {
    use crate::{
        bus::{Bus, BusImpl, mock::MockBus},
        cpu::micro_op::load::sax::*,
        cpu::{Cpu, status::Status},
    };

    // Helper: Initialize CPU + Bus with 64KB memory (0x0000-0xFFFF)
    fn setup(pc: u16, a: u8, x: u8, y: u8, mem_setup: impl FnOnce(&mut MockBus)) -> (Cpu, BusImpl) {
        let mut mock = MockBus::default(); // Uses [u8; 0x10000] after fix
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
    // 1. Zero Page: SAX $nn
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_zero_page() {
        let instr = sax_zero_page();
        let (mut cpu, mut bus) = setup(0xC000, 0b1100_1100, 0b1010_1010, 0x00, |mock| {
            mock.mem[0xC001] = 0x44; // ZP address: $44
            mock.mem[0x0044] = 0x00; // Initial value
        });

        // A & X = 0b11001100 & 0b10101010 = 0x88
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x0044),
            0x88,
            "Zero Page: A & X should be written to $44"
        );
        assert_eq!(cpu.a, 0b1100_1100, "A register should remain unchanged");
        assert_eq!(cpu.x, 0b1010_1010, "X register should remain unchanged");
        assert_eq!(cpu.pc, 0xC002, "PC should increment by 2");
    }

    // -----------------------------------------------------------------------
    // 2. Zero Page,Y: SAX $nn,Y (normal, no wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_zero_page_y_normal() {
        let instr = sax_zero_page_y();
        let (mut cpu, mut bus) = setup(0xC000, 0x3C, 0x33, 0x05, |mock| {
            mock.mem[0xC001] = 0x20; // Base ZP: $20
            mock.mem[0x0025] = 0x00; // $20 + Y=5 = $25 (target)
        });

        // A & X = 0x3C & 0x33 = 0x30
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x0025),
            0x30,
            "Zero Page,Y (normal): A & X should be written to $25"
        );
        assert_eq!(cpu.y, 0x05, "Y register should remain unchanged");
        assert_eq!(cpu.pc, 0xC002, "PC should increment by 2");
    }

    // -----------------------------------------------------------------------
    // 3. Zero Page,Y: SAX $nn,Y (zero-page wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_zero_page_y_wrap() {
        let instr = sax_zero_page_y();
        let (mut cpu, mut bus) = setup(0xC000, 0xF0, 0xCF, 0x10, |mock| {
            mock.mem[0xC001] = 0xF0; // Base ZP: $F0
            mock.mem[0x0000] = 0x00; // $F0 + Y=0x10 = 0x100 → wraps to $00 (target)
        });

        // A & X = 0xF0 & 0xCF = 0xC0 (192)
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x0000),
            0xC0,
            "Zero Page,Y (wrap): A & X should be written to $00"
        );
        assert_eq!(cpu.pc, 0xC002, "PC should increment by 2");
    }

    // -----------------------------------------------------------------------
    // 4. Absolute: SAX $nnnn
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_absolute() {
        let instr = sax_absolute();
        let (mut cpu, mut bus) = setup(0xC000, 0x55, 0x33, 0x00, |mock| {
            mock.mem[0xC001] = 0x78; // Low byte: $78
            mock.mem[0xC002] = 0x12; // High byte: $12 → target $1278
            mock.mem[0x1278] = 0x00; // Initial value
        });

        // A & X = 0x55 & 0x33 = 0x11
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x1278),
            0x11,
            "Absolute: A & X should be written to $1278"
        );
        assert_eq!(cpu.pc, 0xC003, "PC should increment by 3");
    }

    // -----------------------------------------------------------------------
    // 5. (Indirect,X): SAX ($nn,X) (normal)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_indirect_x_normal() {
        let instr = sax_indirect_x();
        let (mut cpu, mut bus) = setup(0xC000, 0x8F, 0x83, 0x02, |mock| {
            mock.mem[0xC001] = 0x30; // ZP base: $30
            mock.mem[0x0032] = 0x45; // $30 + X=2 = $32 → low byte of target ptr
            mock.mem[0x0033] = 0x67; // $32 + 1 = $33 → high byte of target ptr → $6745
            mock.mem[0x6745] = 0x00; // Target address
        });
        // A & X = 0x8F & 0x83 = 0x83 (131)
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x6745),
            0x83,
            "(Indirect,X) normal: A & X should be written to $6745"
        );
        assert_eq!(cpu.x, 0x83, "X register should remain unchanged"); // 修正：原为 0x02
        assert_eq!(cpu.pc, 0xC002, "PC should increment by 2");
    }

    // -----------------------------------------------------------------------
    // 6. (Indirect,X): SAX ($nn,X) (ZP wrap)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_indirect_x_wrap() {
        let instr = sax_indirect_x();
        let (mut cpu, mut bus) = setup(0xC000, 0x3F, 0x55, 0x05, |mock| {
            mock.mem[0xC001] = 0xFB; // ZP base: $FB
            mock.mem[0x0000] = 0x9A; // $FB + X=5 = 0x00 (wrap) → low byte of target ptr
            mock.mem[0x0001] = 0xBC; // $00 + 1 = $01 → high byte of target ptr → $BC9A
            mock.mem[0xBC9A] = 0x00; // Target address
        });
        // A & X = 0x3F & 0x55 = 0x15 (21)
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0xBC9A),
            0x15,
            "(Indirect,X) wrap: A & X should be written to $BC9A"
        );
        assert_eq!(cpu.x, 0x55, "X register should remain unchanged"); // 新增：检查 X 不变
        assert_eq!(cpu.pc, 0xC002, "PC should increment by 2");
    }

    // -----------------------------------------------------------------------
    // 7. Edge Case: A & X = 0 (zero written)
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_zero_result() {
        let instr = sax_zero_page();
        let (mut cpu, mut bus) = setup(0xC000, 0x0F, 0xF0, 0x00, |mock| {
            mock.mem[0xC001] = 0x77; // ZP address: $77
            mock.mem[0x0077] = 0xFF; // Initial value
        });

        // A & X = 0x0F & 0xF0 = 0x00
        cpu.test_clock(&mut bus, &instr);
        assert_eq!(
            bus.read(0x0077),
            0x00,
            "Zero result: 0 should be written to $77"
        );
    }

    // -----------------------------------------------------------------------
    // 8. Register Preservation: A, X, Y unchanged
    // -----------------------------------------------------------------------
    #[test]
    fn test_sax_preserve_registers() {
        let instr = sax_absolute();
        let initial_a = 0xAB;
        let initial_x = 0xCD;
        let initial_y = 0xEF;
        let (mut cpu, mut bus) = setup(0xC000, initial_a, initial_x, initial_y, |mock| {
            mock.mem[0xC001] = 0x11; // Low byte: $11
            mock.mem[0xC002] = 0x22; // High byte: $22 → target $2211
            mock.mem[0x2211] = 0x00; // Initial value
        });

        cpu.test_clock(&mut bus, &instr);
        assert_eq!(cpu.a, initial_a, "A register should be preserved");
        assert_eq!(cpu.x, initial_x, "X register should be preserved");
        assert_eq!(cpu.y, initial_y, "Y register should be preserved");
        assert_eq!(
            bus.read(0x2211),
            initial_a & initial_x,
            "A & X should be written to $2211"
        );
    }
}
