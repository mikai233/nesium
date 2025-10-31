use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    // ================================================================
    //  LAS — Load A and X from Stack Pointer AND Memory
    // ================================================================
    /// 🕹️ Purpose:
    ///     Loads A and X registers with (SP & M).
    ///
    /// ⚙️ Operation:
    ///     A, X ← SP & M
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
