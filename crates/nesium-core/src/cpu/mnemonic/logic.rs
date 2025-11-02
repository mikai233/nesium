use crate::{
    bus::Bus,
    cpu::{
        micro_op::MicroOp,
        mnemonic::Mnemonic,
        status::{BIT_6, BIT_7},
    },
};

impl Mnemonic {
    // ================================================================
    //  AND - Logical AND
    // ================================================================
    /// 🕹️ Purpose:
    ///     Performs a bitwise AND between the accumulator (A) and memory.
    ///
    /// ⚙️ Operation:
    ///     A ← A & M
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn and() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "and",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  EOR - Exclusive OR
    // ================================================================
    /// 🕹️ Purpose:
    ///     Performs a bitwise exclusive OR between A and memory.
    ///
    /// ⚙️ Operation:
    ///     A ← A ⊕ M
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn eor() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "eor",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a ^= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  ORA - Logical Inclusive OR
    // ================================================================
    /// 🕹️ Purpose:
    ///     Performs a bitwise OR between A and memory.
    ///
    /// ⚙️ Operation:
    ///     A ← A | M
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn ora() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "ora",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a |= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BIT - Bit Test
    // ================================================================
    /// 🕹️ Purpose:
    ///     Tests bits in memory with A, setting flags accordingly.
    ///
    /// ⚙️ Operation:
    ///     A & M → (affects Z only)
    ///     N ← bit7(M), V ← bit6(M)
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), V (Overflow), Z (Zero)
    pub(crate) const fn bit() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bit",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let and = cpu.a & m;
                cpu.p.set_z(and == 0);
                cpu.p.set_n(m & BIT_7 != 0);
                cpu.p.set_v(m & BIT_6 != 0);
            },
        };
        &[OP1]
    }
}

#[cfg(test)]
mod logic_tests {
    use crate::cpu::{
        mnemonic::{Mnemonic, tests::InstrTest},
        status::{BIT_6, BIT_7},
    };

    #[test]
    fn test_and() {
        InstrTest::new(Mnemonic::AND).test(|verify, cpu, _| {
            let v = verify.cpu.a & verify.m;
            assert_eq!(cpu.a, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_eor() {
        InstrTest::new(Mnemonic::EOR).test(|verify, cpu, _| {
            let v = verify.cpu.a ^ verify.m;
            assert_eq!(cpu.a, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_ora() {
        InstrTest::new(Mnemonic::ORA).test(|verify, cpu, _| {
            let v = verify.cpu.a | verify.m;
            assert_eq!(cpu.a, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_bit() {
        InstrTest::new(Mnemonic::BIT).test(|verify, cpu, _| {
            // Z flag is set if (A & M) == 0
            let z = (verify.cpu.a & verify.m) == 0;
            assert_eq!(cpu.p.z(), z);

            // N flag = bit 7 of memory operand
            assert_eq!(cpu.p.n(), verify.m & BIT_7 != 0);

            // V flag = bit 6 of memory operand
            assert_eq!(cpu.p.v(), verify.m & BIT_6 != 0);
        });
    }
}
