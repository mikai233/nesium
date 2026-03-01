//! # NES/Ricoh 2A03 CPU Emulation: Cycle-Accurate Stack Operations
//!
//! This module implements the cycle-accurate behavior of stack PUSH (PHA, PHP)
//! and PULL (PLA, PLP) instructions for the NMOS 6502 architecture (used in the NES/Famicom).
//!
//! Due to the 6502's design constraint—requiring a bus access on *every* clock cycle—
//! internal operations (like register setup or pointer arithmetic) are often "filled"
//! with dummy memory reads or writes. This leads to the non-obvious cycle counts.
//!
//! ## 1. PUSH Operations (PHA, PHP) - 3 Cycles Total
//!
//! PUSH operations (Write to Stack) require one extra cycle for internal setup, resulting in 3 total cycles:
//!
//! | Cycle | Bus Action | Address (A) | Data (D) | Purpose                                                      |
//! |-------|------------|-------------|----------|--------------------------------------------------------------|
//! | T1    | Read       | PC          | Opcode   | Fetch the opcode. PC increments.                             |
//! | T2    | Read       | PC + 1      | Junk     | **Internal Setup:** CPU prepares data/address; performs a dummy read from the program counter's next byte (data is discarded). |
//! | T3    | Write      | $01XX       | P/A      | **Execute:** Write data to the Stack; Stack Pointer (SP) decrements. |
//!
//! ## 2. PULL Operations (PLA, PLP) - 4 Cycles Total
//!
//! PULL operations (Read from Stack) require two extra cycles: one for setup and one for Stack Pointer increment, resulting in 4 total cycles:
//!
//! | Cycle | Bus Action | Address (A) | Data (D) | Purpose                                                      |
//! |-------|------------|-------------|----------|--------------------------------------------------------------|
//! | T1    | Read       | PC          | Opcode   | Fetch the opcode. PC increments.                             |
//! | T2    | Read       | PC + 1      | Junk     | **Internal Setup:** CPU prepares to operate. Dummy read from PC+1 (data is discarded). |
//! | T3    | Read       | $01XX       | Junk     | **SP Increment:** CPU increments SP; performs a dummy read from the *old* stack address (data is discarded). |
//! | T4    | Read       | $01XX+1     | Data     | **Execute:** Pull data from the *new* stack address into the target register (A or P). |
//!
//! **Warning:** For cycle-accurate NES emulation, especially when handling Memory-Mapped I/O (MMIO) like the PPU/APU registers, these dummy memory accesses (T2, T3) must be simulated, as they consume crucial clock cycles.

use crate::{
    bus::{CpuBus, STACK_ADDR},
    cartridge::CpuBusAccessKind,
    context::Context,
    cpu::{Cpu, status::Status},
};

/// NV-BDIZC
/// --------
///
/// PHA - Push Accumulator On Stack
/// Operation: A↓
///
/// This instruction transfers the current value of the accumulator to the next
/// location on the stack, automatically decrementing the stack to point to the
/// next empty location.
///
/// The Push A instruction only affects the stack pointer register which is
/// decremented by 1 as a result of the operation. It affects no flags.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ------------------------ | ------ | --------- | ----------
/// Implied         | PHA                      | $48    | 1         | 3
#[inline]
pub fn exec_pha(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            cpu.push_stack(bus, ctx, cpu.a);
        }
        _ => unreachable_step!("invalid PHA step {step}"),
    }
}

/// NV-BDIZC
/// --------
///
/// PHP - Push Processor Status On Stack
/// Operation: P↓
///
/// This instruction transfers the contents of the processor status register
/// unchanged to the stack, as governed by the stack pointer.
///
/// The PHP instruction affects no registers or flags in the microprocessor.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ------------------------ | ------ | --------- | ----------
/// Implied         | PHP                      | $08    | 1         | 3
#[inline]
pub fn exec_php(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            let p = cpu.p | Status::BREAK | Status::UNUSED;
            cpu.push_stack(bus, ctx, p.bits());
        }
        _ => unreachable_step!("invalid PHP step {step}"),
    }
}

/// NV-BDIZC
/// ✓-----✓-
///
/// PLA - Pull Accumulator From Stack
/// Operation: A↑
///
/// This instruction adds 1 to the current value of the stack pointer and uses it
/// to address the stack and loads the contents of the stack into the A register.
///
/// The PLA instruction does not affect the carry or overflow flags. It sets N if
/// the bit 7 is on in accumulator A as a result of instructions, otherwise it is
/// reset. If accumulator A is zero as a result of the PLA, then the Z flag is
/// set, otherwise it is reset. The PLA instruction changes content of the
/// accumulator A to the contents of the memory location at stack register plus 1
/// and also increments the stack register.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ------------------------ | ------ | --------- | ----------
/// Implied         | PLA                      | $68    | 1         | 4
#[inline]
pub fn exec_pla(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            cpu.read(
                STACK_ADDR | cpu.s as u16,
                bus,
                ctx,
                CpuBusAccessKind::DummyRead,
            );
        }
        2 => {
            let value = cpu.pop_stack(bus, ctx);
            cpu.a = value;
            cpu.p.set_zn(value);
        }
        _ => unreachable_step!("invalid PLA step {step}"),
    }
}

/// NV-BDIZC
/// ✓✓--✓✓✓✓
///
/// PLP - Pull Processor Status From Stack
/// Operation: P↑
///
/// This instruction transfers the next value on the stack to the Processor Status
/// register, thereby changing all of the flags and setting the mode switches to
/// the values from the stack.
///
/// The PLP instruction affects no registers in the processor other than the
/// status register. This instruction could affect all flags in the status
/// register.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ------------------------ | ------ | --------- | ----------
/// Implied         | PLP                      | $28    | 1         | 4
#[inline]
pub fn exec_plp(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            cpu.read(
                STACK_ADDR | cpu.s as u16,
                bus,
                ctx,
                CpuBusAccessKind::DummyRead,
            );
        }
        2 => {
            let value = cpu.pop_stack(bus, ctx);
            cpu.p = Status::from_bits_truncate(value);
            cpu.p.remove(Status::UNUSED | Status::BREAK);
        }
        _ => unreachable_step!("invalid PLP step {step}"),
    }
}
