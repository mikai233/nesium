use crate::{
    bus::Bus,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    /// NV-BDIZC
    /// -----1--
    ///
    /// BRK - Break Command
    /// Operation: PC + 2↓, [FFFE] → PCL, [FFFF] → PCH
    ///
    /// The break command causes the microprocessor to go through an interrupt
    /// sequence under program control. This means that the program counter of the
    /// second byte after the BRK is automatically stored on the stack along with the
    /// processor status at the beginning of the break instruction. The
    /// microprocessor then transfers control to the interrupt vector.
    ///
    /// Other than changing the program counter, the break instruction changes no
    /// values in either the registers or the flags.
    ///
    /// **Note on the MOS 6502:**
    /// If an IRQ happens at the same time as a BRK instruction, the BRK instruction
    /// is ignored.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | BRK                      | $00    | 1         | 7
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

    /// NV-BDIZC
    /// --------
    ///
    /// JMP - JMP Indirect
    /// Operation: [PC + 1] → PCL, [PC + 2] → PCH
    ///
    /// This instruction establishes a new value for the program counter.
    ///
    /// It affects only the program counter in the microprocessor and affects no
    /// flags in the status register.
    ///
    /// Addressing Mode     | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// ------------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute            | JMP $nnnn                | $4C    | 3         | 3
    /// Absolute Indirect   | JMP ($nnnn)              | $6C    | 3         | 5
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

    /// NV-BDIZC
    /// --------
    ///
    /// JSR - Jump To Subroutine
    /// Operation: PC + 2↓, [PC + 1] → PCL, [PC + 2] → PCH
    ///
    /// This instruction transfers control of the program counter to a subroutine
    /// location but leaves a return pointer on the stack to allow the user to return
    /// to perform the next instruction in the main program after the subroutine is
    /// complete. To accomplish this, JSR instruction stores the program counter
    /// address which points to the last byte of the jump instruction onto the stack
    /// using the stack pointer. The stack byte contains the program count high
    /// first, followed by program count low. The JSR then transfers the addresses
    /// following the jump instruction to the program counter low and the program
    /// counter high, thereby directing the program to begin at that new address.
    ///
    /// The JSR instruction affects no flags, causes the stack pointer to be
    /// decremented by 2 and substitutes new values into the program counter low and
    /// the program counter high.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Absolute        | JSR $nnnn                | $20    | 3         | 6
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

    /// NV-BDIZC
    /// ✓✓--✓✓✓✓
    ///
    /// RTI - Return From Interrupt
    /// Operation: P↑ PC↑
    ///
    /// This instruction transfers from the stack into the microprocessor the
    /// processor status and the program counter location for the instruction which
    /// was interrupted. By virtue of the interrupt having stored this data before
    /// executing the instruction and the fact that the RTI reinitializes the
    /// microprocessor to the same state as when it was interrupted, the combination
    /// of interrupt plus RTI allows truly reentrant coding.
    ///
    /// The RTI instruction reinitializes all flags to the position to the point they
    /// were at the time the interrupt was taken and sets the program counter back to
    /// its pre-interrupt state. It affects no other registers in the microprocessor.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | RTI                      | $40    | 1         | 6
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

    /// NV-BDIZC
    /// --------
    ///
    /// RTS - Return From Subroutine
    /// Operation: PC↑, PC + 1 → PC
    ///
    /// This instruction loads the program count low and program count high from the
    /// stack into the program counter and increments the program counter so that it
    /// points to the instruction following the JSR. The stack pointer is adjusted
    /// by incrementing it twice.
    ///
    /// The RTS instruction does not affect any flags and affects only PCL and PCH.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | RTS                    | $60    | 1         | 6
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
