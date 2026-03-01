use crate::{
    bus::CpuBus,
    cartridge::CpuBusAccessKind,
    context::Context,
    cpu::{
        Cpu,
        status::{BIT_0, BIT_7, Status},
    },
};

/// NV-BDIZC
/// ✓-----✓✓
///
/// ASL - Arithmetic Shift Left
/// Operation: C ← /M7...M0/ ← 0
///
/// The shift left instruction shifts either the accumulator or the address
/// memory location 1 bit to the left, with the bit 0 always being set to 0 and
/// the input bit 7 being stored in the carry flag. ASL either shifts the
/// accumulator left 1 bit or is a read/modify/write instruction that affects
/// only memory.
///
/// The instruction does not affect the overflow bit, sets N equal to the result
/// bit 7 (bit 6 in the input), sets Z flag if the result is equal to 0,
/// otherwise resets Z and stores the input bit 7 in the carry flag.
///
/// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// ----------------------- | ------------------------ | ------ | --------- | ----------
/// Accumulator             | ASL A                    | $0A    | 1         | 2
/// Absolute                | ASL $nnnn                | $0E    | 3         | 6
/// X-Indexed Absolute      | ASL $nnnn,X              | $1E    | 3         | 7
/// Zero Page               | ASL $nn                  | $06    | 2         | 5
/// X-Indexed Zero Page     | ASL $nn,X                | $16    | 2         | 6
#[inline]
pub fn exec_asl(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
        }
        1 => {
            cpu.dummy_write(cpu.effective_addr, cpu.tmp, bus, ctx);
        }
        2 => {
            if cpu.opcode_in_flight == Some(0x0A) {
                cpu.dummy_read(bus, ctx);
                cpu.p.set_c(cpu.a & BIT_7 != 0);
                cpu.a <<= 1;
                cpu.p.set_zn(cpu.a);
            } else {
                let old_bit7 = cpu.tmp & BIT_7;
                cpu.tmp <<= 1;
                cpu.p.set_c(old_bit7 != 0);
                cpu.p.set_zn(cpu.tmp);
                cpu.write(
                    cpu.effective_addr,
                    cpu.tmp,
                    bus,
                    ctx,
                    CpuBusAccessKind::Write,
                );
            }
        }
        _ => unreachable_step!("invalid ASL step {step}"),
    }
}

/// NV-BDIZC
/// 0-----✓✓
///
/// LSR - Logical Shift Right
/// Operation: 0 → /M7...M0/ → C
///
/// This instruction shifts either the accumulator or a specified memory location
/// 1 bit to the right, with the higher bit of the result always being set to 0,
/// and the low bit which is shifted out of the field being stored in the carry
/// flag.
///
/// The shift right instruction either affects the accumulator by shifting it
/// right 1 or is a read/modify/write instruction which changes a specified
/// memory location but does not affect any internal registers. The shift right
/// does not affect the overflow flag. The N flag is always reset. The Z flag is
/// set if the result of the shift is 0 and reset otherwise. The carry is set
/// equal to bit 0 of the input.
///
/// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// ----------------------- | ------------------------ | ------ | --------- | ----------
/// Accumulator             | LSR A                    | $4A    | 1         | 2
/// Absolute                | LSR $nnnn                | $4E    | 3         | 6
/// X-Indexed Absolute      | LSR $nnnn,X              | $5E    | 3         | 7
/// Zero Page               | LSR $nn                  | $46    | 2         | 5
/// X-Indexed Zero Page     | LSR $nn,X                | $56    | 2         | 6
#[inline]
pub fn exec_lsr(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
        }
        1 => {
            cpu.dummy_write(cpu.effective_addr, cpu.tmp, bus, ctx);
        }
        2 => {
            if cpu.opcode_in_flight == Some(0x4A) {
                cpu.dummy_read(bus, ctx);
                cpu.p.set_c(cpu.a & BIT_0 != 0);
                cpu.a >>= 1;
                cpu.p.remove(Status::NEGATIVE);
                cpu.p.set_z(cpu.a == 0);
            } else {
                let old_bit0 = cpu.tmp & BIT_0;
                cpu.tmp >>= 1;
                cpu.p.set_c(old_bit0 != 0);
                cpu.p.remove(Status::NEGATIVE);
                cpu.p.set_z(cpu.tmp == 0);
                cpu.write(
                    cpu.effective_addr,
                    cpu.tmp,
                    bus,
                    ctx,
                    CpuBusAccessKind::Write,
                );
            }
        }
        _ => unreachable_step!("invalid LSR step {step}"),
    }
}

/// NV-BDIZC
/// ✓-----✓✓
///
/// ROL - Rotate Left
/// Operation: C ← /M7...M0/ ← C
///
/// The rotate left instruction shifts either the accumulator or addressed memory
/// left 1 bit, with the input carry being stored in bit 0 and with the input bit
/// 7 being stored in the carry flags.
///
/// The ROL instruction either shifts the accumulator left 1 bit and stores the
/// carry in accumulator bit 0 or does not affect the internal registers at all.
/// The ROL instruction sets carry equal to the input bit 7, sets N equal to the
/// input bit 6, sets the Z flag if the result of the rotate is 0, otherwise it
/// resets Z and does not affect the overflow flag at all.
///
/// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// ----------------------- | ------------------------ | ------ | --------- | ----------
/// Accumulator             | ROL A                    | $2A    | 1         | 2
/// Absolute                | ROL $nnnn                | $2E    | 3         | 6
/// X-Indexed Absolute      | ROL $nnnn,X              | $3E    | 3         | 7
/// Zero Page               | ROL $nn                  | $26    | 2         | 5
/// X-Indexed Zero Page     | ROL $nn,X                | $36    | 2         | 6
#[inline]
pub fn exec_rol(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
        }
        1 => {
            cpu.dummy_write(cpu.effective_addr, cpu.tmp, bus, ctx);
        }
        2 => {
            if cpu.opcode_in_flight == Some(0x2A) {
                cpu.dummy_read(bus, ctx);
                let old_bit7 = cpu.a & BIT_7;
                let new_a = (cpu.a << 1) | if cpu.p.c() { 1 } else { 0 };
                cpu.a = new_a;
                cpu.p.set_c(old_bit7 != 0);
                cpu.p.set_zn(cpu.a);
            } else {
                let old_bit7 = cpu.tmp & BIT_7;
                let new_a = (cpu.tmp << 1) | if cpu.p.c() { 1 } else { 0 };
                cpu.tmp = new_a;
                cpu.p.set_c(old_bit7 != 0);
                cpu.p.set_zn(cpu.tmp);
                cpu.write(
                    cpu.effective_addr,
                    cpu.tmp,
                    bus,
                    ctx,
                    CpuBusAccessKind::Write,
                );
            }
        }
        _ => unreachable_step!("invalid ROL step {step}"),
    }
}

/// NV-BDIZC
/// ✓-----✓✓
///
/// ROR - Rotate Right
/// Operation: C → /M7...M0/ → C
///
/// The rotate right instruction shifts either the accumulator or addressed memory
/// right 1 bit with bit 0 shifted into the carry and carry shifted into bit 7.
///
/// The ROR instruction either shifts the accumulator right 1 bit and stores the
/// carry in accumulator bit 7 or does not affect the internal registers at all.
/// The ROR instruction sets carry equal to input bit 0, sets N equal to the input
/// carry and sets the Z flag if the result of the rotate is 0; otherwise it
/// resets Z and does not affect the overflow flag at all.
///
/// (Available on Microprocessors after June, 1976)
///
/// Addressing Mode         | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// ----------------------- | ------------------------ | ------ | --------- | ----------
/// Accumulator             | ROR A                    | $6A    | 1         | 2
/// Absolute                | ROR $nnnn                | $6E    | 3         | 6
/// X-Indexed Absolute      | ROR $nnnn,X              | $7E    | 3         | 7
/// Zero Page               | ROR $nn                  | $66    | 2         | 5
/// X-Indexed Zero Page     | ROR $nn,X                | $76    | 2         | 6
#[inline]
pub fn exec_ror(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
        }
        1 => {
            cpu.dummy_write(cpu.effective_addr, cpu.tmp, bus, ctx);
        }
        2 => {
            if cpu.opcode_in_flight == Some(0x6A) {
                cpu.dummy_read(bus, ctx);
                let old_bit0 = cpu.a & BIT_0;
                let new_a = (cpu.a >> 1) | if cpu.p.c() { BIT_7 } else { 0 };
                cpu.a = new_a;
                cpu.p.set_c(old_bit0 != 0);
                cpu.p.set_zn(cpu.a);
            } else {
                let old_bit0 = cpu.tmp & BIT_0;
                let new_a = (cpu.tmp >> 1) | if cpu.p.c() { BIT_7 } else { 0 };
                cpu.tmp = new_a;
                cpu.p.set_c(old_bit0 != 0);
                cpu.p.set_zn(cpu.tmp);
                cpu.write(
                    cpu.effective_addr,
                    cpu.tmp,
                    bus,
                    ctx,
                    CpuBusAccessKind::Write,
                );
            }
        }
        _ => unreachable_step!("invalid ROR step {step}"),
    }
}
