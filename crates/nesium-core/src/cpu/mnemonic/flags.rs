use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    // ================================================================
    // CLC - Clear Carry Flag
    // ================================================================
    /// Purpose: Clears the Carry flag (C = 0)
    /// Opcode: 0x18
    /// Flags: C ← 0
    /// Cycles: 2
    pub(crate) const fn clc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "clc_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "clc_clear_carry",
            micro_fn: |cpu, _bus| {
                // Cycle 2: C = 0
                cpu.p.set(Status::CARRY, false);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // SEC - Set Carry Flag
    // ================================================================
    /// Purpose: Sets the Carry flag (C = 1)
    /// Opcode: 0x38
    /// Flags: C ← 1
    /// Cycles: 2
    pub(crate) const fn sec() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sec_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "sec_set_carry",
            micro_fn: |cpu, _bus| {
                // Cycle 2: C = 1
                cpu.p.set(Status::CARRY, true);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // CLD - Clear Decimal Mode
    // ================================================================
    /// Purpose: Clears Decimal mode (D = 0)
    /// Opcode: 0xD8
    /// Flags: D ← 0
    /// Cycles: 2
    pub(crate) const fn cld() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "cld_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "cld_clear_decimal",
            micro_fn: |cpu, _bus| {
                // Cycle 2: D = 0
                cpu.p.set(Status::DECIMAL, false);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // SED - Set Decimal Mode
    // ================================================================
    /// Purpose: Sets Decimal mode (D = 1)
    /// Opcode: 0xF8
    /// Flags: D ← 1
    /// Cycles: 2
    pub(crate) const fn sed() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sed_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "sed_set_decimal",
            micro_fn: |cpu, _bus| {
                // Cycle 2: D = 1
                cpu.p.set(Status::DECIMAL, true);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // CLI - Clear Interrupt Disable
    // ================================================================
    /// Purpose: Allows maskable interrupts (I = 0)
    /// Opcode: 0x58
    /// Flags: I ← 0
    /// Cycles: 2
    /// Note: IRQ can be serviced after next instruction
    pub(crate) const fn cli() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "cli_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "cli_clear_interrupt",
            micro_fn: |cpu, _bus| {
                // Cycle 2: I = 0
                cpu.p.set(Status::INTERRUPT, false);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // SEI - Set Interrupt Disable
    // ================================================================
    /// Purpose: Disables maskable interrupts (I = 1)
    /// Opcode: 0x78
    /// Flags: I ← 1
    /// Cycles: 2
    pub(crate) const fn sei() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "sei_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "sei_set_interrupt",
            micro_fn: |cpu, _bus| {
                // Cycle 2: I = 1
                cpu.p.set(Status::INTERRUPT, true);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // CLV - Clear Overflow Flag
    // ================================================================
    /// Purpose: Clears the Overflow flag (V = 0)
    /// Opcode: 0xB8
    /// Flags: V ← 0
    /// Cycles: 2
    pub(crate) const fn clv() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "clv_dummy_read",
            micro_fn: |cpu, bus| {
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "clv_clear_overflow",
            micro_fn: |cpu, _bus| {
                // Cycle 2: V = 0
                cpu.p.set(Status::OVERFLOW, false);
            },
        };
        &[OP1, OP2]
    }
}
