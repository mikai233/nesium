use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    // ================================================================
    //  SHS - Store A AND X into Stack Pointer
    // ================================================================
    /// 🕹️ Purpose:
    ///     Stores (A AND X) into the stack pointer (S), and also writes it to memory.
    ///
    /// ⚙️ Operation:
    ///     M, S ← A & X
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn shs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "shs",
            micro_fn: |cpu, bus| {
                let value = cpu.a & cpu.x;
                cpu.s = value;
                bus.write(cpu.effective_addr, value);
                cpu.p.set_zn(value);
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
