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
                let value = cpu.a & cpu.x & hi.wrapping_add(1);
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
                let value = cpu.x & hi.wrapping_add(1);
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
                let value = cpu.y & hi.wrapping_add(1);
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

    use crate::{
        bus::Bus,
        cpu::mnemonic::{Mnemonic, tests::InstrTest},
    };

    #[test]
    fn test_las() {
        InstrTest::new(Mnemonic::LAS).test(|_, verify, cpu, _| {
            let v = verify.m & verify.cpu.s;
            assert_eq!(cpu.a, v);
            assert_eq!(cpu.x, v);
            assert_eq!(cpu.s, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_lax() {
        InstrTest::new(Mnemonic::LAX).test(|_, verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.a, m);
            assert_eq!(cpu.x, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_lda() {
        InstrTest::new(Mnemonic::LDA).test(|_, verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.a, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_ldx() {
        InstrTest::new(Mnemonic::LDX).test(|_, verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.x, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_ldy() {
        InstrTest::new(Mnemonic::LDY).test(|_, verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.y, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_sax() {
        InstrTest::new(Mnemonic::SAX).test(|_, verify, _, bus| {
            let v = verify.cpu.a & verify.cpu.x;
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sha() {
        InstrTest::new(Mnemonic::SHA).test(|_, verify, _, bus| {
            let v = verify.cpu.a & verify.cpu.x & verify.addr_hi.wrapping_add(1);
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_shx() {
        InstrTest::new(Mnemonic::SHX).test(|_, verify, _, bus| {
            let v = verify.cpu.x & verify.addr_hi.wrapping_add(1);
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_shy() {
        InstrTest::new(Mnemonic::SHY).test(|_, verify, _, bus| {
            let v = verify.cpu.y & verify.addr_hi.wrapping_add(1);
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sta() {
        InstrTest::new(Mnemonic::STA).test(|_, verify, _, bus| {
            let v = verify.cpu.a;
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_stx() {
        InstrTest::new(Mnemonic::STX).test(|_, verify, _, bus| {
            let v = verify.cpu.x;
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sty() {
        InstrTest::new(Mnemonic::STY).test(|_, verify, _, bus| {
            let v = verify.cpu.y;
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }
}
