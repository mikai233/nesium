use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{Cpu, micro_op::MicroOp, mnemonic::Mnemonic},
};

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
#[inline]
pub fn exec_shs(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            let s = cpu.a & cpu.x;
            cpu.s = s;
            let m = s & cpu.base.wrapping_add(1);
            bus.mem_write(cpu.effective_addr, m, cpu, ctx);
        }
        _ => unreachable_step!("invalid SHS step {step}"),
    }
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
#[inline]
pub fn exec_tax(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.x = cpu.a;
            cpu.p.set_zn(cpu.x);
        }
        _ => unreachable_step!("invalid TAX step {step}"),
    }
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
#[inline]
pub fn exec_tay(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.y = cpu.a;
            cpu.p.set_zn(cpu.y);
        }
        _ => unreachable_step!("invalid TAY step {step}"),
    }
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
#[inline]
pub fn exec_tsx(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.x = cpu.s;
            cpu.p.set_zn(cpu.x);
        }
        _ => unreachable_step!("invalid TSX step {step}"),
    }
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
#[inline]
pub fn exec_txa(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.a = cpu.x;
            cpu.p.set_zn(cpu.a);
        }
        _ => unreachable_step!("invalid TXA step {step}"),
    }
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
#[inline]
pub fn exec_txs(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.s = cpu.x;
        }
        _ => unreachable_step!("invalid TXS step {step}"),
    }
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
#[inline]
pub fn exec_tya(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.internal_cycle(cpu, ctx);
            cpu.a = cpu.y;
            cpu.p.set_zn(cpu.a);
        }
        _ => unreachable_step!("invalid TYA step {step}"),
    }
}

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
        &[MicroOp {
            name: "shs",
            micro_fn: |cpu, bus, ctx| {
                let s = cpu.a & cpu.x;
                cpu.s = s;
                let m = s & cpu.base.wrapping_add(1);
                bus.mem_write(cpu.effective_addr, m, cpu, ctx);
            },
        }]
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
        &[MicroOp {
            name: "tax",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.x = cpu.a;
                cpu.p.set_zn(cpu.x);
            },
        }]
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
        &[MicroOp {
            name: "tay",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.y = cpu.a;
                cpu.p.set_zn(cpu.y);
            },
        }]
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
        &[MicroOp {
            name: "tsx",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.x = cpu.s;
                cpu.p.set_zn(cpu.x);
            },
        }]
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
        &[MicroOp {
            name: "txa",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.a = cpu.x;
                cpu.p.set_zn(cpu.a);
            },
        }]
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
        &[MicroOp {
            name: "txs",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.s = cpu.x;
            },
        }]
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
        &[MicroOp {
            name: "tya",
            micro_fn: |cpu, bus, ctx| {
                bus.internal_cycle(cpu, ctx);
                cpu.a = cpu.y;
                cpu.p.set_zn(cpu.a);
            },
        }]
    }
}
