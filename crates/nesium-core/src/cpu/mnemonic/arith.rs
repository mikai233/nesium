use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    // ================================================================
    //  ADC - Add with Carry
    // ================================================================
    /// 🕹️ Purpose:
    ///     Adds a memory value and the carry flag to the accumulator.
    ///
    /// ⚙️ Operation:
    ///     A ← A + M + C
    ///
    /// 🧩 Flags Affected:
    ///     N, V, Z, C
    pub(crate) const fn adc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "adc",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let carry = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry as u16;
                let result = sum as u8;
                cpu.p.set_c(sum > 0xFF);
                cpu.p.set_v(((cpu.a ^ result) & (m ^ result) & 0x80) != 0);
                cpu.a = result;
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SBC - Subtract with Carry
    // ================================================================
    /// 🕹️ Purpose:
    ///     Subtracts memory and borrow (1 - Carry) from the accumulator.
    ///
    /// ⚙️ Operation:
    ///     A ← A - M - (1 - C)
    ///
    /// 🧩 Flags Affected:
    ///     N, V, Z, C
    pub(crate) const fn sbc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sbc",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr) ^ 0xFF;
                let carry = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry as u16;
                let result = sum as u8;
                cpu.p.set_c(sum > 0xFF);
                cpu.p.set_v(((cpu.a ^ result) & (!m ^ result) & 0x80) != 0);
                cpu.a = result;
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  CMP - Compare Accumulator
    // ================================================================
    /// 🕹️ Purpose:
    ///     Compares memory with the accumulator (A).
    ///
    /// ⚙️ Operation:
    ///     A - M (affects flags only)
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn cmp() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "cmp",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let result = cpu.a.wrapping_sub(m);
                cpu.p.set_c(cpu.a >= m);
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  CPX - Compare X Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Compares memory with the X register.
    ///
    /// ⚙️ Operation:
    ///     X - M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn cpx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "cpx",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let result = cpu.x.wrapping_sub(m);
                cpu.p.set_c(cpu.x >= m);
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  CPY - Compare Y Register
    // ================================================================
    /// 🕹️ Purpose:
    ///     Compares memory with the Y register.
    ///
    /// ⚙️ Operation:
    ///     Y - M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn cpy() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "cpy",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let result = cpu.y.wrapping_sub(m);
                cpu.p.set_c(cpu.y >= m);
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  ANC - AND + Carry (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Performs A ← A & M, then sets Carry = N.
    ///
    /// ⚙️ Operation:
    ///     A ← A & M
    ///     C ← bit7(A)
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn anc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "anc",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_zn(cpu.a);
                cpu.p.set_c(cpu.a & 0x80 != 0);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  ARR - AND + ROR (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     A ← (A & M) >> 1, with Carry rotation.
    ///
    /// ⚙️ Operation:
    ///     A ← (A & M) >> 1 (with Carry in)
    ///
    /// 🧩 Flags Affected:
    ///     N, V, Z, C
    pub(crate) const fn arr() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "arr",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a &= m;
                let carry_in = if cpu.p.contains(Status::CARRY) {
                    0x80
                } else {
                    0
                };
                let old = cpu.a;
                cpu.a = (cpu.a >> 1) | carry_in;
                cpu.p.set_zn(cpu.a);
                cpu.p.set_c(cpu.a & 0x40 != 0);
                cpu.p.set_v(((old ^ cpu.a) & 0x40) != 0);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  ASR - AND + LSR (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     A ← (A & M) >> 1
    ///
    /// ⚙️ Operation:
    ///     A ← (A & M) >> 1
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn asr() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "asr",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_c(cpu.a & 0x01 != 0);
                cpu.a >>= 1;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SBX - SAX + CMP (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Computes X ← (A & X) - M
    ///
    /// ⚙️ Operation:
    ///     X ← (A & X) - M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn sbx() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sbx",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                let value = (cpu.a & cpu.x).wrapping_sub(m);
                cpu.p.set_c((cpu.a & cpu.x) >= m);
                cpu.x = value;
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  DCP - DEC + CMP (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Decrements memory then compares with A.
    ///
    /// ⚙️ Operation:
    ///     M ← M - 1
    ///     Compare(A, M)
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn dcp() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "dcp",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                m = m.wrapping_sub(1);
                bus.write(cpu.effective_addr, m);
                cpu.p.set_c(cpu.a >= m);
                cpu.p.set_zn(cpu.a.wrapping_sub(m));
            },
        };
        &[OP1]
    }

    // ================================================================
    //  ISC - INC + SBC (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Increments memory then subtracts it from A.
    ///
    /// ⚙️ Operation:
    ///     M ← M + 1
    ///     A ← A - M - (1 - C)
    ///
    /// 🧩 Flags Affected:
    ///     N, V, Z, C
    pub(crate) const fn isc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "isc",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                m = m.wrapping_add(1);
                bus.write(cpu.effective_addr, m);

                let m = m ^ 0xFF;
                let carry = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry as u16;
                let result = sum as u8;
                cpu.p.set_c(sum > 0xFF);
                cpu.p.set_v(((cpu.a ^ result) & (!m ^ result) & 0x80) != 0);
                cpu.a = result;
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  RLA - ROL + AND (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Rotates memory left then ANDs with A.
    ///
    /// ⚙️ Operation:
    ///     M ← (M << 1) | C
    ///     A ← A & M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn rla() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rla",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                let carry_in = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                cpu.p.set_c(m & 0x80 != 0);
                m = (m << 1) | carry_in;
                bus.write(cpu.effective_addr, m);
                cpu.a &= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  RRA - ROR + ADC (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Rotates memory right then adds to A.
    ///
    /// ⚙️ Operation:
    ///     M ← (M >> 1) | (C << 7)
    ///     A ← A + M + C
    ///
    /// 🧩 Flags Affected:
    ///     N, V, Z, C
    pub(crate) const fn rra() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rra",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                let carry_in = if cpu.p.contains(Status::CARRY) {
                    0x80
                } else {
                    0
                };
                cpu.p.set_c(m & 0x01 != 0);
                m = (m >> 1) | carry_in;
                bus.write(cpu.effective_addr, m);

                let carry = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry as u16;
                let result = sum as u8;
                cpu.p.set_c(sum > 0xFF);
                cpu.p.set_v(((cpu.a ^ result) & (m ^ result) & 0x80) != 0);
                cpu.a = result;
                cpu.p.set_zn(result);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SLO - ASL + ORA (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Shifts memory left, then ORs it into A.
    ///
    /// ⚙️ Operation:
    ///     M ← M << 1
    ///     A ← A | M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn slo() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "slo",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                cpu.p.set_c(m & 0x80 != 0);
                m <<= 1;
                bus.write(cpu.effective_addr, m);
                cpu.a |= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  SRE - LSR + EOR (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Shifts memory right, then XORs it with A.
    ///
    /// ⚙️ Operation:
    ///     M ← M >> 1
    ///     A ← A ⊕ M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z, C
    pub(crate) const fn sre() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sre",
            micro_fn: |cpu, bus| {
                let mut m = bus.read(cpu.effective_addr);
                cpu.p.set_c(m & 0x01 != 0);
                m >>= 1;
                bus.write(cpu.effective_addr, m);
                cpu.a ^= m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  XAA - AND + TAX variant (Unofficial)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Combines A and X through AND, then ANDs again with M.
    ///
    /// ⚙️ Operation:
    ///     A ← (A & X) & M
    ///
    /// 🧩 Flags Affected:
    ///     N, Z
    pub(crate) const fn xaa() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "xaa",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a = (cpu.a & cpu.x) & m;
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1]
    }
}
