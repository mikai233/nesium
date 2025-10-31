use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    // ================================================================
    //  LAS — Load A, X, and Stack Pointer from (SP & M)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads A, X, and Stack Pointer with the bitwise AND of
    ///     memory and the current stack pointer.
    ///
    /// ⚙️ Operation:
    ///     A, X, S ← S & M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn las() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "las",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr) & cpu.s;
                cpu.a = value;
                cpu.x = value;
                cpu.s = value;
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  LAX — Load A and X
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads both A and X with the same memory value.
    ///
    /// ⚙️ Operation:
    ///     A, X ← M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn lax() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "lax",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr);
                cpu.a = value;
                cpu.x = value;
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  LDA — Load Accumulator
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads a value from memory into the accumulator (A).
    ///
    /// ⚙️ Operation:
    ///     A ← M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn lda() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "lda",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr);
                cpu.a = value;
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  LDX — Load X Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads a value from memory into the X register.
    ///
    /// ⚙️ Operation:
    ///     X ← M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn ldx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "ldx",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr);
                cpu.x = value;
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  LDY — Load Y Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads a value from memory into the Y register.
    ///
    /// ⚙️ Operation:
    ///     Y ← M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn ldy() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "ldy",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr);
                cpu.y = value;
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SAX — Store A & X (A AND X) into Memory
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores the bitwise AND of A and X into memory.
    ///
    /// ⚙️ Operation:
    ///     M ← A & X
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn sax() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sax",
            micro_fn: |cpu, bus| {
                let value = cpu.a & cpu.x;
                bus.write(cpu.effective_addr, value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SHA — Store A AND X AND (HighByte+1)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores (A & X & (high-byte + 1)) to memory. (Unofficial)
    ///
    /// ⚙️ Operation:
    ///     M ← A & X & (PCH + 1)
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn sha() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sha",
            micro_fn: |cpu, bus| {
                let hi = cpu.base;
                let value = cpu.a & cpu.x & hi;
                bus.write(cpu.effective_addr, value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SHX — Store X AND (HighByte+1)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores (X & (high-byte + 1)) to memory. (Unofficial)
    ///
    /// ⚙️ Operation:
    ///     M ← X & (PCH + 1)
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn shx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "shx",
            micro_fn: |cpu, bus| {
                let hi = cpu.base;
                let value = cpu.x & hi;
                bus.write(cpu.effective_addr, value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SHY — Store Y AND (HighByte+1)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores (Y & (high-byte + 1)) to memory. (Unofficial)
    ///
    /// ⚙️ Operation:
    ///     M ← Y & (PCH + 1)
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn shy() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "shy",
            micro_fn: |cpu, bus| {
                let hi = cpu.base;
                let value = cpu.y & hi;
                bus.write(cpu.effective_addr, value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  STA — Store Accumulator
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores accumulator (A) into memory.
    ///
    /// ⚙️ Operation:
    ///     M ← A
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn sta() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sta",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  STX — Store X Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores X register into memory.
    ///
    /// ⚙️ Operation:
    ///     M ← X
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn stx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "stx",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  STY — Store Y Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores Y register into memory.
    ///
    /// ⚙️ Operation:
    ///     M ← Y
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn sty() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sty",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.y);
            },
        };
        &[OP1]
    }
}

#[cfg(test)]
mod load_tests {

    mod test_las {
        use crate::cpu::{
            addressing::Addressing, instruction::Instruction, mnemonic::tests::setup,
        };

        #[test]
        fn test_las_absolute_y() {
            let instr = Instruction::las(Addressing::AbsoluteY);
            // initial registers: A=0x00, X=0x55, Y=0x03, S=0x21
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x55, 0x03, 0x21, |mock| {
                mock.mem[0x8001] = 0x10; // Low byte of address
                mock.mem[0x8002] = 0x20; // High byte -> base = $2010
                mock.mem[0x2013] = 0xB7; // Memory value at $2010 + Y(3)
            });

            // Execute instruction and measure cycles
            let executed_cycles = cpu.test_clock(&mut bus, &instr);

            // Expected cycles: base cycle count (no cross-page)
            let cross_page = false;
            let branch_taken = false;
            let expected_cycles = instr.cycle().total_cycle(cross_page, branch_taken);
            assert_eq!(executed_cycles, expected_cycles);

            // Validate results: tmp = M & S
            let expected_val = 0xB7 & 0x21;
            assert_eq!(cpu.a, expected_val);
            assert_eq!(cpu.x, expected_val);
            assert_eq!(cpu.s, expected_val);

            // PC should have advanced past the 3-byte Absolute,Y instruction
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_las_absolute_y_cross_page() {
            let instr = Instruction::las(Addressing::AbsoluteY);
            // initial registers: A=0x00, X=0xF0, Y=0x10, S=0x0F
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0xF0, 0x10, 0x0F, |mock| {
                mock.mem[0x8001] = 0xF5; // Low byte
                mock.mem[0x8002] = 0x20; // High byte -> base = $20F5
                // base + Y => $20F5 + 0x10 = $2105 (cross-page)
                mock.mem[0x2105] = 0xAA; // Memory value at cross-page address
            });

            // Execute instruction and measure cycles
            let executed_cycles = cpu.test_clock(&mut bus, &instr);

            // Expected cycles: base + 1 (because of page crossing)
            let cross_page = true;
            let branch_taken = false;
            let expected_cycles = instr.cycle().total_cycle(cross_page, branch_taken);
            assert_eq!(executed_cycles, expected_cycles);

            // Validate results: tmp = M & S
            let expected_val = 0xAA & 0x0F;
            assert_eq!(cpu.a, expected_val);
            assert_eq!(cpu.x, expected_val);
            assert_eq!(cpu.s, expected_val);

            // PC should have advanced past the 3-byte Absolute,Y instruction
            assert_eq!(cpu.pc, 0x8003);
        }
    }

    mod test_lda {
        use crate::cpu::{
            addressing::Addressing, instruction::Instruction, mnemonic::tests::setup,
        };

        #[test]
        fn test_lda_immediate() {
            let instr = Instruction::lda(Addressing::Immediate);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x42;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x42);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_lda_zero_page() {
            let instr = Instruction::lda(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x10, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80;
                mem.mem[0x0080] = 0x37;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x37);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_lda_zero_page_x() {
            let instr = Instruction::lda(Addressing::ZeroPageX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10;
                mem.mem[0x0015] = 0x55; // 0x10 + X = 0x15
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x55);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_lda_absolute() {
            let instr = Instruction::lda(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20;
                mem.mem[0x8002] = 0x40; // → $4020
                mem.mem[0x4020] = 0x99;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x99);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_lda_absolute_x_no_cross() {
            let instr = Instruction::lda(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x20; // base = $2000
                mem.mem[0x2005] = 0xA5;
            });

            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0xA5);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_lda_absolute_x_cross_page() {
            let instr = Instruction::lda(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x10, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0xF8;
                mem.mem[0x8002] = 0x20; // base = $20F8
                mem.mem[0x2108] = 0xB7; // cross page → +1 cycle
            });

            let cross_page = true;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0xB7);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_lda_absolute_y_cross_page() {
            let instr = Instruction::lda(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x0F, 0xFF, |mem| {
                mem.mem[0x8001] = 0xF8;
                mem.mem[0x8002] = 0x20;
                mem.mem[0x2107] = 0xC3; // cross page → +1 cycle
            });

            let cross_page = true;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0xC3);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_lda_indirect_x() {
            let instr = Instruction::lda(Addressing::IndirectX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x04, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20; // operand = $20
                mem.mem[0x0024] = 0x00;
                mem.mem[0x0025] = 0x30; // → effective = $3000
                mem.mem[0x3000] = 0x77;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x77);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_lda_indirect_y_cross_page() {
            let instr = Instruction::lda(Addressing::IndirectY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x0F, 0xFF, |mem| {
                mem.mem[0x8001] = 0x40;
                mem.mem[0x0040] = 0xF8;
                mem.mem[0x0041] = 0x20; // base = $20F8
                mem.mem[0x2107] = 0x5A; // effective + Y → $2107 (cross page)
            });

            let cross_page = true;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.a, 0x5A);
            assert_eq!(cpu.pc, 0x8002);
        }
    }
}
