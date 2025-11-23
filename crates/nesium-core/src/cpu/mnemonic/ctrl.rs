use crate::{
    bus::STACK_ADDR,
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
        &[
            // T2: Dummy Read (Padding Byte)
            MicroOp {
                name: "brk_dummy_read",
                // Bus: READ PC + 1. The byte immediately following BRK is read and discarded.
                // Internal: PC is incremented past the padding byte.
                micro_fn: |cpu, bus| {
                    bus.read(cpu.pc); // Read the byte at PC (which is PC + 1 after T1 fetch)
                    cpu.incr_pc(); //TODO check pc
                },
            },
            // T3: Push PC High Byte (W)
            MicroOp {
                name: "brk_push_pc_hi",
                // Bus: WRITE PC_H to Stack (0x0100 + S).
                // Internal: Stack Pointer (S) is decremented.
                micro_fn: |cpu, bus| {
                    let pc_hi = (cpu.pc >> 8) as u8;
                    cpu.push(bus, pc_hi);
                },
            },
            // T4: Push PC Low Byte (W)
            MicroOp {
                name: "brk_push_pc_lo",
                // Bus: WRITE PC_L to Stack (0x0100 + S).
                // Internal: Stack Pointer (S) is decremented.
                micro_fn: |cpu, bus| {
                    let pc_lo = (cpu.pc & 0xFF) as u8;
                    cpu.push(bus, pc_lo);
                },
            },
            // T5: Push Status Register P (W)
            MicroOp {
                name: "brk_push_p",
                // Bus: WRITE Status Register P to Stack. Pushed flags must have BREAK (B) and UNUSED (U) set.
                // Internal: Stack Pointer (S) is decremented. Status Register's I (Interrupt Disable) flag is set.
                micro_fn: |cpu, bus| {
                    // Pushed P must have the BREAK (0x10) and UNUSED (0x20) flags set.
                    let p_with_b_u = cpu.p | Status::BREAK | Status::UNUSED;
                    cpu.push(bus, p_with_b_u.bits());

                    // Set Interrupt Disable flag *after* pushing P
                    cpu.p.set_i(true);
                },
            },
            // T6: Read Interrupt Vector Low Byte (R)
            MicroOp {
                name: "brk_read_vector_lo",
                // Bus: READ $FFFE (IRQ/BRK vector low byte).
                // Internal: Low byte is temporarily stored.
                micro_fn: |cpu, bus| {
                    // Read from IRQ/BRK vector address
                    cpu.base = bus.read(0xFFFE);
                },
            },
            // T7: Read Interrupt Vector High Byte (R) and Final PC Update
            MicroOp {
                name: "brk_read_vector_hi",
                // Bus: READ $FFFF (IRQ/BRK vector high byte).
                // Internal: Combine bytes and update PC. This is the last cycle.
                micro_fn: |cpu, bus| {
                    // Read high byte from IRQ/BRK vector address
                    let high_byte = bus.read(0xFFFF);

                    // Final PC update
                    cpu.pc = ((high_byte as u16) << 8) | (cpu.base as u16);
                },
            },
        ]
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
        &[]
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
        &[
            // T2: Fetch Target Address Low Byte (R)
            MicroOp {
                name: "jsr_fetch_lo",
                // Bus: READ target address LSB from PC.
                // Internal: Store LSB in cpu.base. PC is advanced to point to the HI byte address.
                micro_fn: |cpu, bus| {
                    // Read LSB (target address) and store it in cpu.base
                    cpu.base = bus.read(cpu.pc);
                    // PC increments (PC + 1), now pointing to the HI byte address
                    cpu.incr_pc();
                },
            },
            // T3: Dummy Read from Stack Address (R_dummy) & Internal PC Prepare
            MicroOp {
                name: "jsr_dummy_read_pc_prep",
                // Bus: Dummy READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: Store the calculated return address (PC + 2) in effective_addr for pushing.
                micro_fn: |cpu, bus| {
                    // The correct value to push is the current PC (which is PC + 2 relative to the opcode address).
                    let return_pc = cpu.pc;

                    // Store full Return PC into effective_addr for T4/T5 push.
                    cpu.effective_addr = return_pc;

                    // Dummy Read cycle
                    bus.read(STACK_ADDR + cpu.s as u16);
                },
            },
            // T4: Push PC High Byte (W)
            MicroOp {
                name: "jsr_push_pc_hi",
                // Bus: WRITE PC_H (from effective_addr) to Stack.
                // Internal: Stack Pointer (S) is decremented (Handled by cpu.push).
                micro_fn: |cpu, bus| {
                    let pc_hi = (cpu.effective_addr >> 8) as u8;
                    cpu.push(bus, pc_hi);
                },
            },
            // T5: Push PC Low Byte (W)
            MicroOp {
                name: "jsr_push_pc_lo",
                // Bus: WRITE PC_L (from effective_addr) to Stack.
                // Internal: Stack Pointer (S) is decremented (Handled by cpu.push).
                micro_fn: |cpu, bus| {
                    let pc_lo = cpu.effective_addr as u8;
                    cpu.push(bus, pc_lo);
                },
            },
            // T6: Fetch Target Address High Byte (R) and Final Jump
            MicroOp {
                name: "jsr_fetch_hi_jump",
                // Bus: READ target address HSB from PC.
                // Internal: Combine HSB with LSB (stored in cpu.base) and update PC.
                micro_fn: |cpu, bus| {
                    // Read HSB of target address from the current PC
                    let hi_byte = bus.read(cpu.pc) as u16;

                    // Get the LSB of the target address (from cpu.base)
                    let lo_byte = cpu.base as u16;

                    // Final PC update: Jump to target address
                    let target_addr = (hi_byte << 8) | lo_byte;
                    cpu.pc = target_addr;
                },
            },
        ]
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
        &[
            // T2: Dummy Read (PC + 1)
            MicroOp {
                name: "rti_dummy_read_pc",
                // Bus: READ from the address AFTER the RTI opcode. Value is discarded.
                // Internal: S remains unchanged. PC should have been auto-incremented to PC+1.
                micro_fn: |cpu, bus| {
                    // NOTE: The address is typically PC+1, where PC is the address of the RTI instruction.
                    // We read the effective address of the next instruction, which is often PC_Start + 1.
                    // In a cycle-accurate model, this T2 read should be PC + 1.
                    bus.read(cpu.pc.wrapping_add(1));
                },
            },
            // T3: Dummy Read (Stack Pointer S)
            MicroOp {
                name: "rti_dummy_read_stack",
                // Bus: READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: S remains unchanged. This is the empty stack cycle.
                micro_fn: |cpu, bus| {
                    // Read from the current stack pointer location. Value is ignored.
                    bus.read(STACK_ADDR + cpu.s as u16);
                },
            },
            // T4: Pop Status Register P
            MicroOp {
                name: "rti_pop_p",
                // Bus: READ Status Register P from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. Status Register P is updated.
                micro_fn: |cpu, bus| {
                    // Stack Pop: S increments (Post-Increment), then read.
                    let p_bits = cpu.pull(bus);

                    // Set P register (0x20 UNUSED flag must be restored/set)
                    cpu.p = Status::from_bits_truncate(p_bits);
                    cpu.p.set_u(true); // Always set UNUSED flag (B/0x10 is ignored/removed by from_bits_truncate)
                    cpu.p.set_b(false);
                    // I flag restored by RTI should also flow through the IRQ
                    // gating pipeline with instruction-boundary latency.
                    cpu.queue_i_update(cpu.p.i());
                },
            },
            // T5: Pop PC Low Byte
            MicroOp {
                name: "rti_pop_pc_lo",
                // Bus: READ PC_L from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC_L stored in cpu.base.
                micro_fn: |cpu, bus| {
                    // Read PC Low byte and store it temporarily in cpu.base
                    cpu.base = cpu.pull(bus);
                },
            },
            // T6: Pop PC High Byte (Final Jump)
            MicroOp {
                name: "rti_pop_pc_hi_jump",
                // Bus: READ PC_H from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC is updated to the restored address.
                micro_fn: |cpu, bus| {
                    // Read PC High byte
                    let hi_byte = cpu.pull(bus) as u16;

                    // Combine the popped HI and LO bytes to form the new PC address
                    let new_pc = (hi_byte << 8) | (cpu.base as u16);

                    // Final PC update
                    cpu.pc = new_pc;
                },
            },
        ]
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
        &[
            // T2: Dummy Read 1 (Stack Address)
            MicroOp {
                name: "rts_dummy_read_1",
                // Bus: READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: S remains unchanged.
                micro_fn: |cpu, bus| {
                    // Read from the current stack pointer location. Value is always ignored.
                    bus.read(STACK_ADDR + cpu.s as u16);
                },
            },
            // T3: Dummy Read 2 (Stack Address)
            MicroOp {
                name: "rts_dummy_read_2",
                // Bus: READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: S remains unchanged. This is a characteristic delay cycle.
                micro_fn: |cpu, bus| {
                    // Read again from the current stack pointer location. Value is ignored.
                    bus.read(STACK_ADDR + cpu.s as u16);
                },
            },
            // T4: Pop PC Low Byte (R)
            MicroOp {
                name: "rts_pop_pc_lo",
                // Bus: READ PC_L from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented (Post-Increment). PC_L stored in cpu.base.
                micro_fn: |cpu, bus| {
                    // Read PC Low byte and store it temporarily in cpu.base
                    cpu.base = cpu.pull(bus);
                },
            },
            // T5: Pop PC High Byte (R)
            MicroOp {
                name: "rts_pop_pc_hi",
                // Bus: READ PC_H from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC_H stored in cpu.effective_addr.
                micro_fn: |cpu, bus| {
                    // Read PC High byte and store it in effective_addr's high byte
                    let hi_byte = cpu.pull(bus) as u16;

                    // Combine the popped HI and LO bytes to form the saved PC (PC_saved)
                    let saved_pc = (hi_byte << 8) | (cpu.base as u16);

                    // Store the full saved address for the final T6 increment/jump
                    cpu.effective_addr = saved_pc;
                },
            },
            // T6: Dummy Read (Increment and Jump)
            MicroOp {
                name: "rts_dummy_read_increment_jump",
                // Bus: Dummy READ from the calculated effective address (PC_saved). Value is discarded.
                // Internal: Increment the saved PC (PC_saved + 1) and set it as the new PC.
                micro_fn: |cpu, bus| {
                    // Dummy read from the saved PC (PC_saved) address
                    bus.read(cpu.effective_addr);

                    // Internal calculation: Increment the saved address and update PC
                    cpu.pc = cpu.effective_addr.wrapping_add(1);

                    // Note: The instruction execution ends here. The next cycle will be the fetch
                    // from the new PC (PC_saved + 1).
                },
            },
        ]
    }
}

#[cfg(test)]
mod ctrl_tests {
    use crate::{
        bus::STACK_ADDR,
        cpu::{
            mnemonic::{Mnemonic, tests::InstrTest},
            status::Status,
        },
    };

    #[test]
    fn test_brk() {
        InstrTest::new(Mnemonic::BRK).test(|verify, cpu, bus| {
            // BRK should push PC+2 and the status register onto the stack.
            // The BRK instruction is two bytes long, so the return address is PC + 2.
            let expected_return_addr = verify.cpu.pc.wrapping_add(2);

            // --- Stack Pointer Check ---

            // Stack pointer should decrease by 3 (PC high, PC low, Status register)
            let expected_s = verify.cpu.s.wrapping_sub(3);
            assert_eq!(cpu.s, expected_s, "Stack pointer not updated correctly");

            // --- Pushed Address Check ---

            let expected_addr_hi = (expected_return_addr >> 8) as u8;
            let expected_addr_lo = expected_return_addr as u8;

            // Verify the pushed high byte of the return address (pushed first)
            let mut stack_ptr = verify.cpu.s;
            let stack_addr_hi = STACK_ADDR | (stack_ptr as u16);
            assert_eq!(
                bus.read(stack_addr_hi),
                expected_addr_hi,
                "Return address high byte not pushed correctly"
            );

            // Verify the pushed low byte of the return address
            stack_ptr = stack_ptr.wrapping_sub(1);
            let stack_addr_lo = STACK_ADDR | (stack_ptr as u16);
            assert_eq!(
                bus.read(stack_addr_lo),
                expected_addr_lo,
                "Return address low byte not pushed correctly"
            );

            // --- Pushed Status Register Check ---

            stack_ptr = stack_ptr.wrapping_sub(1);
            let stack_addr_status = STACK_ADDR | (stack_ptr as u16);
            let pushed_status = bus.read(stack_addr_status);

            // Construct the expected Pushed Status (P_in | B | U)
            // 1. Start with the CPU's status bits before execution.
            let mut expected_pushed_status = verify.cpu.p.bits();

            // 2. The BREAK flag (B, Bit 4) MUST be set (1) when pushed by BRK.
            expected_pushed_status |= Status::BREAK.bits();

            // 3. The UNUSED flag (U, Bit 5) MUST always be set (1) on NMOS 6502/2A03.
            expected_pushed_status |= Status::UNUSED.bits();

            assert_eq!(
                pushed_status, expected_pushed_status,
                "Pushed status byte mismatch (B/U flags check failed)"
            );

            // --- CPU Status and PC Update Check ---

            // The Interrupt Disable flag (I) should be set after the interrupt.
            assert!(
                cpu.p.contains(Status::INTERRUPT),
                "Interrupt disable flag not set"
            );

            // PC should be loaded from the IRQ/BRK vector ($FFFE/$FFFF).
            let irq_vector_lo = bus.read(0xFFFE) as u16;
            let irq_vector_hi = bus.read(0xFFFF) as u16;
            let expected_pc = (irq_vector_hi << 8) | irq_vector_lo;
            assert_eq!(
                cpu.pc, expected_pc,
                "PC not set to interrupt vector address"
            );
        });
    }

    #[test]
    fn test_jmp() {
        // This test assumes that verify.addr already holds the final target PC,
        // handling both Absolute ($4C) and Indirect ($6C) modes, including the $XXFF page wrap bug.
        InstrTest::new(Mnemonic::JMP).test(|verify, cpu, _| {
            // JMP does not affect the status register.
            assert_eq!(
                cpu.p.bits(),
                verify.cpu.p.bits(),
                "JMP should not affect status flags."
            );

            // The expected PC is the final address calculated by the addressing mode.
            let expected_pc = verify.addr;

            // The bus parameter is unused in this simplified JMP test,
            // as the target address is assumed to be pre-calculated in verify.addr.

            // Assert that the CPU's PC register has been set to the calculated jump target.
            assert_eq!(
                cpu.pc, expected_pc,
                "PC not set to the expected jump address."
            );
        });
    }

    #[test]
    fn test_jsr() {
        InstrTest::new(Mnemonic::JSR).test(|verify, cpu, bus| {
            // JSR is a 3-byte instruction (Opcode + 2-byte address).
            // The return address pushed is PC + 2.
            let expected_return_addr = verify.cpu.pc.wrapping_add(2);

            // --- 1. Stack Pointer Check ---

            // JSR pushes 2 bytes (PC high, PC low), so SP should decrease by 2.
            let expected_s = verify.cpu.s.wrapping_sub(2);
            assert_eq!(
                cpu.s, expected_s,
                "Stack pointer not updated correctly after JSR."
            );

            // --- 2. Pushed Address Check ---

            // The return address is pushed high byte first, then low byte.
            let expected_addr_hi = (expected_return_addr >> 8) as u8;
            let expected_addr_lo = expected_return_addr as u8;

            // Verify the pushed high byte (pushed first, at S_in)
            let mut stack_ptr = verify.cpu.s;
            let stack_addr_hi = STACK_ADDR | (stack_ptr as u16);
            assert_eq!(
                bus.read(stack_addr_hi),
                expected_addr_hi,
                "Return address high byte (PC+2) not pushed correctly."
            );

            // Verify the pushed low byte (pushed second, at S_in - 1)
            stack_ptr = stack_ptr.wrapping_sub(1);
            let stack_addr_lo = STACK_ADDR | (stack_ptr as u16);
            assert_eq!(
                bus.read(stack_addr_lo),
                expected_addr_lo,
                "Return address low byte (PC+2) not pushed correctly."
            );

            // --- 3. Status Register Check ---

            // JSR does not affect the status register.
            assert_eq!(
                cpu.p.bits(),
                verify.cpu.p.bits(),
                "JSR should not affect status flags."
            );

            // --- 4. PC Update Check ---

            // PC should be set to the target address, which is assumed to be in verify.addr.
            let expected_pc = verify.addr;
            assert_eq!(
                cpu.pc, expected_pc,
                "PC not set to the expected subroutine address."
            );
        });
    }

    #[test]
    fn test_rti() {
        InstrTest::new(Mnemonic::RTI).test(|verify, cpu, bus| {
            // --- Setup for Verification ---

            // The initial stack pointer (S_in) points to the highest stack address (0x01XX).
            // RTI will read from S_in + 1, S_in + 2, and S_in + 3.
            let initial_s = verify.cpu.s;

            // Read the expected values from the memory (where they were pushed by BRK/IRQ/NMI).

            // 1. Expected Status (P): Read from S_in + 1.
            let expected_status_addr = STACK_ADDR | initial_s.wrapping_add(1) as u16;
            let expected_status_bits = bus.read(expected_status_addr);

            // 2. Expected PC Low: Read from S_in + 2.
            let expected_pc_lo_addr = STACK_ADDR | initial_s.wrapping_add(2) as u16;
            let expected_pc_lo = bus.read(expected_pc_lo_addr) as u16;

            // 3. Expected PC High: Read from S_in + 3.
            let expected_pc_hi_addr = STACK_ADDR | initial_s.wrapping_add(3) as u16;
            let expected_pc_hi = bus.read(expected_pc_hi_addr) as u16;

            let expected_pc = (expected_pc_hi << 8) | expected_pc_lo;

            // --- 1. Status Register Check ---

            // RTI should restore the status register from the stack (S_in + 1).
            // NOTE: The B (Break) flag (Bit 4) and U (Unused) flag (Bit 5) are ignored
            // when pulling P from the stack. The B flag is always cleared in the CPU's P register.
            let mut actual_status_bits = cpu.p.bits();
            // Mask out the B flag (Bit 4) and U flag (Bit 5) from the read value,
            // as they are not meant to be set in the live CPU P register.
            let mask = !(Status::BREAK.bits() | Status::UNUSED.bits());

            // Ensure the B and U flags are cleared in the CPU's P register.
            actual_status_bits &= mask;

            // Check if the restored P (ignoring B/U flags) matches the expected P (ignoring B/U).
            assert_eq!(
                actual_status_bits,
                expected_status_bits & mask,
                "Status register (P) not restored correctly from stack (ignoring B/U flags)."
            );

            // --- 2. PC Update Check ---

            // RTI should restore the PC from the stack (PC low then PC high).
            assert_eq!(cpu.pc, expected_pc, "PC not restored correctly from stack.");

            // --- 3. Stack Pointer Check ---

            // RTI pops 3 bytes (P, PC_lo, PC_hi), so SP should increase by 3.
            let expected_s = initial_s.wrapping_add(3);
            assert_eq!(
                cpu.s, expected_s,
                "Stack pointer not updated correctly after RTI."
            );

            // RTI does not affect A, X, or Y registers.
            assert_eq!(cpu.a, verify.cpu.a, "A register should be unchanged.");
            assert_eq!(cpu.x, verify.cpu.x, "X register should be unchanged.");
            assert_eq!(cpu.y, verify.cpu.y, "Y register should be unchanged.");
        });
    }

    #[test]
    fn test_rts() {
        InstrTest::new(Mnemonic::RTS).test(|verify, cpu, bus| {
            // --- Setup for Verification ---

            // The initial stack pointer (S_in) points to the last address pushed.
            // RTS will read from S_in + 1 and S_in + 2.
            let initial_s = verify.cpu.s;

            // 1. Read the expected PC Low: From S_in + 1.
            let expected_pc_lo_addr = STACK_ADDR | initial_s.wrapping_add(1) as u16;
            let expected_pc_lo = bus.read(expected_pc_lo_addr) as u16;

            // 2. Read the expected PC High: From S_in + 2.
            let expected_pc_hi_addr = STACK_ADDR | initial_s.wrapping_add(2) as u16;
            let expected_pc_hi = bus.read(expected_pc_hi_addr) as u16;

            // The address popped from stack is P_return = PC_pushed (usually PC_JSR + 2).
            let pc_popped = (expected_pc_hi << 8) | expected_pc_lo;

            // The final execution address is P_return + 1.
            let expected_pc = pc_popped.wrapping_add(1);

            // --- 1. PC Update Check ---

            // RTS should restore PC and then increment it by 1.
            assert_eq!(
                cpu.pc, expected_pc,
                "PC not restored (Popped PC + 1) correctly from stack."
            );

            // --- 2. Stack Pointer Check ---

            // RTS pops 2 bytes (PC_lo, PC_hi), so SP should increase by 2.
            let expected_s = initial_s.wrapping_add(2);
            assert_eq!(
                cpu.s, expected_s,
                "Stack pointer not updated correctly after RTS."
            );

            // --- 3. Status Register Check ---

            // RTS does not affect the status register.
            assert_eq!(
                cpu.p.bits(),
                verify.cpu.p.bits(),
                "RTS should not affect status flags."
            );

            // RTS does not affect A, X, or Y registers.
            assert_eq!(cpu.a, verify.cpu.a, "A register should be unchanged.");
            assert_eq!(cpu.x, verify.cpu.x, "X register should be unchanged.");
            assert_eq!(cpu.y, verify.cpu.y, "Y register should be unchanged.");
        });
    }
}
