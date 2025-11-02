use crate::{
    bus::Bus,
    cpu::{
        micro_op::MicroOp,
        mnemonic::Mnemonic,
        status::{BIT_5, BIT_6, BIT_7, Status},
    },
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
