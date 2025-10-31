use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
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
                cpu.p.set_n(m & 0x80 != 0);
                cpu.p.set_v(m & 0x40 != 0);
            },
        };
        &[OP1]
    }
}
