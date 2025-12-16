use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{
        Cpu,
        status::{BIT_6, BIT_7},
    },
};

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
#[inline]
pub fn exec_and(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            let m = bus.mem_read(cpu.effective_addr, cpu, ctx);
            cpu.a &= m;
            cpu.p.set_zn(cpu.a);
        }
        _ => unreachable_step!("invalid AND step {step}"),
    }
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
#[inline]
pub fn exec_bit(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            let m = bus.mem_read(cpu.effective_addr, cpu, ctx);
            let and = cpu.a & m;
            cpu.p.set_z(and == 0);
            cpu.p.set_n(m & BIT_7 != 0);
            cpu.p.set_v(m & BIT_6 != 0);
        }
        _ => unreachable_step!("invalid BIT step {step}"),
    }
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
#[inline]
pub fn exec_eor(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            let m = bus.mem_read(cpu.effective_addr, cpu, ctx);
            cpu.a ^= m;
            cpu.p.set_zn(cpu.a);
        }
        _ => unreachable_step!("invalid EOR step {step}"),
    }
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
#[inline]
pub fn exec_ora(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            let m = bus.mem_read(cpu.effective_addr, cpu, ctx);
            cpu.a |= m;
            cpu.p.set_zn(cpu.a);
        }
        _ => unreachable_step!("invalid ORA step {step}"),
    }
}
