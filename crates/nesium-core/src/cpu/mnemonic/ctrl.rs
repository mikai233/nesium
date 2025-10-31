use crate::{bus::Bus, cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status}};

impl Mnemonic {
    // ================================================================
    //  BRK - Force Interrupt
    // ================================================================
    /// 🕹️ Purpose:
    ///     Forces a software interrupt by pushing PC and P to the stack and
    ///     jumping to the IRQ vector.
    ///
    /// ⚙️ Operation:
    ///     PC+2 ↑ ; push(PC_high) ; push(PC_low)
    ///     push(P | B | UNUSED)
    ///     I ← 1
    ///     PC ← read16($FFFE)
    ///
    /// 🧩 Flags Affected:
    ///     B (set in pushed copy only), I
    pub(crate) const fn brk() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "brk",
            micro_fn: |cpu, bus| {
                let return_pc = cpu.pc.wrapping_add(1);
                // push PC high then low
                bus.write(0x0100 + cpu.s as u16, (return_pc >> 8) as u8);
                cpu.s = cpu.s.wrapping_sub(1);
                bus.write(0x0100 + cpu.s as u16, (return_pc & 0xFF) as u8);
                cpu.s = cpu.s.wrapping_sub(1);

                // push P with BREAK and UNUSED set
                let p = cpu.p | Status::BREAK | Status::UNUSED;
                bus.write(0x0100 + cpu.s as u16, p.bits());
                cpu.s = cpu.s.wrapping_sub(1);

                // set interrupt disable
                cpu.p.insert(Status::INTERRUPT);

                // jump to IRQ vector at $FFFE/$FFFF
                let lo = bus.read(0xFFFE) as u16;
                let hi = bus.read(0xFFFF) as u16;
                cpu.pc = (hi << 8) | lo;
            },
        };
        &[OP1]
    }

    // ================================================================
    //  JMP - Jump
    // ================================================================
    /// 🕹️ Purpose:
    ///     Jump to an absolute address.
    ///
    /// ⚙️ Operation:
    ///     PC ← target_address
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn jmp() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "jmp",
            micro_fn: |cpu, bus| {
                let lo = bus.read(cpu.pc) as u16;
                cpu.incr_pc();
                let hi = bus.read(cpu.pc) as u16;
                cpu.pc = (hi << 8) | lo;
            },
        };
        &[OP1]
    }

    // ================================================================
    //  JSR - Jump to Subroutine
    // ================================================================
    /// 🕹️ Purpose:
    ///     Push return address (PC+2-1) onto stack and jump to subroutine.
    ///
    /// ⚙️ Operation:
    ///     push(PC_high), push(PC_low)
    ///     PC ← target
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn jsr() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "jsr",
            micro_fn: |cpu, bus| {
                let return_pc = cpu.pc.wrapping_add(1);
                bus.write(0x0100 + cpu.s as u16, (return_pc >> 8) as u8);
                cpu.s = cpu.s.wrapping_sub(1);
                bus.write(0x0100 + cpu.s as u16, (return_pc & 0xFF) as u8);
                cpu.s = cpu.s.wrapping_sub(1);

                let lo = bus.read(cpu.pc) as u16;
                cpu.pc = cpu.pc.wrapping_add(1);
                let hi = bus.read(cpu.pc) as u16;
                cpu.pc = (hi << 8) | lo;
            },
        };
        &[OP1]
    }

    // ================================================================
    //  RTI - Return from Interrupt
    // ================================================================
    /// 🕹️ Purpose:
    ///     Pull status and PC from stack to restore CPU state before interrupt.
    ///
    /// ⚙️ Operation:
    ///     P ← pull()
    ///     PC ← pull16()
    ///
    /// 🧩 Flags Affected:
    ///     All restored from stack
    pub(crate) const fn rti() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rti",
            micro_fn: |cpu, bus| {
                cpu.s = cpu.s.wrapping_add(1);
                let p_byte = bus.read(0x0100 + cpu.s as u16);
                cpu.p = Status::from_byte(p_byte & 0xEF); // clear B flag

                cpu.s = cpu.s.wrapping_add(1);
                let lo = bus.read(0x0100 + cpu.s as u16);
                cpu.s = cpu.s.wrapping_add(1);
                let hi = bus.read(0x0100 + cpu.s as u16);
                cpu.pc = (hi as u16) << 8 | (lo as u16);
            },
        };
        &[OP1]
    }

    // ================================================================
    //  RTS - Return from Subroutine
    // ================================================================
    /// 🕹️ Purpose:
    ///     Pull PC from stack, increment, resume execution.
    ///
    /// ⚙️ Operation:
    ///     PC ← pull16() + 1
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn rts() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "rts",
            micro_fn: |cpu, bus| {
                cpu.s = cpu.s.wrapping_add(1);
                let lo = bus.read(0x0100 + cpu.s as u16);
                cpu.s = cpu.s.wrapping_add(1);
                let hi = bus.read(0x0100 + cpu.s as u16);
                cpu.pc = ((hi as u16) << 8 | (lo as u16)).wrapping_add(1);
            },
        };
        &[OP1]
    }
}
