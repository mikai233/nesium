use crate::{
    bus::{Bus, BusImpl},
    cpu::{Cpu, addressing::Addressing, status::Status},
};

pub mod arith;
pub mod bra;
pub mod ctrl;
pub mod flags;
pub mod inc;
pub mod kill;
pub mod load;
pub mod logic;
pub mod nop;
pub mod shift;
pub mod stack;
pub mod trans;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReadFrom {
    Immediate,
    ZeroPage,
    Effective,
}

type MicroFn = fn(&mut Cpu, bus: &mut BusImpl);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MicroOp {
    pub(crate) name: &'static str,
    pub(crate) micro_fn: MicroFn,
}

impl MicroOp {
    /// Execute this micro operation
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut BusImpl) {
        (self.micro_fn)(cpu, bus)
    }

    // ───────────────────────────────────────────────
    //  Fetch & Program Counter Operations
    // ───────────────────────────────────────────────

    /// Cycle 1: Advance PC after fetching opcode.
    pub(crate) const fn advance_pc_after_opcode() -> Self {
        MicroOp {
            name: "advance_pc_after_opcode",
            micro_fn: |cpu, _| cpu.incr_pc(),
        }
    }

    /// Cycle 2: Read zero-page address from PC, increment PC.
    pub(crate) const fn fetch_zp_addr_lo() -> Self {
        MicroOp {
            name: "fetch_zp_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.zp_addr = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 2: Fetch low byte of absolute address from PC.
    pub(crate) const fn fetch_abs_addr_lo() -> Self {
        MicroOp {
            name: "fetch_abs_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.base_lo = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address and form full address.
    pub(crate) const fn fetch_abs_addr_hi() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                cpu.effective_addr = ((hi as u16) << 8) | cpu.base_lo as u16;
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte, add X index, detect page crossing.
    pub(crate) const fn fetch_abs_addr_hi_add_x() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_x",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.x as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.incr_pc();
                cpu.check_cross_page = true;
            },
        }
    }

    /// Cycle 3: Fetch high byte, add Y index, detect page crossing.
    pub(crate) const fn fetch_abs_addr_hi_add_y() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_y",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.y as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.incr_pc();
                cpu.check_cross_page = true;
            },
        }
    }

    // ───────────────────────────────────────────────
    //  Zero Page & Indirect Operations
    // ───────────────────────────────────────────────

    /// Cycle 3 (Indirect,X): Calculate ($nn + X) with zero-page wrap and dummy read.
    pub(crate) const fn read_indirect_x_dummy() -> Self {
        MicroOp {
            name: "read_indirect_x_dummy",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                let _ = bus.read(ptr); // dummy read for timing
            },
        }
    }

    /// Cycle 4 (Indirect,X): Read low byte from ($nn + X) zero-page wrap.
    pub(crate) const fn read_indirect_x_lo() -> Self {
        MicroOp {
            name: "read_indirect_x_lo",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                cpu.base_lo = bus.read(ptr);
            },
        }
    }

    /// Cycle 5 (Indirect,X): Read high byte from ($nn + X + 1) zero-page wrap.
    pub(crate) const fn read_indirect_x_hi() -> Self {
        MicroOp {
            name: "read_indirect_x_hi",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16 + 1) & 0x00FF;
                let hi = bus.read(ptr);
                cpu.effective_addr = ((hi as u16) << 8) | cpu.base_lo as u16;
            },
        }
    }

    /// Read byte from zero-page address ($nn)
    pub(crate) const fn read_zero_page() -> Self {
        MicroOp {
            name: "read_zero_page",
            micro_fn: |cpu, bus| {
                cpu.base_lo = bus.read(cpu.zp_addr as u16);
            },
        }
    }

    /// Cycle 4 (Indirect),Y: Read high byte from ($nn + 1), add Y, detect page crossing.
    pub(crate) const fn read_indirect_y_hi() -> Self {
        MicroOp {
            name: "read_indirect_y_hi",
            micro_fn: |cpu, bus| {
                let hi_addr = (cpu.zp_addr as u16 + 1) & 0x00FF;
                let hi = bus.read(hi_addr);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.y as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.check_cross_page = true;
            },
        }
    }

    /// Cycle 3 (ZeroPage,Y): Add Y to zero-page address with wrap-around, dummy read.
    pub(crate) const fn read_zero_page_add_y_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_y_dummy",
            micro_fn: |cpu, bus| {
                let addr = (cpu.zp_addr as u16 + cpu.y as u16) & 0x00FF;
                let _ = bus.read(addr); // dummy read for timing
                cpu.effective_addr = addr;
            },
        }
    }

    /// Cycle 3 (ZeroPage,X): Add X to zero-page address with wrap-around, dummy read.
    pub(crate) const fn read_zero_page_add_x_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_x_dummy",
            micro_fn: |cpu, bus| {
                let addr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                let _ = bus.read(addr); // dummy read for timing
                cpu.effective_addr = addr;
            },
        }
    }

    /// Cross-page dummy read for Absolute,X
    pub(crate) const fn dummy_read_cross_x() -> Self {
        MicroOp {
            name: "dummy_read_cross_x",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr); // dummy read for cross-page
            },
        }
    }

    /// Cross-page dummy read for Absolute,Y or (Indirect),Y
    pub(crate) const fn dummy_read_cross_y() -> Self {
        MicroOp {
            name: "dummy_read_cross_y",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr); // dummy read for cross-page
            },
        }
    }

    pub(crate) const fn lda(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "lda_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    cpu.a = data;
                    cpu.p.set_zn(data);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "lda_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    cpu.a = data;
                    cpu.p.set_zn(data);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "lda_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    cpu.a = data;
                    cpu.p.set_zn(data);
                },
            },
        }
    }

    pub(crate) const fn and(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "and_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    cpu.a &= data;
                    cpu.p.set_zn(cpu.a);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "and_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    cpu.a &= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "and_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    cpu.a &= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
        }
    }

    pub(crate) const fn bit(read: ReadFrom) -> Self {
        match read {
            ReadFrom::ZeroPage => MicroOp {
                name: "bit_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    let result = cpu.a & data;
                    cpu.p.set_z(result == 0);
                    cpu.p.set_v((data & 0x40) != 0);
                    cpu.p.set_n((data & 0x80) != 0);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "bit_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    let result = cpu.a & data;
                    cpu.p.set_z(result == 0);
                    cpu.p.set_v((data & 0x40) != 0);
                    cpu.p.set_n((data & 0x80) != 0);
                },
            },
            _ => panic!("BIT does not support this addressing mode"),
        }
    }

    /// ORA - Bitwise OR with Accumulator
    /// A = A | M
    /// Flags: N, Z
    pub(crate) const fn ora(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "ora_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    cpu.a |= data;
                    cpu.p.set_zn(cpu.a);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "ora_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    cpu.a |= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "ora_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    cpu.a |= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
        }
    }

    /// EOR - Exclusive OR with Accumulator
    /// A = A ^ M
    /// Flags: N, Z
    pub(crate) const fn eor(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "eor_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    cpu.a ^= data;
                    cpu.p.set_zn(cpu.a);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "eor_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    cpu.a ^= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "eor_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    cpu.a ^= data;
                    cpu.p.set_zn(cpu.a);
                },
            },
        }
    }

    /// ADC - Add with Carry
    /// A = A + M + C
    /// Flags: N, V, Z, C
    /// Note: Decimal mode (D flag) is ignored on NES (2A03), so we use binary addition.
    pub(crate) const fn adc(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "adc_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let a = cpu.a;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let result = a.wrapping_add(data).wrapping_add(c);

                    // Carry flag: set if unsigned overflow
                    let carry = (a as u16 + data as u16 + c as u16) > 0xFF;
                    cpu.p.set_c(carry);

                    // Overflow flag: set if signed overflow (bit 7 sign change incorrect)
                    let overflow = ((a ^ result) & (data ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "adc_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    let a = cpu.a;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let result = a.wrapping_add(data).wrapping_add(c);

                    let carry = (a as u16 + data as u16 + c as u16) > 0xFF;
                    cpu.p.set_c(carry);

                    let overflow = ((a ^ result) & (data ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "adc_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    let a = cpu.a;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let result = a.wrapping_add(data).wrapping_add(c);

                    let carry = (a as u16 + data as u16 + c as u16) > 0xFF;
                    cpu.p.set_c(carry);

                    let overflow = ((a ^ result) & (data ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        }
    }

    /// ANC - AND with Accumulator then set Carry as N (undocumented)
    /// A = A & M
    /// C = bit 7 of result
    /// N = bit 7 of result
    /// Z = result == 0
    /// V, D, I unchanged
    /// Note: This is an illegal/undocumented opcode on 6502.
    pub(crate) const fn anc(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "anc_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    cpu.a &= data;
                    let result = cpu.a;

                    // C = N = bit 7 of result
                    let bit7 = (result & 0x80) != 0;
                    cpu.p.set_c(bit7);
                    cpu.p.set(Status::NEGATIVE, bit7);

                    // Z flag
                    cpu.p.set_z(result == 0);

                    cpu.incr_pc();
                },
            },
            // Note: ANC only exists in Immediate mode on real 6502.
            // Other modes are not used, but we define them as no-op for safety.
            _ => MicroOp {
                name: "anc_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// ARR - AND then Rotate Right (undocumented / unstable)
    /// A = (A & M) >> 1, with bit 7 = old Carry, bit 0 into Carry
    /// Then:
    ///   C = bit 6 of result (after rotate)
    ///   V = (bit 6 XOR bit 5) of result
    ///   N = bit 7 of result
    ///   Z = result == 0
    ///
    /// Warning: This is an *unstable* illegal opcode.
    /// Behavior varies slightly between 6502 revisions.
    /// The version below matches common NES 2A03 behavior.
    pub(crate) const fn arr(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "arr_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let a = cpu.a;
                    let c = cpu.p.contains(Status::CARRY) as u8;

                    // Step 1: A = A & M
                    let mut temp = a & data;

                    // Step 2: Rotate right: bit 0 -> C, old C -> bit 7
                    let old_bit0 = temp & 1;
                    temp = (temp >> 1) | (c << 7);

                    // Store result
                    cpu.a = temp;

                    // Step 3: Update flags based on *final* result
                    let bit6 = (temp >> 6) & 1;
                    let bit5 = (temp >> 5) & 1;

                    cpu.p.set_c(bit6 == 1); // C = bit 6
                    cpu.p.set_v(bit6 != bit5); // V = bit6 XOR bit5
                    cpu.p.set_zn(temp); // N and Z from result

                    cpu.incr_pc();
                },
            },
            // ARR only exists in Immediate mode on real hardware.
            // Other modes are not defined.
            _ => MicroOp {
                name: "arr_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// ASR - AND then Logical Shift Right (undocumented)
    /// A = (A & M) >> 1
    /// Bit 0 of (A & M) goes into Carry
    /// Bit 7 of result is always 0 (logical shift)
    /// Flags:
    ///   C = bit 0 of (A & M) before shift
    ///   Z = result == 0
    ///   N = 0 (always, since bit 7 is 0)
    pub(crate) const fn asr(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "asr_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let temp = cpu.a & data;

                    // Carry = LSB of (A & M)
                    let carry_out = (temp & 1) != 0;
                    cpu.p.set_c(carry_out);

                    // Logical shift right: A = (A & M) >> 1, bit 7 = 0
                    cpu.a = temp >> 1;

                    // Z = result == 0
                    // N = 0 (always cleared)
                    cpu.p.set_zn(cpu.a);
                    cpu.p.remove(Status::NEGATIVE); // Explicitly clear N

                    cpu.incr_pc();
                },
            },
            // ASR only exists in Immediate mode on real hardware.
            // Other modes are not defined.
            _ => MicroOp {
                name: "asr_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// CMP - Compare Accumulator with Memory
    /// result = A - M
    /// Flags:
    ///   C = 1 if A >= M (no borrow)
    ///   Z = 1 if A == M
    ///   N = bit 7 of result
    /// A is unchanged
    pub(crate) const fn cmp(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "cmp_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let result = cpu.a.wrapping_sub(data);

                    cpu.p.set_c(cpu.a >= data); // Carry set if no borrow
                    cpu.p.set_zn(result); // Z and N from subtraction result

                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "cmp_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    let result = cpu.a.wrapping_sub(data);

                    cpu.p.set_c(cpu.a >= data);
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "cmp_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    let result = cpu.a.wrapping_sub(data);

                    cpu.p.set_c(cpu.a >= data);
                    cpu.p.set_zn(result);
                },
            },
        }
    }

    /// CPX - Compare Index Register X with Memory
    /// result = X - M
    /// Flags:
    ///   C = 1 if X >= M (no borrow)
    ///   Z = 1 if X == M
    ///   N = bit 7 of result
    /// X is unchanged
    pub(crate) const fn cpx(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "cpx_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let result = cpu.x.wrapping_sub(data);

                    cpu.p.set_c(cpu.x >= data); // Carry set if no borrow
                    cpu.p.set_zn(result); // Z and N from subtraction result

                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "cpx_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    let result = cpu.x.wrapping_sub(data);

                    cpu.p.set_c(cpu.x >= data);
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "cpx_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    let result = cpu.x.wrapping_sub(data);

                    cpu.p.set_c(cpu.x >= data);
                    cpu.p.set_zn(result);
                },
            },
        }
    }

    /// CPY - Compare Index Register Y with Memory
    /// result = Y - M
    /// Flags:
    ///   C = 1 if Y >= M (no borrow)
    ///   Z = 1 if Y == M
    ///   N = bit 7 of result
    /// Y is unchanged
    pub(crate) const fn cpy(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "cpy_from_immediate",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.pc);
                    let result = cpu.y.wrapping_sub(data);

                    cpu.p.set_c(cpu.y >= data); // Carry set if no borrow
                    cpu.p.set_zn(result); // Z and N from subtraction result

                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "cpy_from_zero_page",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.zp_addr as u16);
                    let result = cpu.y.wrapping_sub(data);

                    cpu.p.set_c(cpu.y >= data);
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "cpy_from_effective",
                micro_fn: |cpu, bus| {
                    let data = bus.read(cpu.effective_addr);
                    let result = cpu.y.wrapping_sub(data);

                    cpu.p.set_c(cpu.y >= data);
                    cpu.p.set_zn(result);
                },
            },
        }
    }

    /// DCP - Decrement Memory then Compare with Accumulator (undocumented)
    /// Equivalent to: DEC M; CMP M
    /// But as a single RMW instruction:
    ///   1. Read M
    ///   2. Write old M back (dummy write)
    ///   3. M = M - 1
    ///   4. Write new M
    ///   5. Compare A with (M - 1)
    /// Flags: C, Z, N (same as CMP)
    pub(crate) const fn dcp_read_modify_write(read: ReadFrom) -> Self {
        match read {
            ReadFrom::ZeroPage => MicroOp {
                name: "dcp_zp_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.zp_addr as u16;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write
                    let new = old.wrapping_sub(1);
                    bus.write(addr, new); // write decremented value

                    // CMP: A - new
                    let result = cpu.a.wrapping_sub(new);
                    cpu.p.set_c(cpu.a >= new);
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "dcp_eff_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.effective_addr;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write
                    let new = old.wrapping_sub(1);
                    bus.write(addr, new); // write decremented value

                    // CMP: A - new
                    let result = cpu.a.wrapping_sub(new);
                    cpu.p.set_c(cpu.a >= new);
                    cpu.p.set_zn(result);
                },
            },
            _ => MicroOp {
                name: "dcp_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// ISC - Increment Memory then Subtract from Accumulator with Carry (undocumented)
    /// Equivalent to: INC M; SBC M
    /// But as a single RMW instruction:
    ///   1. Read M
    ///   2. Write old M back (dummy write)
    ///   3. M = M + 1
    ///   4. Write new M
    ///   5. A = A - new_M - (1 - C)
    /// Flags: N, V, Z, C (same as SBC)
    /// Note: Uses binary addition (NES ignores Decimal mode)
    pub(crate) const fn isc_read_modify_write(read: ReadFrom) -> Self {
        match read {
            ReadFrom::ZeroPage => MicroOp {
                name: "isc_zp_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.zp_addr as u16;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write
                    let new = old.wrapping_add(1);
                    bus.write(addr, new); // write incremented value

                    // SBC: A - new - (1 - C)
                    let m = new;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_sub(m).wrapping_sub(1 - c);

                    // Carry: set if no borrow
                    let borrow = (a as u16) < (m as u16 + (1 - c) as u16);
                    cpu.p.set_c(!borrow);

                    // Overflow: signed overflow detection
                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "isc_eff_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.effective_addr;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write
                    let new = old.wrapping_add(1);
                    bus.write(addr, new); // write incremented value

                    // SBC logic
                    let m = new;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_sub(m).wrapping_sub(1 - c);

                    let borrow = (a as u16) < (m as u16 + (1 - c) as u16);
                    cpu.p.set_c(!borrow);

                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            _ => MicroOp {
                name: "isc_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// RLA - Rotate Left then AND with Accumulator (undocumented)
    /// Equivalent to: ROL M; AND M
    /// But as a single RMW instruction:
    ///   1. Read M
    ///   2. Write old M back (dummy write)
    ///   3. M = (M << 1) | C   (old C in, bit7 -> C)
    ///   4. Write new M
    ///   5. A = A & new_M
    /// Flags: N, Z (from A & M), C = old bit7 of M
    pub(crate) const fn rla_read_modify_write(read: ReadFrom) -> Self {
        match read {
            ReadFrom::ZeroPage => MicroOp {
                name: "rla_zp_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.zp_addr as u16;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write

                    // ROL: bit7 -> C, old C -> bit0
                    let old_bit7 = (old >> 7) & 1;
                    let c_in = cpu.p.contains(Status::CARRY) as u8;
                    let new = (old << 1) | c_in;

                    cpu.p.set_c(old_bit7 == 1); // C = old bit7
                    bus.write(addr, new); // write rotated value

                    // AND: A &= new
                    cpu.a &= new;
                    cpu.p.set_zn(cpu.a); // N, Z from result
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "rla_eff_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.effective_addr;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write

                    let old_bit7 = (old >> 7) & 1;
                    let c_in = cpu.p.contains(Status::CARRY) as u8;
                    let new = (old << 1) | c_in;

                    cpu.p.set_c(old_bit7 == 1);
                    bus.write(addr, new);

                    cpu.a &= new;
                    cpu.p.set_zn(cpu.a);
                },
            },
            _ => MicroOp {
                name: "rla_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// RRA - Rotate Right then Add with Carry (undocumented)
    /// Equivalent to: ROR M; ADC M
    /// But as a single RMW instruction:
    ///   1. Read M
    ///   2. Write old M back (dummy write)
    ///   3. M = (M >> 1) | (C << 7)   (old C in bit7, bit0 -> C)
    ///   4. Write new M
    ///   5. A = A + new_M + C
    /// Flags: N, V, Z, C (same as ADC)
    /// Note: Uses binary addition (NES ignores Decimal mode)
    pub(crate) const fn rra_read_modify_write(read: ReadFrom) -> Self {
        match read {
            ReadFrom::ZeroPage => MicroOp {
                name: "rra_zp_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.zp_addr as u16;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write

                    // ROR: bit0 -> C, old C -> bit7
                    let old_bit0 = old & 1;
                    let c_in = cpu.p.contains(Status::CARRY) as u8;
                    let new = (old >> 1) | (c_in << 7);

                    cpu.p.set_c(old_bit0 == 1); // C = old bit0
                    bus.write(addr, new); // write rotated value

                    // ADC: A + new + C
                    let m = new;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_add(m).wrapping_add(c);

                    let carry = (a as u16 + m as u16 + c as u16) > 0xFF;
                    cpu.p.set_c(carry);

                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "rra_eff_rmw",
                micro_fn: |cpu, bus| {
                    let addr = cpu.effective_addr;
                    let old = bus.read(addr);
                    bus.write(addr, old); // dummy write

                    let old_bit0 = old & 1;
                    let c_in = cpu.p.contains(Status::CARRY) as u8;
                    let new = (old >> 1) | (c_in << 7);

                    cpu.p.set_c(old_bit0 == 1);
                    bus.write(addr, new);

                    let m = new;
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_add(m).wrapping_add(c);

                    let carry = (a as u16 + m as u16 + c as u16) > 0xFF;
                    cpu.p.set_c(carry);

                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            _ => MicroOp {
                name: "rra_invalid",
                micro_fn: |_, _| {},
            },
        }
    }

    /// SBC - Subtract with Borrow (Carry)
    /// A = A - M - (1 - C)
    /// Flags: N, V, Z, C
    /// Note: Decimal mode (D flag) is ignored on NES (2A03) → always binary
    pub(crate) const fn sbc(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "sbc_from_immediate",
                micro_fn: |cpu, bus| {
                    let m = bus.read(cpu.pc);
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_sub(m).wrapping_sub(1 - c);

                    // Carry: set if no borrow (A >= M + (1 - C))
                    let borrow = (a as u16) < (m as u16 + (1 - c) as u16);
                    cpu.p.set_c(!borrow);

                    // Overflow: signed overflow
                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                    cpu.incr_pc();
                },
            },
            ReadFrom::ZeroPage => MicroOp {
                name: "sbc_from_zero_page",
                micro_fn: |cpu, bus| {
                    let m = bus.read(cpu.zp_addr as u16);
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_sub(m).wrapping_sub(1 - c);

                    let borrow = (a as u16) < (m as u16 + (1 - c) as u16);
                    cpu.p.set_c(!borrow);

                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
            ReadFrom::Effective => MicroOp {
                name: "sbc_from_effective",
                micro_fn: |cpu, bus| {
                    let m = bus.read(cpu.effective_addr);
                    let c = cpu.p.contains(Status::CARRY) as u8;
                    let a = cpu.a;
                    let result = a.wrapping_sub(m).wrapping_sub(1 - c);

                    let borrow = (a as u16) < (m as u16 + (1 - c) as u16);
                    cpu.p.set_c(!borrow);

                    let overflow = ((a ^ result) & (m ^ result) & 0x80) != 0;
                    cpu.p.set_v(overflow);

                    cpu.a = result;
                    cpu.p.set_zn(result);
                },
            },
        }
    }

    /// SBX - Subtract from X with Borrow (undocumented)
    /// X = (X & A) - M
    /// Equivalent to: AX = X & A; X = AX - M
    /// Flags:
    ///   C = 1 if (X & A) >= M (no borrow)
    ///   Z = 1 if result == 0
    ///   N = bit 7 of result
    /// Note: Only Immediate mode exists on real hardware.
    pub(crate) const fn sbx(read: ReadFrom) -> Self {
        match read {
            ReadFrom::Immediate => MicroOp {
                name: "sbx_from_immediate",
                micro_fn: |cpu, bus| {
                    let m = bus.read(cpu.pc);
                    let ax = cpu.x & cpu.a;
                    let result = ax.wrapping_sub(m);

                    // Carry: set if no borrow
                    cpu.p.set_c(ax >= m);

                    cpu.x = result;
                    cpu.p.set_zn(result);

                    cpu.incr_pc();
                },
            },
            // SBX only exists in Immediate mode on real 6502.
            // Other modes are not used.
            _ => MicroOp {
                name: "sbx_invalid",
                micro_fn: |_, _| {},
            },
        }
    }
}
