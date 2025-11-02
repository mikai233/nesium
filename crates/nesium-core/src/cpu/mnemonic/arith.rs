use crate::{
    bus::Bus,
    cpu::{
        micro_op::MicroOp,
        mnemonic::Mnemonic,
        status::{BIT_5, BIT_6, BIT_7, Status},
    },
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
                let carry_in = if cpu.p.c() { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry_in as u16;

                // Binary mode (default)
                let mut result = sum as u8;
                let mut carry_out = sum > 0xFF;

                // Decimal mode (BCD correction)
                if cpu.p.d() {
                    let mut lo = (cpu.a & 0x0F) + (m & 0x0F) + carry_in;
                    let mut hi = (cpu.a >> 4) + (m >> 4);
                    if lo > 9 {
                        lo = lo + 6;
                        hi += 1;
                    }
                    if hi > 9 {
                        hi = hi + 6;
                    }
                    result = ((hi << 4) | (lo & 0x0F)) & 0xFF;
                    carry_out = hi > 15;
                }

                // Set flags
                cpu.p.set_c(carry_out);
                cpu.p.set_v((!(cpu.a ^ m) & (cpu.a ^ result) & BIT_7) != 0);
                cpu.a = result;
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
                cpu.p.set_c(cpu.a & BIT_7 != 0);
            },
        };
        &[OP1]
    }

    /// NV-BDIZC
    /// ✓✓----✓✓
    ///
    /// ARR - "AND" Accumulator then Rotate Right
    /// Operation: (A ∧ M) / 2 → A
    ///
    /// The undocumented ARR instruction performs a bit-by-bit "AND" operation of the
    /// accumulator and memory, then shifts the result right 1 bit with bit 0 shifted
    /// into the carry and carry shifted into bit 7. It then stores the result back in
    /// the accumulator.
    ///
    /// If bit 7 of the result is on, then the N flag is set, otherwise it is reset.
    /// The instruction sets the Z flag if the result is 0; otherwise it resets Z.
    ///
    /// The V and C flags depends on the Decimal Mode Flag:
    ///
    /// In decimal mode, the V flag is set if bit 6 is different than the original
    /// data's bit 6, otherwise the V flag is reset. The C flag is set if
    /// (operand & 0xF0) + (operand & 0x10) is greater than 0x50, otherwise the C
    /// flag is reset.
    ///
    /// In binary mode, the V flag is set if bit 6 of the result is different than
    /// bit 5 of the result, otherwise the V flag is reset. The C flag is set if the
    /// result in the accumulator has bit 6 on, otherwise it is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | ARR #$nn                 | $6B*   | 2         | 2
    ///
    /// *Undocumented.
    pub(crate) const fn arr() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "arr",
            micro_fn: |cpu, bus| {
                let m = bus.read(cpu.effective_addr);
                cpu.a &= m;
                let carry_in = if cpu.p.c() { BIT_7 } else { 0 };
                let old = cpu.a;
                cpu.a = (cpu.a >> 1) | carry_in;
                cpu.p.set_n(cpu.a & BIT_7 != 0);
                cpu.p.set_z(cpu.a == 0);
                if cpu.p.d() {
                    // Decimal mode
                    cpu.p.set_v((old & BIT_6) != (cpu.a & BIT_6));
                    let c_calc = (m & 0xF0).wrapping_add(m & 0x10);
                    cpu.p.set_c(c_calc > 0x50);
                } else {
                    // Binary mode
                    cpu.p.set_v((cpu.a & BIT_6 != 0) ^ (cpu.a & BIT_5 != 0));
                    cpu.p.set_c(cpu.a & BIT_6 != 0);
                }
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
                let m = bus.read(cpu.effective_addr);
                let carry_in = if cpu.p.c() { 1 } else { 0 };

                // NOTE:
                // SBC performs A = A - M - (1 - C)
                // which is equivalent to: A + (~M) + C
                let value = (!m) as u16;
                let sum = cpu.a as u16 + value + carry_in as u16;

                let mut result = sum as u8;
                let mut carry_out = sum > 0xFF;

                // Decimal (BCD) correction
                if cpu.p.d() {
                    let mut lo = (cpu.a & 0x0F).wrapping_sub((m & 0x0F) + (1 - carry_in));
                    let mut hi = (cpu.a >> 4).wrapping_sub((m >> 4) & 0x0F);
                    if (lo as i8) < 0 {
                        lo = lo.wrapping_sub(6);
                        hi = hi.wrapping_sub(1);
                    }
                    if (hi as i8) < 0 {
                        hi = hi.wrapping_sub(6);
                    }
                    result = ((hi << 4) | (lo & 0x0F)) & 0xFF;
                    carry_out = hi < 0x10;
                }

                // Update flags
                cpu.p.set_c(carry_out);
                cpu.p.set_v(((cpu.a ^ result) & (!m ^ result) & BIT_7) != 0);
                cpu.a = result;
                cpu.p.set_zn(result);
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

#[cfg(test)]
mod arith_tests {
    use crate::cpu::{
        mnemonic::{Mnemonic, tests::InstrTest},
        status::BIT_7,
    };

    #[test]
    fn test_adc() {
        unimplemented!()
    }

    #[test]
    fn test_anc() {
        InstrTest::new(Mnemonic::ANC).test(|verify, cpu, _| {
            let v = verify.cpu.a & verify.m;
            assert_eq!(cpu.a, v);

            // Carry = bit 7 of result
            let carry = v & BIT_7 != 0;
            assert_eq!(cpu.p.c(), carry);

            // Update N/Z flags
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_arr() {
        InstrTest::new(Mnemonic::ARR).test(|verify, cpu, _| {
            // Step 1: AND with operand
            let mut v = verify.cpu.a & verify.m;

            // Step 2: Logical shift right by 1
            v >>= 1;

            // Check accumulator result
            assert_eq!(cpu.a, v);

            // Carry = bit 6 of result
            let c = v & 0x40 != 0;
            assert_eq!(cpu.p.c(), c);

            // Overflow = bit6 XOR bit5
            let v_flag = ((v >> 6) & 1) ^ ((v >> 5) & 1) != 0;
            assert_eq!(cpu.p.v(), v_flag);

            // Negative / Zero flags
            verify.check_nz(cpu.p, v);
        });
    }
}
