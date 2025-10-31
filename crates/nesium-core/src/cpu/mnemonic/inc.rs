use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    // ================================================================
    //  DEC - Decrement Memory
    // ================================================================
    /// 🕹️ Purpose:
    ///     Decrements the value at the effective memory address by one.
    ///
    /// ⚙️ Operation:
    ///     M ← M - 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn dec() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "dec",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr).wrapping_sub(1);
                bus.write(cpu.effective_addr, value);
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  DEX - Decrement X Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Decrements the X register by one.
    ///
    /// ⚙️ Operation:
    ///     X ← X - 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn dex() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "dex",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_sub(1);
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  DEY - Decrement Y Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Decrements the Y register by one.
    ///
    /// ⚙️ Operation:
    ///     Y ← Y - 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn dey() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "dey",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_sub(1);
                cpu.p.set_zn(cpu.y);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  INC - Increment Memory
    // ================================================================
    /// 🕹️ Purpose:
    ///     Increments the value at the effective memory address by one.
    ///
    /// ⚙️ Operation:
    ///     M ← M + 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn inc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "inc",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr).wrapping_add(1);
                bus.write(cpu.effective_addr, value);
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  INX - Increment X Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Increments the X register by one.
    ///
    /// ⚙️ Operation:
    ///     X ← X + 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn inx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "inx",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_add(1);
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  INY - Increment Y Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Increments the Y register by one.
    ///
    /// ⚙️ Operation:
    ///     Y ← Y + 1
    ///
    /// 🧩 Flags Affected:
    ///     N (Negative), Z (Zero)
    pub(crate) const fn iny() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "iny",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_add(1);
                cpu.p.set_zn(cpu.y);
            },
        };
        &[OP1]
    }
}
