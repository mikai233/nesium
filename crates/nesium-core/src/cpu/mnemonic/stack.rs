use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    // ================================================================
    // PHA - Push Accumulator
    // ================================================================
    /// Purpose:
    /// Pushes the accumulator (A) onto the stack.
    ///
    /// Operation:
    /// M[0x0100 + S] ← A ; S ← S - 1
    ///
    /// Flags Affected:
    /// None
    ///
    /// Cycle-by-cycle (3 cycles):
    /// 1. Dummy read from PC (opcode fetch already done)
    /// 2. Write A to stack at current S, then decrement S
    pub(crate) const fn pha() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "pha_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from current PC (internal operation)
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "pha_write_stack",
            micro_fn: |cpu, bus| {
                // Cycle 2: Write accumulator to stack, then decrement S
                // Hardware writes to [0x0100 + S] using current S, then S--
                bus.write(0x0100 | cpu.s as u16, cpu.a);
                cpu.s = cpu.s.wrapping_sub(1);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // PHP - Push Processor Status
    // ================================================================
    /// Purpose:
    /// Pushes the processor status register (P) onto the stack.
    ///
    /// Operation:
    /// M[0x0100 + S] ← (P | 0x30) ; S ← S - 1
    ///
    /// Flags Affected:
    /// None (but B and bit5 are forced set in pushed value)
    ///
    /// Hardware Notes:
    /// - Bit 4 (B flag) is forced to 1 when pushing
    /// - Bit 5 (unused) is forced to 1 when pushing
    /// - This is hardwired in NMOS 6502
    ///
    /// Cycle-by-cycle (3 cycles):
    /// 1. Dummy read from PC
    /// 2. Write (P | 0x30) to stack, then decrement S
    pub(crate) const fn php() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "php_dummy_read",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from current PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "php_write_stack",
            micro_fn: |cpu, bus| {
                // Cycle 2: Hardware forces B flag (bit4) and unused bit5 when pushing
                let p = cpu.p.to_byte() | 0x30;
                bus.write(0x0100 | cpu.s as u16, p);
                cpu.s = cpu.s.wrapping_sub(1);
            },
        };
        &[OP1, OP2]
    }

    // ================================================================
    // PLA - Pull Accumulator
    // ================================================================
    /// Purpose:
    /// Pulls a byte from the stack into the accumulator (A).
    ///
    /// Operation:
    /// S ← S + 1 ; A ← M[0x0100 + S]
    ///
    /// Flags Affected:
    /// N — Set if bit 7 of A is set
    /// Z — Set if A == 0
    ///
    /// Cycle-by-cycle (4 cycles):
    /// 1. Dummy read from PC
    /// 2. Dummy read from current stack pointer location [0x0100 + S]
    /// 3. Increment S, then read from new stack location into A
    pub(crate) const fn pla() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "pla_dummy_read1",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "pla_dummy_read2",
            micro_fn: |cpu, bus| {
                // Cycle 2: Dummy read from current stack location (before increment)
                let _ = bus.read(0x0100 | cpu.s as u16);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "pla_pull_value",
            micro_fn: |cpu, bus| {
                // Cycle 3: Increment S first, then read from new stack pointer
                cpu.s = cpu.s.wrapping_add(1);
                let value = bus.read(0x0100 | cpu.s as u16);
                cpu.a = value;
                cpu.p.set_zn(value); // Update N and Z flags based on pulled value
            },
        };
        &[OP1, OP2, OP3]
    }

    // ================================================================
    // PLP - Pull Processor Status
    // ================================================================
    /// Purpose:
    /// Pulls a byte from the stack into the processor status register (P).
    ///
    /// Operation:
    /// S ← S + 1 ; P ← (M[0x0100 + S] & 0xEF) | 0x20
    ///
    /// Flags Affected:
    /// All flags are loaded from stack (with modifications)
    ///
    /// Hardware Notes (NMOS 6502):
    /// - Bit 4 (B flag) is ignored on pull — always cleared in P
    /// - Bit 5 (unused) is always set to 1 after pull
    /// - These are hardwired behaviors
    ///
    /// Cycle-by-cycle (4 cycles):
    /// 1. Dummy read from PC
    /// 2. Dummy read from current stack [0x0100 + S]
    /// 3. Increment S, read new location, apply bit fixes, load into P
    pub(crate) const fn plp() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "plp_dummy_read1",
            micro_fn: |cpu, bus| {
                // Cycle 1: Dummy read from PC
                let _ = bus.read(cpu.pc);
            },
        };
        const OP2: MicroOp = MicroOp {
            name: "plp_dummy_read2",
            micro_fn: |cpu, bus| {
                // Cycle 2: Dummy read from current stack location
                let _ = bus.read(0x0100 | cpu.s as u16);
            },
        };
        const OP3: MicroOp = MicroOp {
            name: "plp_pull_status",
            micro_fn: |cpu, bus| {
                // Cycle 3: Increment S first
                cpu.s = cpu.s.wrapping_add(1);
                let value = bus.read(0x0100 | cpu.s as u16);

                // Hardware behavior:
                // - Clear B flag (bit 4): & 0xEF
                // - Force unused bit 5 to 1: | 0x20
                let masked = (value & 0xEF) | 0x20;
                cpu.p = Status::from_byte(masked);
            },
        };
        &[OP1, OP2, OP3]
    }
}
