use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    // ================================================================
    // SHS - Store A AND X into Stack Pointer (illegal opcode)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores (A AND X) into the stack pointer (S), and writes a modified value to memory.
    ///
    /// ⚙️ Operation:
    ///     S ← A & X
    ///     M ← S & (high_byte_of_effective_address + 1)
    ///
    /// 🧩 Flags Affected:
    ///     None (status flags are not affected)
    pub(crate) const fn shs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "shs",
            micro_fn: |cpu, bus| {
                let s = cpu.a & cpu.x;
                cpu.s = s;
                let m = s & cpu.base.wrapping_add(1);
                bus.write(cpu.effective_addr, m);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TAX - Transfer Accumulator to X
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the accumulator (A) into the X register.
    ///
    /// ⚙️ Operation:
    ///     X ← A
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn tax() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "tax",
            micro_fn: |cpu, _| {
                cpu.x = cpu.a;
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TAY - Transfer Accumulator to Y
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the accumulator (A) into the Y register.
    ///
    /// ⚙️ Operation:
    ///     Y ← A
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn tay() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "tay",
            micro_fn: |cpu, _| {
                cpu.y = cpu.a;
                cpu.p.set_zn(cpu.y);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TSX - Transfer Stack Pointer to X
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the stack pointer (S) into the X register.
    ///
    /// ⚙️ Operation:
    ///     X ← S
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn tsx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "tsx",
            micro_fn: |cpu, _| {
                cpu.x = cpu.s;
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TXA - Transfer X to Accumulator
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the X register into the accumulator (A).
    ///
    /// ⚙️ Operation:
    ///     A ← X
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn txa() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "txa",
            micro_fn: |cpu, _| {
                cpu.a = cpu.x;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TXS - Transfer X to Stack Pointer
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the X register into the stack pointer (S).
    ///
    /// ⚙️ Operation:
    ///     S ← X
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn txs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "txs",
            micro_fn: |cpu, _| {
                cpu.s = cpu.x;
            },
        };
        &[OP1]
    }

    // ================================================================
    //  TYA - Transfer Y to Accumulator
    // ================================================================
    /// 🕹️ Purpose:
    ///     Transfers the Y register into the accumulator (A).
    ///
    /// ⚙️ Operation:
    ///     A ← Y
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn tya() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "tya",
            micro_fn: |cpu, _| {
                cpu.a = cpu.y;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }
}

#[cfg(test)]
mod trans_tests {
    mod test_shs {
        use crate::{
            bus::Bus,
            cpu::{addressing::Addressing, instruction::Instruction, mnemonic::tests::setup},
        };

        #[test]
        fn test_shs_absolute_y() {
            // SHS $nnnn,Y  →  S = A & X, M = S & (H + 1)
            let instr = Instruction::shs(Addressing::AbsoluteY);
            let (mut cpu, mut bus) = setup(0x8000, 0xAB, 0xCD, 0x00, 0xFF, |mem| {
                mem.mem[0x8001] = 0x00;
                mem.mem[0x8002] = 0x30; // base = $3000, H = 0x30
                // effective = $3000 + Y = $3000 (Y=0 initially)
            });
            let executed = cpu.test_clock(&mut bus, &instr);
            let expected = instr.cycle().total_cycle(false, false);
            assert_eq!(executed, expected);

            let s = cpu.a & cpu.x; // 0xAB & 0xCD = 0x89
            let h_plus_1 = 0x30 + 1; // 0x31
            let value = s & h_plus_1; // 0x89 & 0x31 = 0x01

            assert_eq!(cpu.s, s);
            assert_eq!(bus.read(0x3000), value); // M = value

            assert_eq!(cpu.pc, 0x8003);
        }
    }

    mod test_tax {
        use crate::cpu::{
            addressing::Addressing, instruction::Instruction, mnemonic::tests::setup,
            status::Status,
        };

        #[test]
        fn test_tax_normal() {
            // TAX - Transfer A → X
            // Addressing mode: Implied
            // Case: Normal value (A = 0xAB)
            let instr = Instruction::tax(Addressing::Implied);

            let (mut cpu, mut bus) = setup(0x8000, 0xAB, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8000] = instr.opcode();
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            assert_eq!(executed, instr.cycle().total_cycle(false, false));
            assert_eq!(cpu.x, cpu.a);
            assert_eq!(cpu.p.contains(Status::ZERO), false);
            assert_eq!(cpu.p.contains(Status::NEGATIVE), true);
            assert_eq!(cpu.pc, 0x8001);
        }

        #[test]
        fn test_tax_zero() {
            // TAX - Transfer A → X
            // Addressing mode: Implied
            // Case: Zero value (A = 0x00)
            let instr = Instruction::tax(Addressing::Implied);

            let (mut cpu, mut bus) = setup(0x8000, 0x00, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8000] = instr.opcode();
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            assert_eq!(cpu.x, 0x00);
            assert_eq!(cpu.p.contains(Status::ZERO), true);
            assert_eq!(cpu.p.contains(Status::NEGATIVE), false);
            assert_eq!(cpu.pc, 0x8001);
        }

        #[test]
        fn test_tax_negative() {
            // TAX - Transfer A → X
            // Addressing mode: Implied
            // Case: Negative value (A = 0xFF)
            let instr = Instruction::tax(Addressing::Implied);

            let (mut cpu, mut bus) = setup(0x8000, 0xFF, 0x00, 0x00, 0xFF, |mem| {
                mem.mem[0x8000] = instr.opcode();
            });

            let executed = cpu.test_clock(&mut bus, &instr);
            assert_eq!(cpu.x, 0xFF);
            assert_eq!(cpu.p.contains(Status::ZERO), false);
            assert_eq!(cpu.p.contains(Status::NEGATIVE), true);
            assert_eq!(cpu.pc, 0x8001);
        }
    }
}
