use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    // ================================================================
    // ASL A - Arithmetic Shift Left Accumulator
    // ================================================================
    /// Purpose:
    /// Shifts the accumulator left by one bit. Bit 0 becomes 0, bit 7 goes to Carry.
    ///
    /// Operation:
    /// C ← A7 ← A6 ← A5 ← A4 ← A3 ← A2 ← A1 ← A0 ← 0
    ///
    /// Flags Affected:
    /// N — Set if result bit 7 is set
    /// Z — Set if result is zero
    /// C — Receives old bit 7 of A
    ///
    /// Cycle-by-cycle (2 cycles):
    /// 1. Dummy read from PC
    /// 2. Perform shift, update A and flags
    pub(crate) const fn asl_a() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "asl_a_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from current PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "asl_a_shift",
            micro_fn: |cpu, bus| {
                // Cycle 2: Shift left
                // C = bit 7 of A
                cpu.p.set_c(cpu.a & 0x80 != 0);
                // A = A << 1, bit 0 = 0
                cpu.a <<= 1;
                // Update N and Z
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // LSR A - Logical Shift Right Accumulator
    // ================================================================
    /// Purpose:
    /// Shifts the accumulator right by one bit. Bit 7 becomes 0, bit 0 goes to Carry.
    ///
    /// Operation:
    /// 0 → A7 → A6 → A5 → A4 → A3 → A2 → A1 → A0 → C
    ///
    /// Flags Affected:
    /// N — Always cleared (result bit 7 is always 0)
    /// Z — Set if result is zero
    /// C — Receives old bit 0 of A
    ///
    /// Cycle-by-cycle (2 cycles):
    /// 1. Dummy read from PC
    /// 2. Perform shift, update A and flags
    pub(crate) const fn lsr_a() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "lsr_a_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "lsr_a_shift",
            micro_fn: |cpu, bus| {
                // Cycle 2: Shift right
                // C = bit 0 of A
                cpu.p.set_c(cpu.a & 0x01 != 0);
                // A = A >> 1, bit 7 = 0
                cpu.a >>= 1;
                // N is always 0 after LSR, Z based on result
                cpu.p.remove(Status::NEGATIVE);
                cpu.p.set_z(cpu.a == 0);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // ROL A - Rotate Left Accumulator
    // ================================================================
    /// Purpose:
    /// Rotates the accumulator left. Old Carry goes to bit 0, bit 7 goes to Carry.
    ///
    /// Operation:
    /// C ← A7 ← A6 ← A5 ← A4 ← A3 ← A2 ← A1 ← A0 ← C
    ///
    /// Flags Affected:
    /// N — Set if result bit 7 is set
    /// Z — Set if result is zero
    /// C — Receives old bit 7 of A
    ///
    /// Cycle-by-cycle (2 cycles):
    /// 1. Dummy read from PC
    /// 2. Perform rotate using current Carry
    pub(crate) const fn rol_a() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rol_a_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "rol_a_rotate",
            micro_fn: |cpu, bus| {
                // Cycle 2: Rotate left through Carry
                let old_bit7 = cpu.a & 0x80;
                let new_a = (cpu.a << 1) | if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
                cpu.a = new_a;
                // C = old bit 7
                cpu.p.set_c(old_bit7 != 0);
                // Update N and Z
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // ROR A - Rotate Right Accumulator
    // ================================================================
    /// Purpose:
    /// Rotates the accumulator right. Old Carry goes to bit 7, bit 0 goes to Carry.
    ///
    /// Operation:
    /// C → A7 → A6 → A5 → A4 → A3 → A2 → A1 → A0 → C
    ///
    /// Flags Affected:
    /// N — Set if result bit 7 is set (from old Carry)
    /// Z — Set if result is zero
    /// C — Receives old bit 0 of A
    ///
    /// Cycle-by-cycle (2 cycles):
    /// 1. Dummy read from PC
    /// 2. Perform rotate using current Carry
    pub(crate) const fn ror_a() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "ror_a_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "ror_a_rotate",
            micro_fn: |cpu, bus| {
                // Cycle 2: Rotate right through Carry
                let old_bit0 = cpu.a & 0x01;
                let new_a = (cpu.a >> 1)
                    | if cpu.p.contains(Status::CARRY) {
                        0x80
                    } else {
                        0
                    };
                cpu.a = new_a;
                // C = old bit 0
                cpu.p.set_c(old_bit0 != 0);
                // Update N and Z (N = bit7 = old C)
                cpu.p.set_zn(cpu.a);
            },
        };
        &[OP1, OP2]
    }
}
