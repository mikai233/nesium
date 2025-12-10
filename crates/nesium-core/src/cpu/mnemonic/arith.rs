use crate::cpu::{
    micro_op::MicroOp,
    mnemonic::Mnemonic,
    status::{BIT_0, BIT_5, BIT_6, BIT_7},
};

impl Mnemonic {
    /// NV-BDIZC
    /// ✓✓----✓✓
    ///
    /// ADC - Add Memory to Accumulator with Carry
    /// Operation: A + M + C → A, C
    ///
    /// This instruction adds the value of memory and carry from the previous
    /// operation to the value of the accumulator and stores the result in the
    /// accumulator.
    ///
    /// This instruction affects the accumulator; sets the carry flag when the sum of
    /// a binary add exceeds 255 or when the sum of a decimal add exceeds 99,
    /// otherwise carry is reset. The overflow flag is set when the sign or bit 7 is
    /// changed due to the result exceeding +127 or -128, otherwise overflow is
    /// reset. The negative flag is set if the accumulator result contains bit 7 on,
    /// otherwise the negative flag is reset. The zero flag is set if the accumulator
    /// result is 0, otherwise the zero flag is reset.
    ///
    /// **Note on the MOS 6502:**
    /// In decimal mode, the N, V and Z flags are not consistent with the decimal
    /// result.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | ADC #$nn                 | $69    | 2         | 2
    /// Absolute                            | ADC $nnnn                | $6D    | 3         | 4
    /// X-Indexed Absolute                  | ADC $nnnn,X              | $7D    | 3         | 4+p
    /// Y-Indexed Absolute                  | ADC $nnnn,Y              | $79    | 3         | 4+p
    /// Zero Page                           | ADC $nn                  | $65    | 2         | 3
    /// X-Indexed Zero Page                 | ADC $nn,X                | $75    | 2         | 4
    /// X-Indexed Zero Page Indirect        | ADC ($nn,X)              | $61    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | ADC ($nn),Y              | $71    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn adc() -> &'static [MicroOp] {
        &[MicroOp {
            name: "adc_binary",
            micro_fn: |cpu, bus| {
                // 1. Fetch Operand
                let m = bus.mem_read(cpu, cpu.effective_addr);

                // 2. Calculate Sum
                let carry_in = if cpu.p.c() { 1 } else { 0 };
                let sum = cpu.a as u16 + m as u16 + carry_in as u16;

                // --- Binary Mode (Standard 6502/2A03 Operation) ---
                // NES's 2A03 CPU ignores the Decimal flag, so we always execute Binary addition.

                let result = sum as u8;
                let carry_out = sum > 0xFF;

                // 3. Set Flags and Update Accumulator

                // C: Carry Flag (Set if sum > 255)
                cpu.p.set_c(carry_out);

                // V: Overflow Flag (Set if signed addition crosses the +/- 127 boundary)
                // (A^M) & 0x80: checks if operands have different signs (0: same sign)
                // (A^Result) & 0x80: checks if A and result have different signs (1: signs crossed)
                // Overflow occurs only if Operands have SAME sign AND the Result has a DIFFERENT sign.
                // Simplified check for overflow: ((A^M) & 0x80) == 0 && ((A^R) & 0x80) != 0
                // Your original calculation is a common alternative and should be fine:
                cpu.p.set_v((!(cpu.a ^ m) & (cpu.a ^ result) & BIT_7) != 0);

                // Update Accumulator
                cpu.a = result;

                // Z/N: Zero and Negative Flags
                cpu.p.set_zn(result);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// ANC - "AND" Memory with Accumulator then Move Negative Flag to Carry Flag
    /// Operation: A ∧ M → A, N → C
    ///
    /// The undocumented ANC instruction performs a bit-by-bit AND operation of the
    /// accumulator and memory and stores the result back in the accumulator.
    ///
    /// This instruction affects the accumulator; sets the zero flag if the result
    /// in the accumulator is 0, otherwise resets the zero flag; sets the negative
    /// flag and the carry flag if the result in the accumulator has bit 7 on,
    /// otherwise resets the negative flag and the carry flag.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | ANC #$nn                 | $0B*   | 2         | 2
    /// Immediate       | ANC #$nn                 | $2B*   | 2         | 2
    ///
    /// *Undocumented.
    pub(crate) const fn anc() -> &'static [MicroOp] {
        &[MicroOp {
            name: "anc",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_zn(cpu.a);
                cpu.p.set_c(cpu.a & BIT_7 != 0);
            },
        }]
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
        &[MicroOp {
            name: "arr_binary",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);

                // 1. A = A & M
                cpu.a &= m;

                // 2. ROR through Carry
                let carry_in = if cpu.p.c() { BIT_7 } else { 0 };
                cpu.a = (cpu.a >> 1) | carry_in;

                // 3. Set Flags (Always Binary Mode for 2A03)

                // N/Z: Standard setting based on final A
                cpu.p.set_n(cpu.a & BIT_7 != 0);
                cpu.p.set_z(cpu.a == 0);

                // V: V = (Result Bit 6) XOR (Result Bit 5)
                // We use your original V flag logic, which is standard for ARR in binary mode.
                cpu.p.set_v(((cpu.a & BIT_6) != 0) ^ ((cpu.a & BIT_5) != 0));

                // C: C = Result Bit 6 (This is the specific, non-standard behavior for ARR's C flag)
                cpu.p.set_c(cpu.a & BIT_6 != 0);
            },
        }]
    }

    /// NV-BDIZC
    /// 0-----✓✓
    ///
    /// ASR - "AND" then Logical Shift Right
    /// Operation: (A ∧ M) / 2 → A
    ///
    /// The undocumented ASR instruction performs a bit-by-bit AND operation of the
    /// accumulator and memory, then shifts the accumulator 1 bit to the right, with
    /// the higher bit of the result always being set to 0, and the low bit which is
    /// shifted out of the field being stored in the carry flag.
    ///
    /// This instruction affects the accumulator. It does not affect the overflow
    /// flag. The N flag is always reset. The Z flag is set if the result of the
    /// shift is 0 and reset otherwise. The carry is set equal to bit 0 of the result
    /// of the "AND" operation.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | ASR #$nn                 | $4B*   | 2         | 2
    ///
    /// *Undocumented.
    pub(crate) const fn asr() -> &'static [MicroOp] {
        &[MicroOp {
            name: "asr",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                cpu.a &= m;
                cpu.p.set_c(cpu.a & BIT_0 != 0);
                cpu.a >>= 1;
                cpu.p.set_zn(cpu.a);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// CMP - Compare Memory and Accumulator
    /// Operation: A - M
    ///
    /// This instruction subtracts the contents of memory from the contents of the
    /// accumulator.
    ///
    /// The use of the CMP affects the following flags: Z flag is set on an equal
    /// comparison, reset otherwise; the N flag is set or reset by the result bit 7,
    /// the carry flag is set when the value in memory is less than or equal to the
    /// accumulator, reset when it is greater than the accumulator. The accumulator
    /// is not affected.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | CMP #$nn                 | $C9    | 2         | 2
    /// Absolute                            | CMP $nnnn                | $CD    | 3         | 4
    /// X-Indexed Absolute                  | CMP $nnnn,X              | $DD    | 3         | 4+p
    /// Y-Indexed Absolute                  | CMP $nnnn,Y              | $D9    | 3         | 4+p
    /// Zero Page                           | CMP $nn                  | $C5    | 2         | 3
    /// X-Indexed Zero Page                 | CMP $nn,X                | $D5    | 2         | 4
    /// X-Indexed Zero Page Indirect        | CMP ($nn,X)              | $C1    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | CMP ($nn),Y              | $D1    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn cmp() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cmp",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                let result = cpu.a.wrapping_sub(m);
                cpu.p.set_c(cpu.a >= m);
                cpu.p.set_zn(result);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// CPX - Compare Index Register X To Memory
    /// Operation: X - M
    ///
    /// This instruction subtracts the value of the addressed memory location from
    /// the content of index register X using the adder but does not store the
    /// result; therefore, its only use is to set the N, Z and C flags to allow for
    /// comparison between the index register X and the value in memory.
    ///
    /// The CPX instruction does not affect any register in the machine; it also
    /// does not affect the overflow flag. It causes the carry to be set on if the
    /// absolute value of the index register X is equal to or greater than the data
    /// from memory. If the value of the memory is greater than the content of the
    /// index register X, carry is reset. If the results of the subtraction contain
    /// a bit 7, then the N flag is set, if not, it is reset. If the value in memory
    /// is equal to the value in index register X, the Z flag is set, otherwise it
    /// is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | CPX #$nn                 | $E0    | 2         | 2
    /// Absolute        | CPX $nnnn                | $EC    | 3         | 4
    /// Zero Page       | CPX $nn                  | $E4    | 2         | 3
    pub(crate) const fn cpx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cpx",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                let result = cpu.x.wrapping_sub(m);
                cpu.p.set_c(cpu.x >= m);
                cpu.p.set_zn(result);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// CPY - Compare Index Register Y To Memory
    /// Operation: Y - M
    ///
    /// This instruction performs a two's complement subtraction between the index
    /// register Y and the specified memory location. The results of the subtraction
    /// are not stored anywhere. The instruction is strictly used to set the flags.
    ///
    /// CPY affects no registers in the microprocessor and also does not affect the
    /// overflow flag. If the value in the index register Y is equal to or greater
    /// than the value in the memory, the carry flag will be set, otherwise it will
    /// be cleared. If the results of the subtraction contain bit 7 on the N bit will
    /// be set, otherwise it will be cleared. If the value in the index register Y
    /// and the value in the memory are equal, the zero flag will be set, otherwise
    /// it will be cleared.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | CPY #$nn                 | $C0    | 2         | 2
    /// Absolute        | CPY $nnnn                | $CC    | 3         | 4
    /// Zero Page       | CPY $nn                  | $C4    | 2         | 3
    pub(crate) const fn cpy() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cpy",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                let result = cpu.y.wrapping_sub(m);
                cpu.p.set_c(cpu.y >= m);
                cpu.p.set_zn(result);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// DCP - Decrement Memory By One then Compare with Accumulator
    /// Operation: M - 1 → M, A - M
    ///
    /// This undocumented instruction subtracts 1, in two's complement, from the
    /// contents of the addressed memory location. It then subtracts the contents of
    /// memory from the contents of the accumulator.
    ///
    /// The DCP instruction does not affect any internal register in the
    /// microprocessor. It does not affect the overflow flag. Z flag is set on an
    /// equal comparison, reset otherwise; the N flag is set or reset by the result
    /// bit 7, the carry flag is set when the result in memory is less than or equal
    /// to the accumulator, reset when it is greater than the accumulator.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | DCP $nnnn                | $CF*   | 3         | 6
    /// X-Indexed Absolute                  | DCP $nnnn,X              | $DF*   | 3         | 7
    /// Y-Indexed Absolute                  | DCP $nnnn,Y              | $DB*   | 3         | 7
    /// Zero Page                           | DCP $nn                  | $C7*   | 2         | 5
    /// X-Indexed Zero Page                 | DCP $nn,X                | $D7*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | DCP ($nn,X)              | $C3*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | DCP ($nn),Y              | $D3*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn dcp() -> &'static [MicroOp] {
        &[
            // T4: Read Old Value (R)
            MicroOp {
                name: "dcp_read_old",
                // Bus: READ V_old from M(effective_addr)
                // Internal: Store V_old in a temporary register (here, cpu.base)
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr);
                },
            },
            // T5: Dummy Write Old Value (W_dummy) & Internal Calculation (Modify)
            MicroOp {
                name: "dcp_dummy_write_dec",
                // Bus: WRITE V_old back to M(effective_addr) (The "dummy" cycle to burn time)
                // Internal: DEC calculation is performed. cpu.base now holds V_new.
                micro_fn: |cpu, bus| {
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base); // Dummy write of the old value

                    // Internal operation: Calculate the new value (V_new = V_old - 1)
                    cpu.base = cpu.base.wrapping_sub(1);
                    // The DEC result (V_new) is temporarily held in cpu.base
                },
            },
            // T6: Final Write New Value (W_new) & Internal CMP Operation
            MicroOp {
                name: "dcp_final_write_cmp",
                // Bus: WRITE V_new to M(effective_addr). This completes the DEC part.
                // Internal: Simultaneously perform CMP (A - V_new) and set flags.
                micro_fn: |cpu, bus| {
                    // Final Write: The correct, decremented value is written to memory.
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base);

                    // Internal Operation: Perform CMP (A - M) and update status flags (N, Z, C).
                    let m = cpu.base; // m is the decremented value (V_new)

                    // Carry flag (C): Set if A >= M (No Borrow)
                    cpu.p.set_c(cpu.a >= m);

                    // Negative (N) and Zero (Z) flags: Set based on the result of A - M
                    cpu.p.set_zn(cpu.a.wrapping_sub(m));
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓✓----✓✓
    ///
    /// ISC - Increment Memory By One then SBC then Subtract Memory from Accumulator with Borrow
    /// Operation: M + 1 → M, A - M → A
    ///
    /// This undocumented instruction adds 1 to the contents of the addressed memory
    /// location. It then subtracts the value of the result in memory and borrow from
    /// the value of the accumulator, using two's complement arithmetic, and stores
    /// the result in the accumulator.
    ///
    /// This instruction affects the accumulator. The carry flag is set if the result
    /// is greater than or equal to 0. The carry flag is reset when the result is
    /// less than 0, indicating a borrow. The overflow flag is set when the result
    /// exceeds +127 or -127, otherwise it is reset. The negative flag is set if the
    /// result in the accumulator has bit 7 on, otherwise it is reset. The Z flag is
    /// set if the result in the accumulator is 0, otherwise it is reset.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | ISC $nnnn                | $EF*   | 3         | 6
    /// X-Indexed Absolute                  | ISC $nnnn,X              | $FF*   | 3         | 7
    /// Y-Indexed Absolute                  | ISC $nnnn,Y              | $FB*   | 3         | 7
    /// Zero Page                           | ISC $nn                  | $E7*   | 2         | 5
    /// X-Indexed Zero Page                 | ISC $nn,X                | $F7*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | ISC ($nn,X)              | $E3*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | ISC ($nn),Y              | $F3*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn isc() -> &'static [MicroOp] {
        &[
            // T4: Read Old Value (R)
            MicroOp {
                name: "isc_read_old",
                micro_fn: |cpu, bus| {
                    // Read M_old into temporary storage (cpu.base)
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr);
                },
            },
            // T5: Dummy Write Old Value (W_dummy) & Internal Calculation
            MicroOp {
                name: "isc_dummy_write_inc",
                micro_fn: |cpu, bus| {
                    // 1. Bus: Dummy write of the old value (M_old) - Must be done for accurate timing
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base);

                    // 2. Internal: Calculate the new value (M_new = M_old + 1)
                    cpu.base = cpu.base.wrapping_add(1);
                },
            },
            // T6: Final Write New Value (W_final) & Internal SBC Operation
            MicroOp {
                name: "isc_final_write_sbc",
                micro_fn: |cpu, bus| {
                    // 1. Bus: Write the new, incremented value (M_new) to memory (Completes INC part)
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base);

                    // 2. Internal: Perform SBC (A - M_new - /C)
                    let m_new = cpu.base;
                    let m_inv = !m_new;
                    let carry = if cpu.p.c() { 1 } else { 0 };
                    let sum = cpu.a as u16 + m_inv as u16 + carry as u16;
                    let result = sum as u8;

                    // 3. Update Status Flags and Accumulator
                    cpu.p.set_c(sum > 0xFF);
                    cpu.p
                        .set_v(((cpu.a ^ result) & (m_inv ^ result) & BIT_7) != 0);
                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// RLA - Rotate Left then "AND" with Accumulator
    /// Operation: C ← /M7...M0/ ← C, A ∧ M → A
    ///
    /// The undocumented RLA instruction shifts the addressed memory left 1 bit, with
    /// the input carry being stored in bit 0 and with the input bit 7 being stored
    /// in the carry flags. It then performs a bit-by-bit AND operation of the result
    /// and the value of the accumulator and stores the result back in the
    /// accumulator.
    ///
    /// This instruction affects the accumulator; sets the zero flag if the result
    /// in the accumulator is 0, otherwise resets the zero flag; sets the negative
    /// flag if the result in the accumulator has bit 7 on, otherwise resets the
    /// negative flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | RLA $nnnn                | $2F*   | 3         | 6
    /// X-Indexed Absolute                  | RLA $nnnn,X              | $3F*   | 3         | 7
    /// Y-Indexed Absolute                  | RLA $nnnn,Y              | $3B*   | 3         | 7
    /// Zero Page                           | RLA $nn                  | $27*   | 2         | 5
    /// X-Indexed Zero Page                 | RLA $nn,X                | $37*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | RLA ($nn,X)              | $23*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | RLA ($nn),Y              | $33*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn rla() -> &'static [MicroOp] {
        &[
            // T4: Read Old Value (R)
            MicroOp {
                name: "rla_read_old",
                micro_fn: |cpu, bus| {
                    // Read M_old from memory into temporary storage (cpu.base)
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr);
                },
            },
            // T5: Dummy Write Old Value (W_dummy) & Internal Calculation (ROL)
            MicroOp {
                name: "rla_dummy_write_rol",
                micro_fn: |cpu, bus| {
                    // 1. Bus: Dummy write of the old value (M_old) - Accurate RMW timing
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base);

                    // 2. Internal: Perform ROL calculation (Rotate Left) to get M_new
                    let m_old = cpu.base;
                    let carry_in = if cpu.p.c() { 1 } else { 0 };

                    // Set the new Carry flag (C) based on the old bit 7
                    cpu.p.set_c(m_old & 0x80 != 0);

                    // Calculate the rotated new value (M_new)
                    let m_new = (m_old << 1) | carry_in;

                    // Store M_new back into cpu.base for final write and AND
                    cpu.base = m_new;
                },
            },
            // T6: Final Write New Value (W_final) & Internal AND Operation
            MicroOp {
                name: "rla_final_write_and",
                micro_fn: |cpu, bus| {
                    // 1. Bus: Write the new, rotated value (M_new) to memory (Completes ROL part)
                    bus.mem_write(cpu, cpu.effective_addr, cpu.base);

                    // 2. Internal: Perform AND operation (A = A & M_new)
                    let m_new = cpu.base;

                    // The ROL operation has already updated the Carry (C) flag in T5.
                    // We only need to update A, N, and Z flags based on A & M_new.
                    cpu.a &= m_new;

                    // Update Negative (N) and Zero (Z) flags based on the new Accumulator value
                    cpu.p.set_zn(cpu.a);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓✓----✓✓
    ///
    /// RRA - Rotate Right and Add Memory to Accumulator
    /// Operation: C → /M7...M0/ → C, A + M + C → A
    ///
    /// The undocumented RRA instruction shifts the addressed memory right 1 bit with
    /// bit 0 shifted into the carry and carry shifted into bit 7. It then adds the
    /// result and generated carry to the value of the accumulator and stores the
    /// result in the accumulator.
    ///
    /// This instruction affects the accumulator; sets the carry flag when the sum of
    /// a binary add exceeds 255 or when the sum of a decimal add exceeds 99,
    /// otherwise carry is reset. The overflow flag is set when the sign or bit 7 is
    /// changed due to the result exceeding +127 or -128, otherwise overflow is
    /// reset. The negative flag is set if the accumulator result contains bit 7 on,
    /// otherwise the negative flag is reset. The zero flag is set if the accumulator
    /// result is 0, otherwise the zero flag is reset.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | RRA $nnnn                | $6F*   | 3         | 6
    /// X-Indexed Absolute                  | RRA $nnnn,X              | $7F*   | 3         | 7
    /// Y-Indexed Absolute                  | RRA $nnnn,Y              | $7B*   | 3         | 7
    /// Zero Page                           | RRA $nn                  | $67*   | 2         | 5
    /// X-Indexed Zero Page                 | RRA $nn,X                | $77*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | RRA ($nn,X)              | $63*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | RRA ($nn),Y              | $73*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn rra() -> &'static [MicroOp] {
        &[
            // T5: Read Old Value (R)
            MicroOp {
                name: "rra_read_old",
                // Bus: READ V_old from M(effective_addr).
                // Internal: Store V_old in cpu.base for modification.
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr); // V_old (M)
                },
            },
            // T6: Dummy Write Old Value (W_dummy) & Internal ROR Calculation
            MicroOp {
                name: "rra_dummy_write_ror_calc",
                // Bus: WRITE V_old back to M(effective_addr). (Cycle Burn/Dummy Write).
                // Internal: Perform ROR calculation. cpu.base now holds M' (new value).
                micro_fn: |cpu, bus| {
                    let m_old = cpu.base;
                    bus.mem_write(cpu, cpu.effective_addr, m_old); // Dummy write of V_old

                    // --- ROR Calculation ---
                    let carry_in = if cpu.p.c() {
                        BIT_7 // Old Carry bit rotates into Bit 7
                    } else {
                        0
                    };

                    // Old Bit 0 becomes the new Carry flag
                    cpu.p.set_c(m_old & BIT_0 != 0);

                    // Perform the Rotate Right
                    let m_new = (m_old >> 1) | carry_in;

                    cpu.base = m_new; // cpu.base now holds M'
                },
            },
            // T7: Final Write New Value (W_new) & ADC Execution
            MicroOp {
                name: "rra_final_write_adc_exec",
                // Bus: WRITE M' (V_new) to M(effective_addr).
                // Internal: Perform ADC calculation (A = A + M' + C) and set all flags (Z/N/V/C).
                micro_fn: |cpu, bus| {
                    let m_prime = cpu.base; // M' (New ROR value)
                    bus.mem_write(cpu, cpu.effective_addr, m_prime); // Final write of M' (RRA's RMW part complete)

                    // --- ADC Execution ---
                    // ADC operates on M' (the value just written)
                    let carry = if cpu.p.c() { 1 } else { 0 };
                    let sum = cpu.a as u16 + m_prime as u16 + carry as u16;
                    let result = sum as u8;

                    // V Flag calculation (Standard ADC)
                    // (A^R) & (M'^R) & 0x80 -> Check if signs crossed when adding same-sign numbers
                    // Since M' is involved in the V calculation, this must occur after ROR.
                    cpu.p
                        .set_v(((cpu.a ^ result) & (m_prime ^ result) & BIT_7) != 0);

                    // C Flag calculation (Standard ADC)
                    cpu.p.set_c(sum > 0xFF);

                    // Final A and Z/N update
                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓✓----✓✓
    ///
    /// SBC - Subtract Memory from Accumulator with Borrow
    /// Operation: A - M - ~C → A
    ///
    /// This instruction subtracts the value of memory and borrow from the value of
    /// the accumulator, using two's complement arithmetic, and stores the result in
    /// the accumulator. Borrow is defined as the carry flag complemented; therefore,
    /// a resultant carry flag indicates that a borrow has not occurred.
    ///
    /// This instruction affects the accumulator. The carry flag is set if the result
    /// is greater than or equal to 0. The carry flag is reset when the result is
    /// less than 0, indicating a borrow. The overflow flag is set when the result
    /// exceeds +127 or -127, otherwise it is reset. The negative flag is set if the
    /// result in the accumulator has bit 7 on, otherwise it is reset. The Z flag is
    /// set if the result in the accumulator is 0, otherwise it is reset.
    ///
    /// **Note on the MOS 6502:**
    /// In decimal mode, the N, V and Z flags are not consistent with the decimal
    /// result.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | SBC #$nn                 | $E9    | 2         | 2
    /// Immediate                           | SBC #$nn                 | $EB*   | 2         | 2
    /// Absolute                            | SBC $nnnn                | $ED    | 3         | 4
    /// X-Indexed Absolute                  | SBC $nnnn,X              | $FD    | 3         | 4+p
    /// Y-Indexed Absolute                  | SBC $nnnn,Y              | $F9    | 3         | 4+p
    /// Zero Page                           | SBC $nn                  | $E5    | 2         | 3
    /// X-Indexed Zero Page                 | SBC $nn,X                | $F5    | 2         | 4
    /// X-Indexed Zero Page Indirect        | SBC ($nn,X)              | $E1    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | SBC ($nn),Y              | $F1    | 2         | 5+p
    ///
    /// *Undocumented.
    /// p: =1 if page is crossed.
    pub(crate) const fn sbc() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sbc_binary",
            // Micro-Op for SBC (Subtract with Carry) - Single cycle for execution, assuming effective_addr is ready.
            micro_fn: |cpu, bus| {
                // 1. Fetch Operand
                let m = bus.mem_read(cpu, cpu.effective_addr);

                // 2. Calculate Sum (using 2's complement addition: A + ~M + C)
                let carry_in = if cpu.p.c() { 1 } else { 0 };

                // Invert M to get ~M (one's complement)
                let value = (!m) as u16;

                // Perform the addition: A + ~M + C
                let sum = cpu.a as u16 + value + carry_in as u16;

                // --- Binary Mode (Standard 6502/2A03 Operation) ---
                let result = sum as u8;

                // C: Carry Flag (Set if sum > 255).
                // In subtraction, Carry means NO borrow occurred, i.e., A >= M.
                let carry_out = sum > 0xFF;

                // 3. Set Flags and Update Accumulator

                // V: Overflow Flag (Set if signed subtraction crosses the +/- 127 boundary)
                // SBC V-flag: ((A^Result) & (~M ^ Result) & 0x80) != 0
                // Note: We use ~M (value) in the calculation.
                cpu.p
                    .set_v(((cpu.a ^ result) & (value as u8 ^ result) & BIT_7) != 0);

                // C: Carry Flag (Set if no borrow)
                cpu.p.set_c(carry_out);

                // Update Accumulator
                cpu.a = result;

                // Z/N: Zero and Negative Flags
                cpu.p.set_zn(result);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// SBX - Subtract Memory from Accumulator "AND" Index Register X
    /// Operation: (A ∧ X) - M → X
    ///
    /// This undocumented instruction performs a bit-by-bit "AND" of the value of the
    /// accumulator and the index register X and subtracts the value of memory from
    /// this result, using two's complement arithmetic, and stores the result in the
    /// index register X.
    ///
    /// This instruction affects the index register X. The carry flag is set if the
    /// result is greater than or equal to 0. The carry flag is reset when the result
    /// is less than 0, indicating a borrow. The negative flag is set if the result
    /// in index register X has bit 7 on, otherwise it is reset. The Z flag is set if
    /// the result in index register X is 0, otherwise it is reset. The overflow flag
    /// not affected at all.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | SBX #$nn                 | $CB*   | 2         | 2
    ///
    /// *Undocumented.
    pub(crate) const fn sbx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sbx",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                let value = (cpu.a & cpu.x).wrapping_sub(m);
                cpu.p.set_c((cpu.a & cpu.x) >= m);
                cpu.x = value;
                cpu.p.set_zn(cpu.x);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// SLO - Arithmetic Shift Left then "OR" Memory with Accumulator
    /// Operation: M * 2 → M, A ∨ M → A
    ///
    /// The undocumented SLO instruction shifts the address memory location 1 bit to
    /// the left, with the bit 0 always being set to 0 and the bit 7 output always
    /// being contained in the carry flag. It then performs a bit-by-bit "OR"
    /// operation on the result and the accumulator and stores the result in the
    /// accumulator.
    ///
    /// The negative flag is set if the accumulator result contains bit 7 on,
    /// otherwise the negative flag is reset. It sets Z flag if the result is equal
    /// to 0, otherwise resets Z and stores the input bit 7 in the carry flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | SLO $nnnn                | $0F*   | 3         | 6
    /// X-Indexed Absolute                  | SLO $nnnn,X              | $1F*   | 3         | 7
    /// Y-Indexed Absolute                  | SLO $nnnn,Y              | $1B*   | 3         | 7
    /// Zero Page                           | SLO $nn                  | $07*   | 2         | 5
    /// X-Indexed Zero Page                 | SLO $nn,X                | $17*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | SLO ($nn,X)              | $03*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | SLO ($nn),Y              | $13*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn slo() -> &'static [MicroOp] {
        &[
            // T5: Read Old Value (R)
            MicroOp {
                name: "slo_read_old",
                // Bus: READ V_old from M(effective_addr).
                // Internal: Store V_old in cpu.base for modification.
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr); // V_old (M)
                },
            },
            // T6: Dummy Write Old Value (W_dummy) & Internal ASL Calculation
            MicroOp {
                name: "slo_dummy_write_asl_calc",
                // Bus: WRITE V_old back to M(effective_addr). (Cycle Burn/Dummy Write).
                // Internal: Perform ASL (Shift Left) calculation. cpu.base now holds M' (new value).
                micro_fn: |cpu, bus| {
                    let m_old = cpu.base;
                    bus.mem_write(cpu, cpu.effective_addr, m_old); // Dummy write of V_old

                    // --- ASL Calculation (Shift Left) ---

                    // Old Bit 7 becomes the new Carry flag
                    cpu.p.set_c(m_old & BIT_7 != 0);

                    // Perform the Arithmetic Shift Left (Bit 0 gets 0, Bit 7 goes to C)
                    let m_new = m_old.wrapping_mul(2); // m_old << 1

                    cpu.base = m_new; // cpu.base now holds M'
                },
            },
            // T7: Final Write New Value (W_new) & ORA Execution
            MicroOp {
                name: "slo_final_write_ora_exec",
                // Bus: WRITE M' (V_new) to M(effective_addr).
                // Internal: Perform ORA operation (A = A | M') and set N/Z flags.
                micro_fn: |cpu, bus| {
                    let m_prime = cpu.base; // M' (New ASL value)
                    bus.mem_write(cpu, cpu.effective_addr, m_prime); // Final write of M' (SLO's RMW part complete)

                    // --- ORA Execution ---
                    // ORA operates on A and M' (the value just written)

                    // A = A | M'
                    let result = cpu.a | m_prime;

                    // Final A and Z/N update
                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓✓
    ///
    /// SRE - Logical Shift Right then "Exclusive OR" Memory with Accumulator
    /// Operation: M / 2 → M, A ⊻ M → A
    ///
    /// The undocumented SRE instruction shifts the specified memory location 1 bit
    /// to the right, with the higher bit of the result always being set to 0, and
    /// the low bit which is shifted out of the field being stored in the carry
    /// flag. It then performs a bit-by-bit "EXCLUSIVE OR" of the result and the
    /// value of the accumulator and stores the result in the accumulator.
    ///
    /// This instruction affects the accumulator. It does not affect the overflow
    /// flag. The negative flag is set if the accumulator result contains bit 7 on,
    /// otherwise the negative flag is reset. The Z flag is set if the result is 0
    /// and reset otherwise. The carry is set equal to input bit 0.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | SRE $nnnn                | $4F*   | 3         | 6
    /// X-Indexed Absolute                  | SRE $nnnn,X              | $5F*   | 3         | 7
    /// Y-Indexed Absolute                  | SRE $nnnn,Y              | $5B*   | 3         | 7
    /// Zero Page                           | SRE $nn                  | $47*   | 2         | 5
    /// X-Indexed Zero Page                 | SRE $nn,X                | $57*   | 2         | 6
    /// X-Indexed Zero Page Indirect        | SRE ($nn,X)              | $43*   | 2         | 8
    /// Zero Page Indirect Y-Indexed        | SRE ($nn),Y              | $53*   | 2         | 8
    ///
    /// *Undocumented.
    pub(crate) const fn sre() -> &'static [MicroOp] {
        &[
            // T5: Read Old Value (R)
            MicroOp {
                name: "sre_read_old",
                // Bus: READ V_old from M(effective_addr).
                // Internal: Store V_old in cpu.base for modification.
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu, cpu.effective_addr); // V_old (M)
                },
            },
            // T6: Dummy Write Old Value (W_dummy) & Internal LSR Calculation
            MicroOp {
                name: "sre_dummy_write_lsr_calc",
                // Bus: WRITE V_old back to M(effective_addr). (Cycle Burn/Dummy Write).
                // Internal: Perform LSR (Shift Right) calculation. cpu.base now holds M' (new value).
                micro_fn: |cpu, bus| {
                    let m_old = cpu.base;
                    bus.mem_write(cpu, cpu.effective_addr, m_old); // Dummy write of V_old

                    // --- LSR Calculation (Logical Shift Right) ---

                    // Old Bit 0 becomes the new Carry flag
                    cpu.p.set_c(m_old & BIT_0 != 0);

                    // Perform the Logical Shift Right (Bit 7 gets 0, Bit 0 goes to C)
                    let m_new = m_old >> 1;

                    cpu.base = m_new; // cpu.base now holds M'
                },
            },
            // T7: Final Write New Value (W_new) & EOR Execution
            MicroOp {
                name: "sre_final_write_eor_exec",
                // Bus: WRITE M' (V_new) to M(effective_addr).
                // Internal: Perform EOR operation (A = A ^ M') and set N/Z flags.
                micro_fn: |cpu, bus| {
                    let m_prime = cpu.base; // M' (New LSR value)
                    bus.mem_write(cpu, cpu.effective_addr, m_prime); // Final write of M' (SRE's RMW part complete)

                    // --- EOR Execution ---
                    // EOR operates on A and M' (the value just written)

                    // A = A ^ M'
                    let result = cpu.a ^ m_prime;

                    // Final A and Z/N update
                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// XAA - Non-deterministic Operation of Accumulator, Index Register X, Memory and Bus Contents
    /// Operation: (A ∨ V) ∧ X ∧ M → A
    ///
    /// The operation of the undocumented XAA instruction depends on the individual
    /// microprocessor. On most machines, it performs a bit-by-bit AND operation of
    /// the following three operands: The first two are the index register X and
    /// memory.
    ///
    /// The third operand is the result of a bit-by-bit AND operation of the
    /// accumulator and a magic component. This magic component depends on the
    /// individual microprocessor and is usually one of `$00`, `$EE`, `$EF`, `$FE`
    /// and `$FF`, and may be influenced by the RDY pin, leftover contents of the
    /// data bus, the temperature of the microprocessor, the supplied voltage, and
    /// other factors.
    ///
    /// On some machines, additional bits of the result may be set or reset
    /// depending on non-deterministic factors.
    ///
    /// It then transfers the result to the accumulator.
    ///
    /// XAA does not affect the C or V flags; sets Z if the value loaded was zero,
    /// otherwise resets it; sets N if the result in bit 7 is a 1; otherwise N is
    /// reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate       | XAA #$nn                 | $8B*   | 2         | 2
    ///
    /// *Undocumented.
    pub(crate) const fn xaa() -> &'static [MicroOp] {
        &[MicroOp {
            name: "xaa",
            micro_fn: |cpu, bus| {
                let m = bus.mem_read(cpu, cpu.effective_addr);
                cpu.a = (cpu.a & cpu.x) & m;
                cpu.p.set_zn(cpu.a);
            },
        }]
    }
}

#[cfg(test)]
mod arith_tests {
    use crate::cpu::{
        mnemonic::{Mnemonic, tests::InstrTest},
        status::{BIT_0, BIT_7},
    };

    #[test]
    fn test_adc() {
        InstrTest::new(Mnemonic::ADC).test(|verify, cpu, _| {
            let m = verify.m;
            let a_in = verify.cpu.a;
            let c_in = if verify.cpu.p.c() { 1 } else { 0 };

            let sum_16 = a_in as u16 + m as u16 + c_in as u16;
            let final_a = (sum_16 & 0xFF) as u8;

            assert_eq!(cpu.a, final_a, "Accumulator mismatch after ADC");

            let final_c = sum_16 > 0xFF;
            assert_eq!(cpu.p.c(), final_c, "Carry flag mismatch");

            let final_v = ((a_in ^ final_a) & (m ^ final_a) & BIT_7) != 0;
            assert_eq!(cpu.p.v(), final_v, "Overflow flag mismatch");

            let zero = final_a == 0;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            let negative = final_a & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            verify.check_nz(cpu.p, final_a);
        });
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
            let old_carry = verify.cpu.p.c();
            let old_a = verify.cpu.a;
            let m = verify.m;

            // Step 1: AND with operand
            let and_result = old_a & m;

            // Step 2: ROR operation with carry input
            let shifted = (and_result >> 1) | (if old_carry { 0x80 } else { 0 });

            // Check accumulator result
            assert_eq!(cpu.a, shifted, "Accumulator result mismatch");

            // Carry = bit 6 of result
            let c = shifted & 0x40 != 0;
            assert_eq!(cpu.p.c(), c, "Carry flag mismatch");

            // Overflow = bit6 XOR bit5
            let v_flag = (shifted & 0x40 != 0) ^ (shifted & 0x20 != 0);
            assert_eq!(cpu.p.v(), v_flag, "Overflow flag mismatch");

            // Negative / Zero flags
            verify.check_nz(cpu.p, shifted);
        });
    }

    #[test]
    fn test_asr() {
        InstrTest::new(Mnemonic::ASR).test(|verify, cpu, _| {
            let mut v = verify.cpu.a & verify.m;
            v >>= 1;
            assert_eq!(cpu.a, v);
            let original_low_bit = (verify.cpu.a & verify.m) & 1;
            assert_eq!(cpu.p.c(), original_low_bit != 0);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_cmp() {
        InstrTest::new(Mnemonic::CMP).test(|verify, cpu, _| {
            // Step 1: Simulate (A - M) operation without storing the result
            let result = verify.cpu.a.wrapping_sub(verify.m);

            // Step 2: Verify processor flags
            // Carry = 1 if A >= M
            let carry = verify.cpu.a >= verify.m;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if A == M
            let zero = verify.cpu.a == verify.m;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Negative = bit 7 of (A - M)
            let negative = result & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Step 3: Ensure accumulator remains unchanged
            assert_eq!(cpu.a, verify.cpu.a, "Accumulator should remain unchanged");

            // Step 4: Cross-check N/Z flags with helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_cpx() {
        InstrTest::new(Mnemonic::CPX).test(|verify, cpu, _| {
            // Step 1: Simulate (X - M) operation
            let (result, _borrow) = verify.cpu.x.overflowing_sub(verify.m);

            // Step 2: Verify processor flags
            // Carry = 1 if X >= M
            let carry = verify.cpu.x >= verify.m;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if X == M
            let zero = verify.cpu.x == verify.m;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Negative = bit 7 of (X - M)
            let negative = result & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Step 3: Ensure register X remains unchanged
            assert_eq!(cpu.x, verify.cpu.x, "Register X should remain unchanged");

            // Step 4: Cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_cpy() {
        InstrTest::new(Mnemonic::CPY).test(|verify, cpu, _| {
            // Step 1: Simulate (Y - M) operation
            let (result, _borrow) = verify.cpu.y.overflowing_sub(verify.m);

            // Step 2: Verify processor flags
            // Carry = 1 if Y >= M
            let carry = verify.cpu.y >= verify.m;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if Y == M
            let zero = verify.cpu.y == verify.m;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Negative = bit 7 of (Y - M)
            let negative = result & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Step 3: Ensure register Y remains unchanged
            assert_eq!(cpu.y, verify.cpu.y, "Register Y should remain unchanged");

            // Step 4: Cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_dcp() {
        InstrTest::new(Mnemonic::DCP).test(|verify, cpu, bus| {
            // Step 1: Decrement memory value (simulate DEC M)
            let new_m = verify.m.wrapping_sub(1);

            // Step 2: Perform CMP A, new_m
            let (result, _borrow) = verify.cpu.a.overflowing_sub(new_m);

            // Step 3: Verify processor flags for CMP part
            // Carry = 1 if A >= new_m
            let carry = verify.cpu.a >= new_m;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if A == new_m
            let zero = verify.cpu.a == new_m;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Negative = bit 7 of (A - new_m)
            let negative = result & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Step 4: Verify memory has been decremented
            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                new_m,
                "Memory was not decremented"
            );

            // Step 5: Accumulator remains unchanged
            assert_eq!(cpu.a, verify.cpu.a, "Accumulator should remain unchanged");

            // Step 6: Optional: cross-check N/Z flags
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_isc() {
        InstrTest::new(Mnemonic::ISC).test(|verify, cpu, bus| {
            // Step 1: Increment memory value (simulate INC M)
            let new_m = verify.m.wrapping_add(1);

            // Step 2: Perform SBC A, new_m
            // SBC: A - M - (1 - C) 等价于 A + (!M) + C
            let carry_in = if verify.cpu.p.c() { 1 } else { 0 };
            let inverted_m = !new_m;
            let sum = verify.cpu.a as u16 + inverted_m as u16 + carry_in as u16;
            let sbc_result = sum as u8;

            // Step 3: Verify memory has been incremented
            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                new_m,
                "Memory was not incremented"
            );

            // Step 4: Accumulator updated correctly
            assert_eq!(cpu.a, sbc_result, "Accumulator mismatch after SBC");

            // Step 5: Update flags for SBC
            // Carry = 1 if no borrow occurred (sum > 0xFF)
            let carry = sum > 0xFF;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if result == 0
            assert_eq!(cpu.p.z(), sbc_result == 0, "Zero flag mismatch");

            // Negative = bit 7 of result
            assert_eq!(cpu.p.n(), sbc_result & BIT_7 != 0, "Negative flag mismatch");

            // Overflow = signed overflow detection
            let overflow = ((verify.cpu.a ^ sbc_result) & (inverted_m ^ sbc_result) & BIT_7) != 0;
            assert_eq!(cpu.p.v(), overflow, "Overflow flag mismatch");

            // Step 6: Optional: cross-check N/Z flags
            verify.check_nz(cpu.p, sbc_result);
        });
    }

    #[test]
    fn test_rla() {
        InstrTest::new(Mnemonic::RLA).test(|verify, cpu, bus| {
            // Step 1: Rotate memory left through carry
            let old_carry = if verify.cpu.p.c() { 1 } else { 0 };
            let mut rotated = (verify.m << 1) | old_carry;
            rotated &= 0xFF; // ensure 8-bit
            let new_carry = (verify.m & BIT_7) != 0;

            // Step 2: Update memory with rotated value
            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                rotated,
                "Memory not rotated correctly"
            );

            // Step 3: AND accumulator with rotated memory
            let result = verify.cpu.a & rotated;

            // Step 4: Update accumulator
            assert_eq!(cpu.a, result, "Accumulator not ANDed correctly");

            // Step 5: Verify flags
            // Carry = bit7 of original memory
            assert_eq!(cpu.p.c(), new_carry, "Carry flag mismatch");

            // Negative = bit7 of result
            let negative = result & BIT_7 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Zero = 1 if result is zero
            let zero = result == 0;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Optional: cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_rra() {
        InstrTest::new(Mnemonic::RRA).test(|verify, cpu, bus| {
            let old_carry = verify.cpu.p.c();
            let m = verify.m;

            let ror_new_carry = (m & BIT_0) != 0;
            let carry_in_for_ror = if old_carry { BIT_7 } else { 0 };
            let rotated = (m >> 1) | carry_in_for_ror;

            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                rotated,
                "Memory not rotated correctly"
            );

            let carry_for_adc = if ror_new_carry { 1 } else { 0 };

            let sum = verify.cpu.a as u16 + rotated as u16 + carry_for_adc as u16;

            let adc_result = sum as u8;

            assert_eq!(cpu.a, adc_result, "Accumulator mismatch after ADC");

            let carry = sum > 0xFF;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            assert_eq!(cpu.p.z(), adc_result == 0, "Zero flag mismatch");

            assert_eq!(cpu.p.n(), adc_result & BIT_7 != 0, "Negative flag mismatch");

            let overflow = ((verify.cpu.a ^ adc_result) & (rotated ^ adc_result) & BIT_7) != 0;
            assert_eq!(cpu.p.v(), overflow, "Overflow flag mismatch");
        });
    }

    #[test]
    fn test_sbc() {
        InstrTest::new(Mnemonic::SBC).test(|verify, cpu, _| {
            let m = verify.m;
            let a_in = verify.cpu.a;
            let c_in = if verify.cpu.p.c() { 1 } else { 0 };

            let inverted_m = !m;
            let sum_16 = a_in as u16 + inverted_m as u16 + c_in as u16;
            let final_a = (sum_16 & 0xFF) as u8;

            assert_eq!(cpu.a, final_a, "Accumulator mismatch after SBC");

            let final_c = sum_16 > 0xFF;
            assert_eq!(cpu.p.c(), final_c, "Carry flag mismatch");

            let final_v = ((a_in ^ final_a) & (inverted_m ^ final_a) & BIT_7) != 0;
            assert_eq!(cpu.p.v(), final_v, "Overflow flag mismatch");

            assert_eq!(cpu.p.z(), final_a == 0, "Zero flag mismatch");
            assert_eq!(cpu.p.n(), final_a & BIT_7 != 0, "Negative flag mismatch");
        });
    }

    #[test]
    fn test_sbx() {
        InstrTest::new(Mnemonic::SBX).test(|verify, cpu, _| {
            // Step 1: Compute A & X
            let ax = verify.cpu.a & verify.cpu.x;

            // Step 2: Compute X = (A & X) - M
            let result = ax.wrapping_sub(verify.m);

            // Step 3: Update X register
            assert_eq!(cpu.x, result, "X register mismatch after SBX");

            // Step 4: Update flags
            // Carry = 1 if (A & X) >= M
            let carry = ax >= verify.m;
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");

            // Zero = 1 if result == 0
            let zero = result == 0;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Negative = bit7 of result
            let negative = result & 0x80 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Optional: cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_slo() {
        InstrTest::new(Mnemonic::SLO).test(|verify, cpu, bus| {
            // Step 1: Perform ASL on memory
            let asl_result = verify.m << 1;
            let carry = (verify.m & BIT_7) != 0;

            // Step 2: Update memory
            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                asl_result,
                "Memory not shifted correctly"
            );

            // Step 3: OR accumulator with rotated memory
            let result = verify.cpu.a | asl_result;

            // Step 4: Update accumulator
            assert_eq!(cpu.a, result, "Accumulator not ORed correctly");

            // Step 5: Verify flags
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");
            assert_eq!(cpu.p.n(), result & BIT_7 != 0, "Negative flag mismatch");
            assert_eq!(cpu.p.z(), result == 0, "Zero flag mismatch");

            // Optional: cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_sre() {
        InstrTest::new(Mnemonic::SRE).test(|verify, cpu, bus| {
            // Step 1: Perform LSR on memory
            let lsr_result = verify.m >> 1;
            let carry = (verify.m & 0x01) != 0;

            // Step 2: Update memory
            assert_eq!(
                bus.mem_read(cpu, verify.addr),
                lsr_result,
                "Memory not shifted correctly"
            );

            // Step 3: EOR accumulator with shifted memory
            let result = verify.cpu.a ^ lsr_result;

            // Step 4: Update accumulator
            assert_eq!(cpu.a, result, "Accumulator not XORed correctly");

            // Step 5: Verify flags
            assert_eq!(cpu.p.c(), carry, "Carry flag mismatch");
            assert_eq!(cpu.p.n(), result & 0x80 != 0, "Negative flag mismatch");
            assert_eq!(cpu.p.z(), result == 0, "Zero flag mismatch");

            // Optional: cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }

    #[test]
    fn test_xaa() {
        InstrTest::new(Mnemonic::XAA).test(|verify, cpu, _| {
            // Step 1: Perform A & X & M
            let result = verify.cpu.a & verify.cpu.x & verify.m;

            // Step 2: Update accumulator
            assert_eq!(cpu.a, result, "Accumulator mismatch after XAA");

            // Step 3: Verify flags
            // Negative = bit7 of result
            let negative = result & 0x80 != 0;
            assert_eq!(cpu.p.n(), negative, "Negative flag mismatch");

            // Zero = 1 if result == 0
            let zero = result == 0;
            assert_eq!(cpu.p.z(), zero, "Zero flag mismatch");

            // Optional: cross-check N/Z flags using helper
            verify.check_nz(cpu.p, result);
        });
    }
}
