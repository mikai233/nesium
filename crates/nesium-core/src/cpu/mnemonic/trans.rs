use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

impl Mnemonic {
    /// NV-BDIZC
    /// --------
    ///
    /// SHS - Transfer Accumulator "AND" Index Register X to Stack Pointer then Store Stack Pointer "AND" Hi-Byte In Memory
    /// Operation: A ∧ X → S, S ∧ (H + 1) → M
    ///
    /// The undocumented SHS instruction performs a bit-by-bit AND operation of the
    /// value of the accumulator and the value of the index register X and stores
    /// the result in the stack pointer. It then performs a bit-by-bit AND operation
    /// of the resulting stack pointer and the upper 8 bits of the given address
    /// (ignoring the addressing mode's Y offset), plus 1, and transfers the result
    /// to the addressed memory location.
    ///
    /// No flags or registers in the microprocessor are affected by the store
    /// operation.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Y-Indexed Absolute  | SHS $nnnn,Y              | $9B*   | 3         | 5
    ///
    /// *Undocumented.
    pub(crate) const fn shs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "shs",
            micro_fn: |cpu, bus| {
                let s = cpu.a & cpu.x;
                cpu.s = s;
                let m = s & cpu.base.wrapping_add(1);
                bus.write(cpu.effective_addr, m);
            },
        };
        &[OP1]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// TAX - Transfer Accumulator To Index X
    /// Operation: A → X
    ///
    /// This instruction takes the value from accumulator A and transfers or loads
    /// it into the index register X without disturbing the content of the
    /// accumulator A.
    ///
    /// TAX only affects the index register X, does not affect the carry or overflow
    /// flags. The N flag is set if the resultant value in the index register X has
    /// bit 7 on, otherwise N is reset. The Z bit is set if the content of the
    /// register X is 0 as a result of the operation, otherwise it is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TAX                      | $AA    | 1         | 2
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

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// TAY - Transfer Accumulator To Index Y
    /// Operation: A → Y
    ///
    /// This instruction moves the value of the accumulator into index register Y
    /// without affecting the accumulator.
    ///
    /// TAY instruction only affects the Y register and does not affect either the
    /// carry or overflow flags. If the index register Y has bit 7 on, then N is set,
    /// otherwise it is reset. If the content of the index register Y equals 0 as a
    /// result of the operation, Z is set on, otherwise it is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TAY                      | $A8    | 1         | 2
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

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// TSX - Transfer Stack Pointer To Index X
    /// Operation: S → X
    ///
    /// This instruction transfers the value in the stack pointer to the index
    /// register X.
    ///
    /// TSX does not affect the carry or overflow flags. It sets N if bit 7 is on in
    /// index X as a result of the instruction, otherwise it is reset. If index X is
    /// zero as a result of the TSX, the Z flag is set, otherwise it is reset. TSX
    /// changes the value of index X, making it equal to the content of the stack
    /// pointer.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TSX                      | $BA    | 1         | 2
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

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// TXA - Transfer Index X To Accumulator
    /// Operation: X → A
    ///
    /// This instruction moves the value that is in the index register X to the
    /// accumulator A without disturbing the content of the index register X.
    ///
    /// TXA does not affect any register other than the accumulator and does not
    /// affect the carry or overflow flag. If the result in A has bit 7 on, then the
    /// N flag is set, otherwise it is reset. If the resultant value in the
    /// accumulator is 0, then the Z flag is set, otherwise it is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TXA                      | $8A    | 1         | 2
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

    /// NV-BDIZC
    /// --------
    ///
    /// TXS - Transfer Index X To Stack Pointer
    /// Operation: X → S
    ///
    /// This instruction transfers the value in the index register X to the stack
    /// pointer.
    ///
    /// TXS changes only the stack pointer, making it equal to the content of the
    /// index register X. It does not affect any of the flags.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TXS                      | $9A    | 1         | 2
    pub(crate) const fn txs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "txs",
            micro_fn: |cpu, _| {
                cpu.s = cpu.x;
            },
        };
        &[OP1]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// TYA - Transfer Index Y To Accumulator
    /// Operation: Y → A
    ///
    /// This instruction moves the value that is in the index register Y to
    /// accumulator A without disturbing the content of the register Y.
    ///
    /// TYA does not affect any other register other than the accumulator and does
    /// not affect the carry or overflow flag. If the result in the accumulator A has
    /// bit 7 on, the N flag is set, otherwise it is reset. If the resultant value
    /// in the accumulator A is 0, then the Z flag is set, otherwise it is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | TYA                      | $98    | 1         | 2
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

#[cfg(test)]
mod trans_tests {
    use crate::{
        bus::Bus,
        cpu::mnemonic::{Mnemonic, tests::InstrTest},
    };

    #[test]
    fn test_shs() {
        InstrTest::new(Mnemonic::SHS).test(|verify, cpu, bus| {
            let v = verify.cpu.a & verify.cpu.x;
            assert_eq!(cpu.s, v);
            let v = v & verify.addr_hi.wrapping_add(1);
            let m = bus.read(verify.addr);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_tax() {
        InstrTest::new(Mnemonic::TAX).test(|verify, cpu, bus| {
            let v = verify.cpu.a;
            assert_eq!(cpu.x, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_tay() {
        InstrTest::new(Mnemonic::TAY).test(|verify, cpu, bus| {
            let v = verify.cpu.a;
            assert_eq!(cpu.y, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_tsx() {
        InstrTest::new(Mnemonic::TSX).test(|verify, cpu, bus| {
            let v = verify.cpu.s;
            assert_eq!(cpu.x, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_txa() {
        InstrTest::new(Mnemonic::TXA).test(|verify, cpu, bus| {
            let v = verify.cpu.x;
            assert_eq!(cpu.a, v);
            verify.check_nz(cpu.p, v);
        });
    }

    #[test]
    fn test_txs() {
        InstrTest::new(Mnemonic::TXS).test(|verify, cpu, bus| {
            let v = verify.cpu.x;
            assert_eq!(cpu.s, v);
        });
    }

    #[test]
    fn test_tya() {
        InstrTest::new(Mnemonic::TYA).test(|verify, cpu, bus| {
            let v = verify.cpu.y;
            assert_eq!(cpu.a, v);
            verify.check_nz(cpu.p, v);
        });
    }
}
