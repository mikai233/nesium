use crate::cpu::{
    micro_op::MicroOp,
    mnemonic::Mnemonic,
    status::{BIT_6, BIT_7},
};

impl Mnemonic {
    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// AND - "AND" Memory with Accumulator
    /// Operation: A ∧ M → A
    ///
    /// The AND instruction transfer the accumulator and memory to the adder which
    /// performs a bit-by-bit AND operation and stores the result back in the
    /// accumulator.
    ///
    /// This instruction affects the accumulator; sets the zero flag if the result
    /// in the accumulator is 0, otherwise resets the zero flag; sets the negative
    /// flag if the result in the accumulator has bit 7 on, otherwise resets the
    /// negative flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | AND #$nn                 | $29    | 2         | 2
    /// Absolute                            | AND $nnnn                | $2D    | 3         | 4
    /// X-Indexed Absolute                  | AND $nnnn,X              | $3D    | 3         | 4+p
    /// Y-Indexed Absolute                  | AND $nnnn,Y              | $39    | 3         | 4+p
    /// Zero Page                           | AND $nn                  | $25    | 2         | 3
    /// X-Indexed Zero Page                 | AND $nn,X                | $35    | 2         | 4
    /// X-Indexed Zero Page Indirect        | AND ($nn,X)              | $21    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | AND ($nn),Y              | $31    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn and() -> &'static [MicroOp] {
        &[MicroOp {
            name: "and",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_zn(cpu.a);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓✓----✓-
    ///
    /// BIT - Test Bits in Memory with Accumulator
    /// Operation: A ∧ M, M7 → N, M6 → V
    ///
    /// This instruction performs an AND between a memory location and the accumulator
    /// but does not store the result of the AND into the accumulator.
    ///
    /// The bit instruction affects the N flag with N being set to the value of bit 7
    /// of the memory being tested, the V flag with V being set equal to bit 6 of the
    /// memory being tested and Z being set by the result of the AND operation
    /// between the accumulator and the memory if the result is Zero, Z is reset
    /// otherwise. It does not affect the accumulator.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute        | BIT $nnnn                | $2C    | 3         | 4
    /// Zero Page       | BIT $nn                  | $24    | 2         | 3
    pub(crate) const fn bit() -> &'static [MicroOp] {
        &[MicroOp {
            name: "bit",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu.effective_addr);
                let and = cpu.a & m;
                cpu.p.set_z(and == 0);
                cpu.p.set_n(m & BIT_7 != 0);
                cpu.p.set_v(m & BIT_6 != 0);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// EOR - "Exclusive OR" Memory with Accumulator
    /// Operation: A ⊻ M → A
    ///
    /// The EOR instruction transfers the memory and the accumulator to the adder
    /// which performs a binary "EXCLUSIVE OR" on a bit-by-bit basis and stores the
    /// result in the accumulator.
    ///
    /// This instruction affects the accumulator; sets the zero flag if the result
    /// in the accumulator is 0, otherwise resets the zero flag sets the negative
    /// flag if the result in the accumulator has bit 7 on, otherwise resets the
    /// negative flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | EOR #$nn                 | $49    | 2         | 2
    /// Absolute                            | EOR $nnnn                | $4D    | 3         | 4
    /// X-Indexed Absolute                  | EOR $nnnn,X              | $5D    | 3         | 4+p
    /// Y-Indexed Absolute                  | EOR $nnnn,Y              | $59    | 3         | 4+p
    /// Zero Page                           | EOR $nn                  | $45    | 2         | 3
    /// X-Indexed Zero Page                 | EOR $nn,X                | $55    | 2         | 4
    /// X-Indexed Zero Page Indirect        | EOR ($nn,X)              | $41    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | EOR ($nn),Y              | $51    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn eor() -> &'static [MicroOp] {
        &[MicroOp {
            name: "eor",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu.effective_addr);
                cpu.a ^= m;
                cpu.p.set_zn(cpu.a);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// ORA - "OR" Memory with Accumulator
    /// Operation: A ∨ M → A
    ///
    /// The ORA instruction transfers the memory and the accumulator to the adder
    /// which performs a binary "OR" on a bit-by-bit basis and stores the result in
    /// the accumulator.
    ///
    /// This instruction affects the accumulator; sets the zero flag if the result
    /// in the accumulator is 0, otherwise resets the zero flag; sets the negative
    /// flag if the result in the accumulator has bit 7 on, otherwise resets the
    /// negative flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | ORA #$nn                 | $09    | 2         | 2
    /// Absolute                            | ORA $nnnn                | $0D    | 3         | 4
    /// X-Indexed Absolute                  | ORA $nnnn,X              | $1D    | 3         | 4+p
    /// Y-Indexed Absolute                  | ORA $nnnn,Y              | $19    | 3         | 4+p
    /// Zero Page                           | ORA $nn                  | $05    | 2         | 3
    /// X-Indexed Zero Page                 | ORA $nn,X                | $15    | 2         | 4
    /// X-Indexed Zero Page Indirect        | ORA ($nn,X)              | $01    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | ORA ($nn),Y              | $11    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn ora() -> &'static [MicroOp] {
        &[MicroOp {
            name: "ora",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu.effective_addr);
                cpu.a |= m;
                cpu.p.set_zn(cpu.a);
            },
        }]
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
