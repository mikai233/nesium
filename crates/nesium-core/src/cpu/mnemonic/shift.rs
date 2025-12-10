use crate::cpu::{
    micro_op::MicroOp,
    mnemonic::Mnemonic,
    status::{BIT_0, BIT_7, Status},
};

impl Mnemonic {
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
    pub(crate) const fn asl() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "asl_read",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.effective_addr);
                },
            },
            MicroOp {
                name: "asl_dummy_write",
                micro_fn: |cpu, bus| {
                    bus.mem_write(cpu.effective_addr, cpu.base);
                },
            },
            MicroOp {
                name: "asl_shift",
                micro_fn: |cpu, bus| {
                    if cpu.opcode_in_flight == Some(0x0A) {
                        // Accumulator
                        // Dummy read
                        let _ = bus.mem_read(cpu.pc);
                        // C = bit 7 of A
                        cpu.p.set_c(cpu.a & BIT_7 != 0);
                        // A = A << 1, bit 0 = 0
                        cpu.a <<= 1;
                        // Update N and Z
                        cpu.p.set_zn(cpu.a);
                    } else {
                        // Memory RMW: compute result and flags, then perform final write
                        let old_bit7 = cpu.base & BIT_7;
                        cpu.base <<= 1;
                        // C = old bit 7
                        cpu.p.set_c(old_bit7 != 0);
                        // Update N and Z based on the new value
                        cpu.p.set_zn(cpu.base);
                        // Final write with the new value
                        bus.mem_write(cpu.effective_addr, cpu.base);
                    }
                },
            },
        ]
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
    pub(crate) const fn lsr() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "lsr_read",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.effective_addr);
                },
            },
            MicroOp {
                name: "lsr_dummy_write",
                micro_fn: |cpu, bus| {
                    bus.mem_write(cpu.effective_addr, cpu.base);
                },
            },
            MicroOp {
                name: "lsr_shift",
                micro_fn: |cpu, bus| {
                    if cpu.opcode_in_flight == Some(0x4A) {
                        // Accumulator
                        // Dummy read
                        let _ = bus.mem_read(cpu.pc);
                        // C = bit 7 of A
                        cpu.p.set_c(cpu.a & BIT_0 != 0);
                        // A = A << 1, bit 7 = 0
                        cpu.a >>= 1;
                        // Update N and Z
                        cpu.p.remove(Status::NEGATIVE);
                        cpu.p.set_z(cpu.a == 0);
                    } else {
                        // Memory RMW: compute result and flags, then perform final write
                        let old_bit0 = cpu.base & BIT_0;
                        cpu.base >>= 1;
                        // C = old bit 0
                        cpu.p.set_c(old_bit0 != 0);
                        // LSR always shifts in 0 on bit 7, so N is always cleared
                        cpu.p.remove(Status::NEGATIVE);
                        cpu.p.set_z(cpu.base == 0);
                        // Final write with the new value
                        bus.mem_write(cpu.effective_addr, cpu.base);
                    }
                },
            },
        ]
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
    pub(crate) const fn rol() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "rol_read",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.effective_addr);
                },
            },
            MicroOp {
                name: "rol_dummy_write",
                micro_fn: |cpu, bus| {
                    bus.mem_write(cpu.effective_addr, cpu.base);
                },
            },
            MicroOp {
                name: "rol_rotate",
                micro_fn: |cpu, bus| {
                    // Cycle 2: Rotate left through Carry
                    if cpu.opcode_in_flight == Some(0x2A) {
                        // Dummy read
                        let _ = bus.mem_read(cpu.pc);
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
                        // C = old bit 7
                        cpu.p.set_c(old_bit7 != 0);
                        // Update N and Z
                        cpu.p.set_zn(cpu.base);
                        // Final write with the new value
                        bus.mem_write(cpu.effective_addr, cpu.base);
                    }
                },
            },
        ]
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
    pub(crate) const fn ror() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "ror_read",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.effective_addr);
                },
            },
            MicroOp {
                name: "ror_dummy_write",
                micro_fn: |cpu, bus| {
                    bus.mem_write(cpu.effective_addr, cpu.base);
                },
            },
            MicroOp {
                name: "ror_rotate",
                micro_fn: |cpu, bus| {
                    if cpu.opcode_in_flight == Some(0x6A) {
                        // Dummy read
                        let _ = bus.mem_read(cpu.pc);
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
                        // C = old bit 0
                        cpu.p.set_c(old_bit0 != 0);
                        // Update N and Z (N = bit7 = old C)
                        cpu.p.set_zn(cpu.base);
                        // Final write with the new value
                        bus.mem_write(cpu.effective_addr, cpu.base);
                    }
                },
            },
        ]
    }
}

#[cfg(test)]
mod shift_tests {
    use crate::cpu::{
        mnemonic::{Mnemonic, tests::InstrTest},
        status::{BIT_0, BIT_7},
    };

    #[test]
    fn test_asl() {
        InstrTest::new(Mnemonic::ASL).test(|verify, cpu, bus| {
            if cpu.opcode_in_flight == Some(0x0A) {
                let c = verify.cpu.a & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.cpu.a << 1;
                verify.check_nz(cpu.p, v);
            } else {
                let c = verify.m & BIT_7 != 0;
                assert_eq!(cpu.p.c(), c);
                let v = verify.m << 1;
                let m = bus.mem_read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_lsr() {
        InstrTest::new(Mnemonic::LSR).test(|verify, cpu, bus| {
            if cpu.opcode_in_flight == Some(0x4A) {
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
                let m = bus.mem_read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_rol() {
        InstrTest::new(Mnemonic::ROL).test(|verify, cpu, bus| {
            if cpu.opcode_in_flight == Some(0x2A) {
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
                let m = bus.mem_read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }

    #[test]
    fn test_ror() {
        InstrTest::new(Mnemonic::ROR).test(|verify, cpu, bus| {
            if cpu.opcode_in_flight == Some(0x6A) {
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
                let m = bus.mem_read(verify.addr);
                assert_eq!(v, m);
                verify.check_nz(cpu.p, v);
            }
        });
    }
}
