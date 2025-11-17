use crate::cpu::{micro_op::MicroOp, mnemonic::Mnemonic};

impl Mnemonic {
    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// DEC - Decrement Memory By One
    /// Operation: M - 1 → M
    ///
    /// This instruction subtracts 1, in two's complement, from the contents of the
    /// addressed memory location.
    ///
    /// The decrement instruction does not affect any internal register in the
    /// microprocessor. It does not affect the carry or overflow flags. If bit 7 is
    /// on as a result of the decrement, then the N flag is set, otherwise it is
    /// reset. If the result of the decrement is 0, the Z flag is set, otherwise it
    /// is reset.
    ///
    /// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                | DEC $nnnn                | $CE    | 3         | 6
    /// X-Indexed Absolute      | DEC $nnnn,X              | $DE    | 3         | 7
    /// Zero Page               | DEC $nn                  | $C6    | 2         | 5
    /// X-Indexed Zero Page     | DEC $nn,X                | $D6    | 2         | 6
    pub(crate) const fn dec() -> &'static [MicroOp] {
        &[
            // T5: Read Old Value (R)
            MicroOp {
                name: "dec_read_old",
                // Bus: READ V_old from M(effective_addr). This is the value to be decremented.
                // Internal: Store V_old in a temporary register (cpu.base).
                micro_fn: |cpu, bus| {
                    // Read the old value from memory
                    cpu.base = bus.read(cpu.effective_addr);
                },
            },
            // T6: Dummy Write Old Value (W_dummy) & Internal Calculation (Modify)
            MicroOp {
                name: "dec_dummy_write_calc",
                // Bus: WRITE V_old back to M(effective_addr). This burns a cycle (Dummy Write).
                // Internal: DEC calculation is performed. cpu.base now holds V_new.
                micro_fn: |cpu, bus| {
                    // Dummy write of the old value (V_old)
                    bus.write(cpu.effective_addr, cpu.base);

                    // Internal operation: Calculate the new value (V_new = V_old - 1)
                    cpu.base = cpu.base.wrapping_sub(1);
                    // The DEC result (V_new) is temporarily held in cpu.base
                },
            },
            // T7: Final Write New Value (W_new) & Internal Flag Update
            MicroOp {
                name: "dec_final_write_flags",
                // Bus: WRITE V_new to M(effective_addr). This completes the RMW sequence.
                // Internal: Update status flags (N, Z) based on V_new.
                micro_fn: |cpu, bus| {
                    // Final Write: The correct, decremented value is written to memory.
                    let new_value = cpu.base;
                    bus.write(cpu.effective_addr, new_value);

                    // Internal Operation: Update Negative (N) and Zero (Z) flags.
                    cpu.p.set_zn(new_value);

                    // Note: DEC does not affect the Carry flag (C)
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// DEX - Decrement Index Register X By One
    /// Operation: X - 1 → X
    ///
    /// This instruction subtracts one from the current value of the index register X
    /// and stores the result in the index register X.
    ///
    /// DEX does not affect the carry or overflow flag, it sets the N flag if it has
    /// bit 7 on as a result of the decrement, otherwise it resets the N flag; sets
    /// the Z flag if X is a 0 as a result of the decrement, otherwise it resets the
    /// Z flag.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | DEX                      | $CA    | 1         | 2
    pub(crate) const fn dex() -> &'static [MicroOp] {
        &[MicroOp {
            name: "dex",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_sub(1);
                cpu.p.set_zn(cpu.x);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// DEY - Decrement Index Register Y By One
    /// Operation: Y - 1 → Y
    ///
    /// This instruction subtracts one from the current value in the index register Y
    /// and stores the result into the index register Y. The result does not affect
    /// or consider carry so that the value in the index register Y is decremented to
    /// 0 and then through 0 to FF.
    ///
    /// Decrement Y does not affect the carry or overflow flags; if the Y register
    /// contains bit 7 on as a result of the decrement the N flag is set, otherwise
    /// the N flag is reset. If the Y register is 0 as a result of the decrement, the
    /// Z flag is set otherwise the Z flag is reset. This instruction only affects
    /// the index register Y.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | DEY                      | $88    | 1         | 2
    pub(crate) const fn dey() -> &'static [MicroOp] {
        &[MicroOp {
            name: "dey",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_sub(1);
                cpu.p.set_zn(cpu.y);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// INC - Increment Memory By One
    /// Operation: M + 1 → M
    ///
    /// This instruction adds 1 to the contents of the addressed memory location.
    ///
    /// The increment memory instruction does not affect any internal registers and
    /// does not affect the carry or overflow flags. If bit 7 is on as the result of
    /// the increment, N is set, otherwise it is reset; if the increment causes the
    /// result to become 0, the Z flag is set on, otherwise it is reset.
    ///
    /// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                | INC $nnnn                | $EE    | 3         | 6
    /// X-Indexed Absolute      | INC $nnnn,X              | $FE    | 3         | 7
    /// Zero Page               | INC $nn                  | $E6    | 2         | 5
    /// X-Indexed Zero Page     | INC $nn,X                | $F6    | 2         | 6
    pub(crate) const fn inc() -> &'static [MicroOp] {
        &[
            // T5: Read Old Value (R)
            MicroOp {
                name: "inc_read_old",
                // Bus: READ V_old from M(effective_addr). This is the value to be incremented.
                // Internal: Store V_old in a temporary CPU register (cpu.base).
                micro_fn: |cpu, bus| {
                    // Read the old value from memory
                    cpu.base = bus.read(cpu.effective_addr);
                },
            },
            // T6: Dummy Write Old Value (W_dummy) & Internal Calculation (Modify)
            MicroOp {
                name: "inc_dummy_write_calc",
                // Bus: WRITE V_old back to M(effective_addr). This burns a cycle (Dummy Write).
                // Internal: INC calculation is performed. cpu.base is updated to V_new.
                micro_fn: |cpu, bus| {
                    // Dummy write of the old value (V_old) - This is the "extra" RMW cycle.
                    bus.write(cpu.effective_addr, cpu.base);

                    // Internal operation: Calculate the new value (V_new = V_old + 1)
                    cpu.base = cpu.base.wrapping_add(1);
                    // The INC result (V_new) is temporarily held in cpu.base
                },
            },
            // T7: Final Write New Value (W_new) & Internal Flag Update
            MicroOp {
                name: "inc_final_write_flags",
                // Bus: WRITE V_new to M(effective_addr). This completes the RMW sequence.
                // Internal: Update status flags (N, Z) based on V_new.
                micro_fn: |cpu, bus| {
                    // Final Write: The correct, incremented value is written to memory.
                    let new_value = cpu.base;
                    bus.write(cpu.effective_addr, new_value);

                    // Internal Operation: Update Negative (N) and Zero (Z) flags.
                    cpu.p.set_zn(new_value);

                    // Note: INC does not affect the Carry flag (C)
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// INX - Increment Index Register X By One
    /// Operation: X + 1 → X
    ///
    /// Increment X adds 1 to the current value of the X register. This is an 8-bit
    /// increment which does not affect the carry operation, therefore, if the value
    /// of X before the increment was FF, the resulting value is 00.
    ///
    /// INX does not affect the carry or overflow flags; it sets the N flag if the
    /// result of the increment has a one in bit 7, otherwise resets N; sets the Z
    /// flag if the result of the increment is 0, otherwise it resets the Z flag.
    ///
    /// INX does not affect any other register other than the X register.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | INX                      | $E8    | 1         | 2
    pub(crate) const fn inx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "inx",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_add(1);
                cpu.p.set_zn(cpu.x);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// INY - Increment Index Register Y By One
    /// Operation: Y + 1 → Y
    ///
    /// Increment Y increments or adds one to the current value in the Y register,
    /// storing the result in the Y register. As in the case of INX the primary
    /// application is to step thru a set of values using the Y register.
    ///
    /// The INY does not affect the carry or overflow flags, sets the N flag if the
    /// result of the increment has a one in bit 7, otherwise resets N, sets Z if as
    /// a result of the increment the Y register is zero otherwise resets the Z flag.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | INY                      | $C8    | 1         | 2
    pub(crate) const fn iny() -> &'static [MicroOp] {
        &[MicroOp {
            name: "iny",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_add(1);
                cpu.p.set_zn(cpu.y);
            },
        }]
    }
}

#[cfg(test)]
mod inc_tests {
    use crate::cpu::{
        mnemonic::{Mnemonic, tests::InstrTest},
        status::BIT_7,
    };

    #[test]
    fn test_dec() {
        InstrTest::new(Mnemonic::DEC).test(|verify, cpu, bus| {
            let expected_value = verify.m.wrapping_sub(1);

            assert_eq!(
                bus.read(verify.addr),
                expected_value,
                "Memory was not decremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_value == 0, "Zero flag mismatch");
            assert_eq!(
                cpu.p.n(),
                expected_value & BIT_7 != 0,
                "Negative flag mismatch"
            );

            verify.check_nz(cpu.p, expected_value);
        });
    }

    #[test]
    fn test_dex() {
        InstrTest::new(Mnemonic::DEX).test(|verify, cpu, _| {
            let expected_x = verify.cpu.x.wrapping_sub(1);

            assert_eq!(
                cpu.x, expected_x,
                "X register was not decremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_x == 0, "Zero flag mismatch");
            assert_eq!(cpu.p.n(), expected_x & BIT_7 != 0, "Negative flag mismatch");

            verify.check_nz(cpu.p, expected_x);
        });
    }

    #[test]
    fn test_dey() {
        InstrTest::new(Mnemonic::DEY).test(|verify, cpu, _| {
            let expected_y = verify.cpu.y.wrapping_sub(1);

            assert_eq!(
                cpu.y, expected_y,
                "Y register was not decremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_y == 0, "Zero flag mismatch");
            assert_eq!(cpu.p.n(), expected_y & BIT_7 != 0, "Negative flag mismatch");

            verify.check_nz(cpu.p, expected_y);
        });
    }

    #[test]
    fn test_inc() {
        InstrTest::new(Mnemonic::INC).test(|verify, cpu, bus| {
            let expected_value = verify.m.wrapping_add(1);

            assert_eq!(
                bus.read(verify.addr),
                expected_value,
                "Memory was not incremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_value == 0, "Zero flag mismatch");
            assert_eq!(
                cpu.p.n(),
                expected_value & BIT_7 != 0,
                "Negative flag mismatch"
            );

            verify.check_nz(cpu.p, expected_value);
        });
    }

    #[test]
    fn test_inx() {
        InstrTest::new(Mnemonic::INX).test(|verify, cpu, _| {
            let expected_x = verify.cpu.x.wrapping_add(1);

            assert_eq!(
                cpu.x, expected_x,
                "X register was not incremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_x == 0, "Zero flag mismatch");
            assert_eq!(cpu.p.n(), expected_x & BIT_7 != 0, "Negative flag mismatch");

            verify.check_nz(cpu.p, expected_x);
        });
    }

    #[test]
    fn test_iny() {
        InstrTest::new(Mnemonic::INY).test(|verify, cpu, _| {
            let expected_y = verify.cpu.y.wrapping_add(1);

            assert_eq!(
                cpu.y, expected_y,
                "Y register was not incremented correctly"
            );

            assert_eq!(cpu.p.z(), expected_y == 0, "Zero flag mismatch");
            assert_eq!(cpu.p.n(), expected_y & BIT_7 != 0, "Negative flag mismatch");

            verify.check_nz(cpu.p, expected_y);
        });
    }
}
