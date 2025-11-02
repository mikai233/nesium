use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic},
};

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
        const OP1: MicroOp = MicroOp {
            name: "dec",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr).wrapping_sub(1);
                bus.write(cpu.effective_addr, value);
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
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
        const OP1: MicroOp = MicroOp {
            name: "dex",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_sub(1);
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
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
        const OP1: MicroOp = MicroOp {
            name: "dey",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_sub(1);
                cpu.p.set_zn(cpu.y);
            },
        };
        &[OP1]
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
        const OP1: MicroOp = MicroOp {
            name: "inc",
            micro_fn: |cpu, bus| {
                let value = bus.read(cpu.effective_addr).wrapping_add(1);
                bus.write(cpu.effective_addr, value);
                cpu.p.set_zn(value);
            },
        };
        &[OP1]
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
        const OP1: MicroOp = MicroOp {
            name: "inx",
            micro_fn: |cpu, _| {
                cpu.x = cpu.x.wrapping_add(1);
                cpu.p.set_zn(cpu.x);
            },
        };
        &[OP1]
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
        const OP1: MicroOp = MicroOp {
            name: "iny",
            micro_fn: |cpu, _| {
                cpu.y = cpu.y.wrapping_add(1);
                cpu.p.set_zn(cpu.y);
            },
        };
        &[OP1]
    }
}
