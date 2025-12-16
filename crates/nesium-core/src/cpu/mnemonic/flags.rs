use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{Cpu, micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

/// N V - B D I Z C
/// - - - - - - - 0
///
/// CLC - Clear Carry Flag
/// Operation: 0 → C
///
/// This instruction initializes the carry flag to a 0. This operation should
/// normally precede an ADC loop. It is also useful when used with a ROL
/// instruction to clear a bit in memory.
///
/// This instruction affects no registers in the microprocessor and no flags
/// other than the carry flag which is reset.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | CLC                    | $18    | 1         | 2
#[inline]
pub fn exec_clc(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.set_c(false);
        }
        _ => unreachable_step!("invalid CLC step {step}"),
    }
}

/// N V - B D I Z C
/// - - - - 0 - - -
///
/// CLD - Clear Decimal Mode
/// Operation: 0 → D
///
/// This instruction sets the decimal mode flag to a 0. This all subsequent
/// ADC and SBC instructions to operate as simple operations.
///
/// CLD affects no registers in the microprocessor and no flags other than the
/// decimal mode flag which is set to a 0.
///
/// **Note on the MOS 6502:**
///
/// The value of the decimal mode flag is indeterminate after a RESET.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | CLD                    | $D8    | 1         | 2
#[inline]
pub fn exec_cld(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.set_d(false);
        }
        _ => unreachable_step!("invalid CLD step {step}"),
    }
}

/// N V - B D I Z C
/// - - - - - 0 - -
///
/// CLI - Clear Interrupt Disable
/// Operation: 0 → I
///
/// This instruction initializes the interrupt disable to a 0. This allows the
/// microprocessor to receive interrupts.
///
/// It affects no registers in the microprocessor and no flags other than the
/// interrupt disable which is cleared.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | CLI                    | $58    | 1         | 2
#[inline]
pub fn exec_cli(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.remove(Status::INTERRUPT);
        }
        _ => unreachable_step!("invalid CLI step {step}"),
    }
}

/// N V - B D I Z C
/// - 0 - - - - - -
///
/// CLV - Clear Overflow Flag
/// Operation: 0 → V
///
/// This instruction clears the overflow flag to a 0. This command is used in
/// conjunction with the set overflow pin which can change the state of the
/// overflow flag with an external signal.
///
/// CLV affects no registers in the microprocessor and no flags other than the
/// overflow flag which is set to a 0.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | CLV                    | $B8    | 1         | 2
#[inline]
pub fn exec_clv(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.set_v(false);
        }
        _ => unreachable_step!("invalid CLV step {step}"),
    }
}

/// N V - B D I Z C
/// - - - - - - - 1
///
/// SEC - Set Carry Flag
/// Operation: 1 → C
///
/// This instruction initializes the carry flag to a 1. This operation should
/// normally precede a SBC loop. It is also useful when used with a ROL
/// instruction to initialize a bit in memory to a 1.
///
/// This instruction affects no registers in the microprocessor and no flags
/// other than the carry flag which is set.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | SEC                    | $38    | 1         | 2
#[inline]
pub fn exec_sec(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.set_c(true);
        }
        _ => unreachable_step!("invalid SEC step {step}"),
    }
}

/// N V - B D I Z C
/// - - - - 1 - - -
///
/// SED - Set Decimal Mode
/// Operation: 1 → D
///
/// This instruction sets the decimal mode flag D to a 1. This makes all
/// subsequent ADC and SBC instructions operate as a decimal arithmetic
/// operation.
///
/// SED affects no registers in the microprocessor and no flags other than the
/// decimal mode which is set to a 1.
///
/// **Note on the MOS 6502:**
///
/// The value of the decimal mode flag is indeterminate after a RESET.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | SED                    | $F8    | 1         | 2
#[inline]
pub fn exec_sed(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.set_d(true);
        }
        _ => unreachable_step!("invalid SED step {step}"),
    }
}

/// N V - B D I Z C
/// - - - - - 1 - -
///
/// SEI - Set Interrupt Disable
/// Operation: 1 → I
///
/// This instruction initializes the interrupt disable to a 1. It is used to
/// mask interrupt requests during system reset operations and during interrupt
/// commands.
///
/// It affects no registers in the microprocessor and no flags other than the
/// interrupt disable which is set.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | SEI                    | $78    | 1         | 2
#[inline]
pub fn exec_sei(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
            cpu.p.insert(Status::INTERRUPT);
        }
        _ => unreachable_step!("invalid SEI step {step}"),
    }
}

impl Mnemonic {
    /// N V - B D I Z C
    /// - - - - - - - 0
    ///
    /// CLC - Clear Carry Flag
    /// Operation: 0 → C
    ///
    /// This instruction initializes the carry flag to a 0. This operation should
    /// normally precede an ADC loop. It is also useful when used with a ROL
    /// instruction to clear a bit in memory.
    ///
    /// This instruction affects no registers in the microprocessor and no flags
    /// other than the carry flag which is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLC                    | $18    | 1         | 2
    pub(crate) const fn clc() -> &'static [MicroOp] {
        &[MicroOp {
            name: "clc_clear_carry",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                // Cycle 2: C = 0
                cpu.p.set_c(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - 0 - - -
    ///
    /// CLD - Clear Decimal Mode
    /// Operation: 0 → D
    ///
    /// This instruction sets the decimal mode flag to a 0. This all subsequent
    /// ADC and SBC instructions to operate as simple operations.
    ///
    /// CLD affects no registers in the microprocessor and no flags other than the
    /// decimal mode flag which is set to a 0.
    ///
    /// **Note on the MOS 6502:**
    ///
    /// The value of the decimal mode flag is indeterminate after a RESET.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLD                    | $D8    | 1         | 2
    pub(crate) const fn cld() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cld_clear_decimal",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                // Cycle 2: D = 0
                cpu.p.set_d(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - 0 - -
    ///
    /// CLI - Clear Interrupt Disable
    /// Operation: 0 → I
    ///
    /// This instruction initializes the interrupt disable to a 0. This allows the
    /// microprocessor to receive interrupts.
    ///
    /// It affects no registers in the microprocessor and no flags other than the
    /// interrupt disable which is cleared.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLI                    | $58    | 1         | 2
    pub(crate) const fn cli() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cli_clear_interrupt",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                cpu.p.remove(Status::INTERRUPT);
            },
        }]
    }

    /// N V - B D I Z C
    /// - 0 - - - - - -
    ///
    /// CLV - Clear Overflow Flag
    /// Operation: 0 → V
    ///
    /// This instruction clears the overflow flag to a 0. This command is used in
    /// conjunction with the set overflow pin which can change the state of the
    /// overflow flag with an external signal.
    ///
    /// CLV affects no registers in the microprocessor and no flags other than the
    /// overflow flag which is set to a 0.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLV                    | $B8    | 1         | 2
    pub(crate) const fn clv() -> &'static [MicroOp] {
        &[MicroOp {
            name: "clv_clear_overflow",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                // Cycle 2: V = 0
                cpu.p.set_v(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - - - 1
    ///
    /// SEC - Set Carry Flag
    /// Operation: 1 → C
    ///
    /// This instruction initializes the carry flag to a 1. This operation should
    /// normally precede a SBC loop. It is also useful when used with a ROL
    /// instruction to initialize a bit in memory to a 1.
    ///
    /// This instruction affects no registers in the microprocessor and no flags
    /// other than the carry flag which is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SEC                    | $38    | 1         | 2
    pub(crate) const fn sec() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sec_set_carry",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                // Cycle 2: C = 1
                cpu.p.set_c(true);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - 1 - - -
    ///
    /// SED - Set Decimal Mode
    /// Operation: 1 → D
    ///
    /// This instruction sets the decimal mode flag D to a 1. This makes all
    /// subsequent ADC and SBC instructions operate as a decimal arithmetic
    /// operation.
    ///
    /// SED affects no registers in the microprocessor and no flags other than the
    /// decimal mode which is set to a 1.
    ///
    /// **Note on the MOS 6502:**
    ///
    /// The value of the decimal mode flag is indeterminate after a RESET.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SED                    | $F8    | 1         | 2
    pub(crate) const fn sed() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sed_set_decimal",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                // Cycle 2: D = 1
                cpu.p.set_d(true);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - 1 - -
    ///
    /// SEI - Set Interrupt Disable
    /// Operation: 1 → I
    ///
    /// This instruction initializes the interrupt disable to a 1. It is used to
    /// mask interrupt requests during system reset operations and during interrupt
    /// commands.
    ///
    /// It affects no registers in the microprocessor and no flags other than the
    /// interrupt disable which is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SEI                    | $78    | 1         | 2
    pub(crate) const fn sei() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sei_set_interrupt",
            micro_fn: |cpu, bus, ctx| {
                cpu.dummy_read(bus, ctx);
                cpu.p.insert(Status::INTERRUPT);
            },
        }]
    }
}
