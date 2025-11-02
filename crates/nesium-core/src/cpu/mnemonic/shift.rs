use crate::{
    bus::Bus,
    cpu::{
        micro_op::MicroOp,
        mnemonic::Mnemonic,
        status::{BIT_0, BIT_7, Status},
    },
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
    pub(crate) const fn asl() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "asl_read_operand",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.effective_addr);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "asl_dummy_write",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.base);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "asl_shift",
            micro_fn: |cpu, bus| {
                if cpu.opcode == Some(0x0A) {
                    // Accumulator
                    // Dummy read
                    let _ = bus.read(cpu.pc);
                    // C = bit 7 of A
                    cpu.p.set_c(cpu.a & BIT_7 != 0);
                    // A = A << 1, bit 0 = 0
                    cpu.a <<= 1;
                    // Update N and Z
                    cpu.p.set_zn(cpu.a);
                } else {
                    // Other
                    cpu.p.set_c(cpu.base & BIT_7 != 0);
                    cpu.base <<= 1;
                    bus.write(cpu.effective_addr, cpu.base);
                    // Update N and Z
                    cpu.p.set_zn(cpu.base);
                }
            },
        };
        &[OP1, OP2, OP3]
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
    pub(crate) const fn lsr() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "lsr_read_operand",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.effective_addr);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "lsr_dummy_write",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.base);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "lsr_shift",
            micro_fn: |cpu, bus| {
                if cpu.opcode == Some(0x4A) {
                    // Accumulator
                    // Dummy read
                    let _ = bus.read(cpu.pc);
                    // C = bit 7 of A
                    cpu.p.set_c(cpu.a & BIT_0 != 0);
                    // A = A << 1, bit 7 = 0
                    cpu.a >>= 1;
                    // Update N and Z
                    cpu.p.remove(Status::NEGATIVE);
                    cpu.p.set_z(cpu.a == 0);
                } else {
                    // Other
                    cpu.p.set_c(cpu.base & BIT_0 != 0);
                    cpu.base >>= 1;
                    bus.write(cpu.effective_addr, cpu.base);
                    // Update N and Z
                    cpu.p.remove(Status::NEGATIVE);
                    cpu.p.set_z(cpu.base == 0);
                }
            },
        };
        &[OP1, OP2, OP3]
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
    pub(crate) const fn rol() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rol_read_operand",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.effective_addr);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "rol_dummy_write",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.base);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "rol_rotate",
            micro_fn: |cpu, bus| {
                // Cycle 2: Rotate left through Carry
                if cpu.opcode == Some(0x2A) {
                    // Dummy read
                    let _ = bus.read(cpu.pc);
                    let old_bit7 = cpu.a & BIT_7;
                    let new_a = (cpu.a << 1) | if cpu.p.c() { 1 } else { 0 };
                    cpu.a = new_a;
                    // C = old bit 7
                    cpu.p.set_c(old_bit7 != 0);
                    // Update N and Z
                    cpu.p.set_zn(cpu.a);
                } else {
                    let old_bit7 = cpu.base & BIT_7;
                    let new_a = (cpu.base << 1) | if cpu.p.c() { 1 } else { 0 };
                    cpu.base = new_a;
                    bus.write(cpu.effective_addr, cpu.base);
                    // C = old bit 7
                    cpu.p.set_c(old_bit7 != 0);
                    // Update N and Z
                    cpu.p.set_zn(cpu.base);
                }
            },
        };
        &[OP1, OP2, OP3]
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
    pub(crate) const fn ror() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "ror_read_operand",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.effective_addr);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "ror_dummy_write",
            micro_fn: |cpu, bus| {
                bus.write(cpu.effective_addr, cpu.base);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "ror_rotate",
            micro_fn: |cpu, bus| {
                if cpu.opcode == Some(0x6A) {
                    // Dummy read
                    let _ = bus.read(cpu.pc);
                    let old_bit0 = cpu.a & BIT_0;
                    let new_a = (cpu.a >> 1) | if cpu.p.c() { BIT_7 } else { 0 };
                    cpu.a = new_a;
                    // C = old bit 0
                    cpu.p.set_c(old_bit0 != 0);
                    // Update N and Z (N = bit7 = old C)
                    cpu.p.set_zn(cpu.a);
                } else {
                    let old_bit0 = cpu.base & BIT_0;
                    let new_a = (cpu.base >> 1) | if cpu.p.c() { BIT_7 } else { 0 };
                    cpu.base = new_a;
                    bus.write(cpu.effective_addr, cpu.base);
                    // C = old bit 0
                    cpu.p.set_c(old_bit0 != 0);
                    // Update N and Z (N = bit7 = old C)
                    cpu.p.set_zn(cpu.base);
                }
            },
        };
        &[OP1, OP2, OP3]
    }
}

#[cfg(test)]
mod shift_test {
    use crate::{
        bus::Bus,
        cpu::{
            mnemonic::{Mnemonic, tests::InstrTest},
            status::{BIT_0, BIT_7},
        },
    };

    #[test]
    fn test_asl() {
        InstrTest::new(Mnemonic::ASL).test(|verify, cpu, bus| {
            if cpu.opcode == Some(0x0A) {
                let c = verify.cpu.a & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.cpu.a << 1;
                verify.check_nz(cpu.p, v);
            } else {
                let c = verify.m & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.m << 1;
                let m = bus.read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_lsr() {
        InstrTest::new(Mnemonic::LSR).test(|verify, cpu, bus| {
            if cpu.opcode == Some(0x4A) {
                // Accumulator mode
                let c = verify.cpu.a & BIT_0 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.cpu.a >> 1;
                verify.check_nz(cpu.p, v);
            } else {
                // Memory mode
                let c = verify.m & BIT_0 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.m >> 1;
                let m = bus.read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_rol() {
        InstrTest::new(Mnemonic::ROL).test(|verify, cpu, bus| {
            if cpu.opcode == Some(0x2A) {
                // Accumulator mode
                let c_in = verify.cpu.p.c() as u8;
                let c_out = verify.cpu.a & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c_out);
                let v = (verify.cpu.a << 1) | c_in;
                verify.check_nz(cpu.p, v);
            } else {
                // Memory mode
                let c_in = verify.cpu.p.c() as u8;
                let c_out = verify.m & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c_out);
                let v = (verify.m << 1) | c_in;
                let m = bus.read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_ror() {
        InstrTest::new(Mnemonic::ROR).test(|verify, cpu, bus| {
            if cpu.opcode == Some(0x6A) {
                // Accumulator mode
                let c_in = (verify.cpu.p.c() as u8) << 7;
                let c_out = verify.cpu.a & BIT_0 != 0;
                assert_eq!(cpu.p.c(), c_out);
                let v = (verify.cpu.a >> 1) | c_in;
                verify.check_nz(cpu.p, v);
            } else {
                // Memory mode
                let c_in = (verify.cpu.p.c() as u8) << 7;
                let c_out = verify.m & BIT_0 != 0;
                assert_eq!(cpu.p.c(), c_out);
                let v = (verify.m >> 1) | c_in;
                let m = bus.read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }
}
