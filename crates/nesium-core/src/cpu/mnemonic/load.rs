use crate::cpu::{micro_op::MicroOp, mnemonic::Mnemonic};

impl Mnemonic {
    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// LAS - "AND" Memory with Stack Pointer
    /// Operation: M & S → A, X, S
    ///
    /// This undocumented instruction performs a bit-by-bit "AND" operation of the
    /// stack pointer and memory and stores the result back in the accumulator,
    /// the index register X and the stack pointer.
    ///
    /// The LAS instruction does not affect the carry or overflow flags. It sets N
    /// if the bit 7 of the result is on, otherwise it is reset. If the result is
    /// zero, then the Z flag is set, otherwise it is reset.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Y-Indexed Absolute  | LAS $nnnn,Y              | $BB*   | 3         | 4+p
    ///
    /// *Undocumented.
    /// p: =1 if page is crossed.
    pub(crate) const fn las() -> &'static [MicroOp] {
        &[MicroOp {
            name: "las",
            micro_fn: |cpu, bus| {
                let value = bus.mem_read(cpu, cpu.effective_addr) & cpu.s;
                cpu.a = value;
                cpu.x = value;
                cpu.s = value;
                cpu.p.set_zn(value);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// LAX - Load Accumulator and Index Register X From Memory
    /// Operation: M → A, X
    ///
    /// The undocumented LAX instruction loads the accumulator and the index
    /// register X from memory.
    ///
    /// LAX does not affect the C or V flags; sets Z if the value loaded was zero,
    /// otherwise resets it; sets N if the value loaded in bit 7 is a 1; otherwise
    /// N is reset, and affects only the X register.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | LAX #$nn                 | $AB*   | 2         | 2
    /// Absolute                            | LAX $nnnn                | $AF*   | 3         | 4
    /// Y-Indexed Absolute                  | LAX $nnnn,Y              | $BF*   | 3         | 4+p
    /// Zero Page                           | LAX $nn                  | $A7*   | 2         | 3
    /// Y-Indexed Zero Page                 | LAX $nn,Y                | $B7*   | 2         | 4
    /// X-Indexed Zero Page Indirect        | LAX ($nn,X)              | $A3*   | 2         | 6
    /// Zero Page Indirect Y-Indexed        | LAX ($nn),Y              | $B3*   | 2         | 5+p
    ///
    /// *Undocumented.
    /// p: =1 if page is crossed.
    pub(crate) const fn lax() -> &'static [MicroOp] {
        &[MicroOp {
            name: "lax",
            micro_fn: |cpu, bus| {
                let value = bus.mem_read(cpu, cpu.effective_addr);
                cpu.a = value;
                cpu.x = value;
                cpu.p.set_zn(value);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// LDA - Load Accumulator with Memory
    /// Operation: M → A
    ///
    /// When instruction LDA is executed by the microprocessor, data is transferred
    /// from memory to the accumulator and stored in the accumulator.
    ///
    /// LDA affects the contents of the accumulator, does not affect the carry or
    /// overflow flags; sets the zero flag if the accumulator is zero as a result of
    /// the LDA, otherwise resets the zero flag; sets the negative flag if bit 7 of
    /// the accumulator is a 1, otherwise resets the negative flag.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate                           | LDA #$nn                 | $A9    | 2         | 2
    /// Absolute                            | LDA $nnnn                | $AD    | 3         | 4
    /// X-Indexed Absolute                  | LDA $nnnn,X              | $BD    | 3         | 4+p
    /// Y-Indexed Absolute                  | LDA $nnnn,Y              | $B9    | 3         | 4+p
    /// Zero Page                           | LDA $nn                  | $A5    | 2         | 3
    /// X-Indexed Zero Page                 | LDA $nn,X                | $B5    | 2         | 4
    /// X-Indexed Zero Page Indirect        | LDA ($nn,X)              | $A1    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | LDA ($nn),Y              | $B1    | 2         | 5+p
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn lda() -> &'static [MicroOp] {
        &[MicroOp {
            name: "lda",
            micro_fn: |cpu, bus| {
                let value = bus.mem_read(cpu, cpu.effective_addr);
                cpu.a = value;
                cpu.p.set_zn(value);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// LDX - Load Index Register X From Memory
    /// Operation: M → X
    ///
    /// Load the index register X from memory.
    ///
    /// LDX does not affect the C or V flags; sets Z if the value loaded was zero,
    /// otherwise resets it; sets N if the value loaded in bit 7 is a 1; otherwise
    /// N is reset, and affects only the X register.
    ///
    /// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate               | LDX #$nn                 | $A2    | 2         | 2
    /// Absolute                | LDX $nnnn                | $AE    | 3         | 4
    /// Y-Indexed Absolute      | LDX $nnnn,Y              | $BE    | 3         | 4+p
    /// Zero Page               | LDX $nn                  | $A6    | 2         | 3
    /// Y-Indexed Zero Page     | LDX $nn,Y                | $B6    | 2         | 4
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn ldx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "ldx",
            micro_fn: |cpu, bus| {
                let value = bus.mem_read(cpu, cpu.effective_addr);
                cpu.x = value;
                cpu.p.set_zn(value);
            },
        }]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// LDY - Load Index Register Y From Memory
    /// Operation: M → Y
    ///
    /// Load the index register Y from memory.
    ///
    /// LDY does not affect the C or V flags, sets the N flag if the value loaded in
    /// bit 7 is a 1, otherwise resets N, sets Z flag if the loaded value is zero
    /// otherwise resets Z and only affects the Y register.
    ///
    /// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------- | ------------------------ | ------ | --------- | ----------
    /// Immediate               | LDY #$nn                 | $A0    | 2         | 2
    /// Absolute                | LDY $nnnn                | $AC    | 3         | 4
    /// X-Indexed Absolute      | LDY $nnnn,X              | $BC    | 3         | 4+p
    /// Zero Page               | LDY $nn                  | $A4    | 2         | 3
    /// X-Indexed Zero Page     | LDY $nn,X                | $B4    | 2         | 4
    ///
    /// p: =1 if page is crossed.
    pub(crate) const fn ldy() -> &'static [MicroOp] {
        &[MicroOp {
            name: "ldy",
            micro_fn: |cpu, bus| {
                let value = bus.mem_read(cpu, cpu.effective_addr);
                cpu.y = value;
                cpu.p.set_zn(value);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// SAX - Store Accumulator "AND" Index Register X in Memory
    /// Operation: A & X → M
    ///
    /// The undocumented SAX instruction performs a bit-by-bit AND operation of the
    /// value of the accumulator and the value of the index register X and stores
    /// the result in memory.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode                | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------------------ | ------------------------ | ------ | --------- | ----------
    /// Absolute                       | SAX $nnnn                | $8F*   | 3         | 4
    /// Zero Page                      | SAX $nn                  | $87*   | 2         | 3
    /// Y-Indexed Zero Page            | SAX $nn,Y                | $97*   | 2         | 4
    /// X-Indexed Zero Page Indirect   | SAX ($nn,X)              | $83*   | 2         | 6
    ///
    /// *Undocumented.
    pub(crate) const fn sax() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sax",
            micro_fn: |cpu, bus| {
                let value = cpu.a & cpu.x;
                bus.mem_write(cpu, cpu.effective_addr, value);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// SHA - Store Accumulator "AND" Index Register X "AND" Value
    /// Operation: A ∧ X ∧ V → M
    ///
    /// The undocumented SHA instruction performs a bit-by-bit AND operation of the
    /// following three operands: The first two are the accumulator and the index
    /// register X.
    ///
    /// The third operand depends on the addressing mode.
    /// - In the zero page indirect Y-indexed case, the third operand is the data in
    ///   memory at the given zero page address (ignoring the addressing mode's Y
    ///   offset) plus 1.
    /// - In the Y-indexed absolute case, it is the upper 8 bits of the given address
    ///   (ignoring the addressing mode's Y offset), plus 1.
    ///
    /// It then transfers the result to the addressed memory location.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode                | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------------------ | ------------------------ | ------ | --------- | ----------
    /// Y-Indexed Absolute             | SHA $nnnn,Y              | $9F*   | 3         | 5
    /// Zero Page Indirect Y-Indexed   | SHA ($nn),Y              | $93*   | 2         | 6
    ///
    /// *Undocumented.
    pub(crate) const fn sha() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sha",
            micro_fn: |cpu, bus| {
                let hi = cpu.base;
                let value = cpu.a & cpu.x & hi.wrapping_add(1);
                bus.mem_write(cpu, cpu.effective_addr, value);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// SHX - Store Index Register X "AND" Value
    /// Operation: X ∧ (H + 1) → M
    ///
    /// The undocumented SHX instruction performs a bit-by-bit AND operation of the
    /// index register X and the upper 8 bits of the given address (ignoring the
    /// addressing mode's Y offset), plus 1. It then transfers the result to the
    /// addressed memory location.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Y-Indexed Absolute  | SHX $nnnn,Y              | $9E*   | 3         | 5
    ///
    /// *Undocumented.
    pub(crate) const fn shx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "shx",
            micro_fn: |cpu, bus| {
                // Reconstruct base operand address before applying Y index.
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let lo = base as u8;
                let hi = (base >> 8) as u8;

                // Undocumented behaviour: the high byte of the store address is
                // masked by X+1, producing the "SXA" addressing quirk.
                let addr_hi = hi & cpu.x.wrapping_add(1);
                let addr_lo = lo.wrapping_add(cpu.y);
                let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

                // Stored value: X & (H+1), where H is the original operand high byte.
                let value = cpu.x & hi.wrapping_add(1);
                bus.mem_write(cpu, addr, value);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// SHY - Store Index Register Y "AND" Value
    /// Operation: Y ∧ (H + 1) → M
    ///
    /// The undocumented SHY instruction performs a bit-by-bit AND operation of the
    /// index register Y and the upper 8 bits of the given address (ignoring the
    /// addressing mode's X offset), plus 1. It then transfers the result to the
    /// addressed memory location.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// X-Indexed Absolute  | SHY $nnnn,X              | $9C*   | 3         | 5
    ///
    /// *Undocumented.
    pub(crate) const fn shy() -> &'static [MicroOp] {
        &[MicroOp {
            name: "shy",
            micro_fn: |cpu, bus| {
                // Reconstruct base operand address before applying X index.
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let lo = base as u8;
                let hi = (base >> 8) as u8;

                // Undocumented behaviour: the high byte of the store address is
                // masked by Y+1, producing the "SYA" addressing quirk.
                let addr_hi = hi & cpu.y.wrapping_add(1);
                let addr_lo = lo.wrapping_add(cpu.x);
                let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

                // Stored value: Y & (H+1), where H is the original operand high byte.
                let value = cpu.y & hi.wrapping_add(1);
                bus.mem_write(cpu, addr, value);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// STA - Store Accumulator in Memory
    /// Operation: A → M
    ///
    /// This instruction transfers the contents of the accumulator to memory.
    ///
    /// This instruction affects none of the flags in the processor status register
    /// and does not affect the accumulator.
    ///
    /// Addressing Mode                     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ----------------------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute                            | STA $nnnn                | $8D    | 3         | 4
    /// X-Indexed Absolute                  | STA $nnnn,X              | $9D    | 3         | 5
    /// Y-Indexed Absolute                  | STA $nnnn,Y              | $99    | 3         | 5
    /// Zero Page                           | STA $nn                  | $85    | 2         | 3
    /// X-Indexed Zero Page                 | STA $nn,X                | $95    | 2         | 4
    /// X-Indexed Zero Page Indirect        | STA ($nn,X)              | $81    | 2         | 6
    /// Zero Page Indirect Y-Indexed        | STA ($nn),Y              | $91    | 2         | 6
    pub(crate) const fn sta() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sta",
            micro_fn: |cpu, bus| {
                bus.mem_write(cpu, cpu.effective_addr, cpu.a);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// STX - Store Index Register X In Memory
    /// Operation: X → M
    ///
    /// Transfers value of X register to addressed memory location.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute            | STX $nnnn                | $8E    | 3         | 4
    /// Zero Page           | STX $nn                  | $86    | 2         | 3
    /// Y-Indexed Zero Page | STX $nn,Y                | $96    | 2         | 4
    pub(crate) const fn stx() -> &'static [MicroOp] {
        &[MicroOp {
            name: "stx",
            micro_fn: |cpu, bus| {
                bus.mem_write(cpu, cpu.effective_addr, cpu.x);
            },
        }]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// STY - Store Index Register Y In Memory
    /// Operation: Y → M
    ///
    /// Transfer the value of the Y register to the addressed memory location.
    ///
    /// STY does not affect any flags or registers in the microprocessor.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute            | STY $nnnn                | $8C    | 3         | 4
    /// Zero Page           | STY $nn                  | $84    | 2         | 3
    /// X-Indexed Zero Page | STY $nn,X                | $94    | 2         | 4
    pub(crate) const fn sty() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sty",
            micro_fn: |cpu, bus| {
                bus.mem_write(cpu, cpu.effective_addr, cpu.y);
            },
        }]
    }
}

#[cfg(test)]
mod load_tests {

    use crate::cpu::mnemonic::{Mnemonic, tests::InstrTest};

    #[test]
    fn test_las() {
        InstrTest::new(Mnemonic::LAS).test(|verify, cpu, _| {
            let v = verify.m & verify.cpu.s;
            assert_eq!(cpu.a, v);
            assert_eq!(cpu.x, v);
            assert_eq!(cpu.s, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_lax() {
        InstrTest::new(Mnemonic::LAX).test(|verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.a, m);
            assert_eq!(cpu.x, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_lda() {
        InstrTest::new(Mnemonic::LDA).test(|verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.a, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_ldx() {
        InstrTest::new(Mnemonic::LDX).test(|verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.x, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_ldy() {
        InstrTest::new(Mnemonic::LDY).test(|verify, cpu, _| {
            let m = verify.m;
            assert_eq!(cpu.y, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_sax() {
        InstrTest::new(Mnemonic::SAX).test(|verify, cpu, bus| {
            let v = verify.cpu.a & verify.cpu.x;
            let m = bus.mem_read(cpu, verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sha() {
        InstrTest::new(Mnemonic::SHA).test(|verify, cpu, bus| {
            let v = verify.cpu.a & verify.cpu.x & verify.addr_hi.wrapping_add(1);
            let m = bus.mem_read(cpu, verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_shx() {
        InstrTest::new(Mnemonic::SHX).test(|verify, cpu, bus| {
            // Reconstruct base operand address before applying Y index.
            let base = verify.addr.wrapping_sub(verify.cpu.y as u16);
            let lo = base as u8;
            let hi = (base >> 8) as u8;

            let addr_hi = hi & verify.cpu.x.wrapping_add(1);
            let addr_lo = lo.wrapping_add(verify.cpu.y);
            let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

            let v = verify.cpu.x & hi.wrapping_add(1);
            let m = bus.mem_read(cpu, addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_shy() {
        InstrTest::new(Mnemonic::SHY).test(|verify, cpu, bus| {
            // Reconstruct base operand address before applying X index.
            let base = verify.addr.wrapping_sub(verify.cpu.x as u16);
            let lo = base as u8;
            let hi = (base >> 8) as u8;

            let addr_hi = hi & verify.cpu.y.wrapping_add(1);
            let addr_lo = lo.wrapping_add(verify.cpu.x);
            let addr = ((addr_hi as u16) << 8) | addr_lo as u16;

            let v = verify.cpu.y & hi.wrapping_add(1);
            let m = bus.mem_read(cpu, addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sta() {
        InstrTest::new(Mnemonic::STA).test(|verify, cpu, bus| {
            let v = verify.cpu.a;
            let m = bus.mem_read(cpu, verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_stx() {
        InstrTest::new(Mnemonic::STX).test(|verify, cpu, bus| {
            let v = verify.cpu.x;
            let m = bus.mem_read(cpu, verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_sty() {
        InstrTest::new(Mnemonic::STY).test(|verify, cpu, bus| {
            let v = verify.cpu.y;
            let m = bus.mem_read(cpu, verify.addr);
            assert_eq!(v, m);
        });
    }
}
