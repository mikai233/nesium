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
                tracing::debug!("load from: {:04X}", cpu.effective_addr);
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

    use tracing::{debug, info};

    use crate::cpu::mnemonic::{Mnemonic, tests::InstrTest};

    #[test]
    fn test_lda() {
        let test = InstrTest::new(Mnemonic::LDA);
        for _ in 0..1 {
            // let seed = rand::random();
            let seed = 8360970990284233340;
            debug!("using seed: {}", seed);
            test.run(seed, |instr, verify, cpu, bus| {
                assert_eq!(cpu.a, verify.m);
                if verify.m == 0 {
                    assert!(cpu.p.z());
                } else {
                    assert!(!cpu.p.z());
                }
                if verify.m & 0x80 != 0 {
                    assert!(cpu.p.n())
                } else {
                    assert!(!cpu.p.n())
                }
            });
        }
    }

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
            status::Status,
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
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

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }
    }

    mod test_ldx {
        use crate::cpu::{
            addressing::Addressing, instruction::Instruction, mnemonic::tests::setup,
            status::Status,
        };

        #[test]
        fn test_ldx_immediate() {
            let instr = Instruction::ldx(Addressing::Immediate);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x42;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0x42);
            assert_eq!(cpu.pc, 0x8002);

            // ⚙️ Processor Status
            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldx_zero_page() {
            let instr = Instruction::ldx(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80;
                mem.mem[0x0080] = 0x00; // load zero to trigger Z flag
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0x00);
            assert_eq!(cpu.pc, 0x8002);

            // ⚙️ Processor Status
            assert!(cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldx_zero_page_y() {
            let instr = Instruction::ldx(Addressing::ZeroPageY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x05, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10;
                mem.mem[0x0015] = 0xFF; // 0x10 + Y = 0x15 → 0xFF
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0xFF);
            assert_eq!(cpu.pc, 0x8002);

            // ⚙️ Processor Status
            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE)); // bit7 = 1
        }

        #[test]
        fn test_ldx_absolute() {
            let instr = Instruction::ldx(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20;
                mem.mem[0x8002] = 0x40;
                mem.mem[0x4020] = 0x11;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0x11);
            assert_eq!(cpu.pc, 0x8003);

            // ⚙️ Processor Status
            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldx_absolute_y_no_cross() {
            let instr = Instruction::ldx(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x05, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x20;
                mem.mem[0x2005] = 0x7F; // positive, bit7=0
            });

            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0x7F);
            assert_eq!(cpu.pc, 0x8003);

            // ⚙️ Processor Status
            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldx_absolute_y_cross_page() {
            let instr = Instruction::ldx(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x10, 0xFF, |mem| {
                mem.mem[0x8001] = 0xF8;
                mem.mem[0x8002] = 0x20; // base = $20F8
                mem.mem[0x2108] = 0x80; // bit7=1 → negative flag set
            });

            let cross_page = true;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.x, 0x80);
            assert_eq!(cpu.pc, 0x8003);

            // ⚙️ Processor Status
            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
        }
    }

    mod test_ldy {
        use crate::cpu::{
            addressing::Addressing, instruction::Instruction, mnemonic::tests::setup,
            status::Status,
        };

        #[test]
        fn test_ldy_immediate() {
            let instr = Instruction::ldy(Addressing::Immediate);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x42;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0x42);
            assert_eq!(cpu.pc, 0x8002);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_zero_page() {
            let instr = Instruction::ldy(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80;
                mem.mem[0x0080] = 0x37;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0x37);
            assert_eq!(cpu.pc, 0x8002);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_zero_page_x() {
            let instr = Instruction::ldy(Addressing::ZeroPageX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10;
                mem.mem[0x0015] = 0x55; // 0x10 + X = 0x15
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0x55);
            assert_eq!(cpu.pc, 0x8002);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_absolute() {
            let instr = Instruction::ldy(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20;
                mem.mem[0x8002] = 0x40; // → $4020
                mem.mem[0x4020] = 0x99;
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0x99);
            assert_eq!(cpu.pc, 0x8003);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_absolute_x_no_cross() {
            let instr = Instruction::ldy(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x20; // base = $2000
                mem.mem[0x2005] = 0xA5;
            });

            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0xA5);
            assert_eq!(cpu.pc, 0x8003);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_absolute_x_cross_page() {
            let instr = Instruction::ldy(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x10, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0xF8;
                mem.mem[0x8002] = 0x20; // base = $20F8
                mem.mem[0x2108] = 0xB7; // cross page → +1 cycle
            });

            let cross_page = true;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(cpu.y, 0xB7);
            assert_eq!(cpu.pc, 0x8003);

            assert!(!cpu.p.contains(Status::ZERO));
            assert!(cpu.p.contains(Status::NEGATIVE));
        }

        #[test]
        fn test_ldy_zero_flag() {
            let instr = Instruction::ldy(Addressing::Immediate);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
            });

            cpu.test_clock(&mut bus, &instr);

            assert_eq!(cpu.y, 0x00);
            assert!(cpu.p.contains(Status::ZERO));
            assert!(!cpu.p.contains(Status::NEGATIVE));
        }
    }

    mod test_sax {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        // ================================================================
        // 1. Zero Page
        // ================================================================
        #[test]
        fn test_sax_zero_page() {
            let instr = Instruction::sax(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x0F, 0x33, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x40; // operand = $40
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let expected_val = 0x0F & 0x33;
            assert_eq!(bus.read(0x0040), expected_val);
            assert_eq!(cpu.pc, 0x8002);
        }

        // ================================================================
        // 2. Zero Page, Y
        // ================================================================
        #[test]
        fn test_sax_zero_page_y() {
            let instr = Instruction::sax(Addressing::ZeroPageY);
            let (mut cpu, mut bus) = setup(0x8000, 0xAA, 0x55, 0x04, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20; // base = $20
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let addr = (0x20u8.wrapping_add(cpu.y)) as u16; // 0x24
            let expected_val = 0xAA & 0x55;
            assert_eq!(bus.read(addr), expected_val);
            assert_eq!(cpu.pc, 0x8002);
        }

        // ================================================================
        // 3. Absolute
        // ================================================================
        #[test]
        fn test_sax_absolute() {
            let instr = Instruction::sax(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0xF0, 0x0F, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x40; // → $4000
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let expected_val = 0xF0 & 0x0F;
            assert_eq!(bus.read(0x4000), expected_val);
            assert_eq!(cpu.pc, 0x8003);
        }

        // ================================================================
        // 4. (Indirect,X)
        // ================================================================
        #[test]
        fn test_sax_indirect_x() {
            let instr = Instruction::sax(Addressing::IndirectX);
            let (mut cpu, mut bus) = setup(0x8000, 0xAA, 0x04, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20; // operand = $20
                mem.mem[0x0024] = 0x00;
                mem.mem[0x0025] = 0x40; // → effective = $4000
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let expected_val = 0xAA & 0x04;
            assert_eq!(bus.read(0x4000), expected_val);
            assert_eq!(cpu.pc, 0x8002);
        }
    }

    mod test_sha {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        // ================================================================
        // 1. Absolute,Y
        // ================================================================
        #[test]
        fn test_sha_absolute_y() {
            let instr = Instruction::sha(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0xAA, 0x55, 0x02, 0xFF, |mem| {
                mem.mem[0x8001] = 0x34;
                mem.mem[0x8002] = 0x12; // base = $1234
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let eff_addr = 0x1234u16.wrapping_add(cpu.y as u16); // 0x1236
            let high = ((0x1234 >> 8) as u8).wrapping_add(1); // 0x12 + 1 = 0x13
            let expected_val = cpu.a & cpu.x & high; // 0xAA & 0x55 & 0x13
            assert_eq!(bus.read(eff_addr), expected_val);
            assert_eq!(cpu.pc, 0x8003);
        }

        // ================================================================
        // 2. Indirect,Y
        // ================================================================
        #[test]
        fn test_sha_indirect_y() {
            let instr = Instruction::sha(Addressing::IndirectY);
            let (mut cpu, mut bus) = setup(0x8000, 0xCC, 0x0F, 0x03, 0xFF, |mem| {
                mem.mem[0x8001] = 0x44;
                mem.mem[0x0044] = 0x78; // low byte
                mem.mem[0x0045] = 0x56; // high byte → base = $5678
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let base = 0x5678u16;
            let eff_addr = base.wrapping_add(cpu.y as u16); // 0x567B
            let high = ((base >> 8) as u8).wrapping_add(1); // 0x56 + 1 = 0x57
            let expected_val = cpu.a & cpu.x & high; // 0xCC & 0x0F & 0x57
            assert_eq!(bus.read(eff_addr), expected_val);
            assert_eq!(cpu.pc, 0x8002);
        }
    }

    mod test_shx {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        // ================================================================
        // SHX - Absolute,Y
        // ================================================================
        #[test]
        fn test_shx_absolute_y() {
            let instr = Instruction::shx(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0xAA, 0x03, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20;
                mem.mem[0x8002] = 0x12; // base = $1220
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let eff_addr = 0x1220u16.wrapping_add(cpu.y as u16); // 0x1223
            let high = ((0x1220 >> 8) as u8).wrapping_add(1); // 0x12 + 1 = 0x13
            let expected_val = cpu.x & high; // X & (high+1)
            assert_eq!(bus.read(eff_addr), expected_val);
            assert_eq!(cpu.pc, 0x8003);
        }
    }

    mod test_shy {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        // ================================================================
        // SHY - Absolute,X
        // ================================================================
        #[test]
        fn test_shy_absolute_x() {
            let instr = Instruction::shy(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0xAA, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10;
                mem.mem[0x8002] = 0x20; // base = $2010
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let eff_addr = 0x2010u16.wrapping_add(cpu.x as u16); // 0x2010 + X = 0x2010 (X=0)
            let high = ((0x2010 >> 8) as u8).wrapping_add(1); // 0x20 + 1 = 0x21
            let expected_val = cpu.y & high; // Y & (high+1)
            assert_eq!(bus.read(eff_addr), expected_val);
            assert_eq!(cpu.pc, 0x8003);
        }
    }

    mod test_sta {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        #[test]
        fn test_sta_zero_page() {
            let instr = Instruction::sta(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x5A, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80; // operand = $80
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x0080), 0x5A);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_sta_zero_page_x() {
            let instr = Instruction::sta(Addressing::ZeroPageX);
            let (mut cpu, mut bus) = setup(0x8000, 0x3C, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10; // base = $10
                // effective = $10 + X = $15
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x0015), 0x3C);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_sta_absolute() {
            let instr = Instruction::sta(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x99, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x40; // → $4000
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x4000), 0x99);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_sta_absolute_x() {
            let instr = Instruction::sta(Addressing::AbsoluteX);
            let (mut cpu, mut bus) = setup(0x8000, 0xAB, 0x05, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x20; // base = $2000
                // effective = $2000 + X = $2005 (no extra cycle for stores)
            });

            // store addressing for absolute,X does not add page-cross cycles for STA
            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x2005), 0xAB);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_sta_absolute_y() {
            let instr = Instruction::sta(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0xFE, 0x00, 0x03, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x30; // base = $3000
                // effective = $3000 + Y = $3003
            });

            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x3003), 0xFE);
            assert_eq!(cpu.pc, 0x8003);
        }

        #[test]
        fn test_sta_indirect_x() {
            let instr = Instruction::sta(Addressing::IndirectX);
            let (mut cpu, mut bus) = setup(0x8000, 0x77, 0x04, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x20; // operand = $20
                // pointer = ($20 + X) -> $24/$25
                mem.mem[0x0024] = 0x00;
                mem.mem[0x0025] = 0x50; // effective = $5000
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x5000), 0x77);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_sta_indirect_y() {
            let instr = Instruction::sta(Addressing::IndirectY);
            let (mut cpu, mut bus) = setup(0x8000, 0x22, 0x00, 0x05, 0xFF, |mem| {
                mem.mem[0x8001] = 0x40; // operand = $40
                mem.mem[0x0040] = 0xF8; // low
                mem.mem[0x0041] = 0x20; // high -> base = $20F8
                // effective = base + Y = $20FD
            });
            cpu.opcode = Some(instr.opcode());

            // For stores, page crossing typically does not add an extra cycle.
            let cross_page = false;
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(cross_page, false);
            assert_eq!(executed, expected);

            assert_eq!(bus.read(0x20FD), 0x22);
            assert_eq!(cpu.pc, 0x8002);
        }
    }

    mod test_stx {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        #[test]
        fn test_stx_zero_page() {
            let instr = Instruction::stx(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x5A, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80; // operand = $80
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x0080), 0x5A);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_stx_zero_page_y() {
            let instr = Instruction::stx(Addressing::ZeroPageY);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x03, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10; // base = $10
                // effective = $10 + Y = $13
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x0013), 0x00); // X register is 0x00
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_stx_absolute() {
            let instr = Instruction::stx(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x99, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x40; // → $4000
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x4000), 0x99);
            assert_eq!(cpu.pc, 0x8003);
        }
    }

    mod test_sty {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        #[test]
        fn test_sty_zero_page() {
            let instr = Instruction::sty(Addressing::ZeroPage);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x5A, 0xFF, |mem| {
                mem.mem[0x8001] = 0x80; // operand = $80
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x0080), 0x5A);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_sty_zero_page_x() {
            let instr = Instruction::sty(Addressing::ZeroPageX);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x03, 0x3C, 0xFF, |mem| {
                mem.mem[0x8001] = 0x10; // base = $10
                // effective = $10 + X = $13
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x0013), 0x3C);
            assert_eq!(cpu.pc, 0x8002);
        }

        #[test]
        fn test_sty_absolute() {
            let instr = Instruction::sty(Addressing::Absolute);
            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x99, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x40; // → $4000
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);
            assert_eq!(bus.read(0x4000), 0x99);
            assert_eq!(cpu.pc, 0x8003);
        }
    }
}
