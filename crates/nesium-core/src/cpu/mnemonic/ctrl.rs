use crate::{
    bus::{CpuBus, STACK_ADDR},
    context::Context,
    cpu::{Cpu, micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
    memory::cpu::{IRQ_VECTOR_HI, IRQ_VECTOR_LO, NMI_VECTOR_HI, NMI_VECTOR_LO},
};

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
#[inline]
pub fn exec_brk(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        1 => {
            let pc_hi = ((cpu.pc + 1) >> 8) as u8;
            cpu.push(bus, ctx, pc_hi);
        }
        2 => {
            let pc_lo = ((cpu.pc + 1) & 0xFF) as u8;
            cpu.push(bus, ctx, pc_lo);
            if cpu.nmi_latch {
                cpu.nmi_latch = false;
                cpu.effective_addr = NMI_VECTOR_LO;
            } else {
                cpu.effective_addr = IRQ_VECTOR_LO;
            }
        }
        3 => {
            let p_with_b_u = cpu.p | Status::BREAK | Status::UNUSED;
            cpu.push(bus, ctx, p_with_b_u.bits());
            cpu.p.insert(Status::INTERRUPT);
        }
        4 => {
            cpu.base = bus.mem_read(cpu.effective_addr, cpu, ctx);
            if cpu.effective_addr == NMI_VECTOR_LO {
                cpu.effective_addr = NMI_VECTOR_HI;
            } else {
                cpu.effective_addr = IRQ_VECTOR_HI;
            }
        }
        5 => {
            let high_byte = bus.mem_read(cpu.effective_addr, cpu, ctx);
            cpu.pc = ((high_byte as u16) << 8) | (cpu.base as u16);
            cpu.prev_nmi_latch = false;
        }
        _ => unreachable_step!("invalid BRK step {step}"),
    }
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
#[inline]
pub fn exec_jmp(_: &mut Cpu, _: &mut CpuBus<'_>, _: &mut Context, step: u8) {
    unreachable_step!("invalid JMP step {step}");
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
#[inline]
pub fn exec_jsr(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.base = bus.mem_read(cpu.pc, cpu, ctx);
            cpu.incr_pc();
        }
        1 => {
            let return_pc = cpu.pc;
            cpu.effective_addr = return_pc;
            bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
        }
        2 => {
            let pc_hi = (cpu.effective_addr >> 8) as u8;
            cpu.push(bus, ctx, pc_hi);
        }
        3 => {
            let pc_lo = cpu.effective_addr as u8;
            cpu.push(bus, ctx, pc_lo);
        }
        4 => {
            let hi_byte = bus.mem_read(cpu.pc, cpu, ctx) as u16;
            let lo_byte = cpu.base as u16;
            cpu.pc = (hi_byte << 8) | lo_byte;
        }
        _ => unreachable_step!("invalid JSR step {step}"),
    }
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
#[inline]
pub fn exec_rti(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.mem_read(cpu.pc.wrapping_add(1), cpu, ctx);
        }
        1 => {
            bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
        }
        2 => {
            let p_bits = cpu.pull(bus, ctx);
            cpu.p = Status::from_bits_truncate(p_bits);
            cpu.p.remove(Status::UNUSED | Status::BREAK);
        }
        3 => {
            cpu.base = cpu.pull(bus, ctx);
        }
        4 => {
            let hi_byte = cpu.pull(bus, ctx) as u16;
            cpu.pc = (hi_byte << 8) | (cpu.base as u16);
        }
        _ => unreachable_step!("invalid RTI step {step}"),
    }
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
#[inline]
pub fn exec_rts(cpu: &mut Cpu, bus: &mut CpuBus<'_>, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
        }
        1 => {
            bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
        }
        2 => {
            cpu.base = cpu.pull(bus, ctx);
        }
        3 => {
            let hi_byte = cpu.pull(bus, ctx) as u16;
            cpu.effective_addr = (hi_byte << 8) | (cpu.base as u16);
        }
        4 => {
            bus.mem_read(cpu.effective_addr, cpu, ctx);
            cpu.pc = cpu.effective_addr.wrapping_add(1);
        }
        _ => unreachable_step!("invalid RTS step {step}"),
    }
}

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
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx); // Read the byte at PC (which is PC + 1 after T1 fetch)
                    cpu.incr_pc(); //TODO check pc
                },
            },
            // T3: Push PC High Byte (W)
            MicroOp {
                name: "brk_push_pc_hi",
                // Bus: WRITE PC_H to Stack (0x0100 + S).
                // Internal: Stack Pointer (S) is decremented.
                micro_fn: |cpu, bus, ctx| {
                    let pc_hi = (cpu.pc >> 8) as u8;
                    cpu.push(bus, ctx, pc_hi);
                },
            },
            // T4: Push PC Low Byte (W)
            MicroOp {
                name: "brk_push_pc_lo",
                // Bus: WRITE PC_L to Stack (0x0100 + S).
                // Internal: Stack Pointer (S) is decremented.
                micro_fn: |cpu, bus, ctx| {
                    let pc_lo = (cpu.pc & 0xFF) as u8;
                    cpu.push(bus, ctx, pc_lo);
                },
            },
            // T5: Push Status Register P (W)
            MicroOp {
                name: "brk_push_p",
                // Bus: WRITE Status Register P to Stack. Pushed flags must have BREAK (B) and UNUSED (U) set.
                // Internal: Stack Pointer (S) is decremented. Status Register's I (Interrupt Disable) flag is set.
                micro_fn: |cpu, bus, ctx| {
                    // Pushed P must have the BREAK (0x10) and UNUSED (0x20) flags set.
                    let p_with_b_u = cpu.p | Status::BREAK | Status::UNUSED;
                    cpu.push(bus, ctx, p_with_b_u.bits());

                    // Set Interrupt Disable flag *after* pushing P
                    cpu.p.insert(Status::INTERRUPT);
                },
            },
            // T6: Read Interrupt Vector Low Byte (R)
            MicroOp {
                name: "brk_read_vector_lo",
                // Bus: READ $FFFE (IRQ/BRK vector low byte).
                // Internal: Low byte is temporarily stored.
                micro_fn: |cpu, bus, ctx| {
                    // Read from IRQ/BRK vector address
                    cpu.base = bus.mem_read(0xFFFE, cpu, ctx);
                },
            },
            // T7: Read Interrupt Vector High Byte (R) and Final PC Update
            MicroOp {
                name: "brk_read_vector_hi",
                // Bus: READ $FFFF (IRQ/BRK vector high byte).
                // Internal: Combine bytes and update PC. This is the last cycle.
                micro_fn: |cpu, bus, ctx| {
                    // Read high byte from IRQ/BRK vector address
                    let high_byte = bus.mem_read(0xFFFF, cpu, ctx);

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
                micro_fn: |cpu, bus, ctx| {
                    // Read LSB (target address) and store it in cpu.base
                    cpu.base = bus.mem_read(cpu.pc, cpu, ctx);
                    // PC increments (PC + 1), now pointing to the HI byte address
                    cpu.incr_pc();
                },
            },
            // T3: Dummy Read from Stack Address (R_dummy) & Internal PC Prepare
            MicroOp {
                name: "jsr_dummy_read_pc_prep",
                // Bus: Dummy READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: Store the calculated return address (PC + 2) in effective_addr for pushing.
                micro_fn: |cpu, bus, ctx| {
                    // The correct value to push is the current PC (which is PC + 2 relative to the opcode address).
                    let return_pc = cpu.pc;

                    // Store full Return PC into effective_addr for T4/T5 push.
                    cpu.effective_addr = return_pc;

                    // Dummy Read cycle
                    bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
                },
            },
            // T4: Push PC High Byte (W)
            MicroOp {
                name: "jsr_push_pc_hi",
                // Bus: WRITE PC_H (from effective_addr) to Stack.
                // Internal: Stack Pointer (S) is decremented (Handled by cpu.push).
                micro_fn: |cpu, bus, ctx| {
                    let pc_hi = (cpu.effective_addr >> 8) as u8;
                    cpu.push(bus, ctx, pc_hi);
                },
            },
            // T5: Push PC Low Byte (W)
            MicroOp {
                name: "jsr_push_pc_lo",
                // Bus: WRITE PC_L (from effective_addr) to Stack.
                // Internal: Stack Pointer (S) is decremented (Handled by cpu.push).
                micro_fn: |cpu, bus, ctx| {
                    let pc_lo = cpu.effective_addr as u8;
                    cpu.push(bus, ctx, pc_lo);
                },
            },
            // T6: Fetch Target Address High Byte (R) and Final Jump
            MicroOp {
                name: "jsr_fetch_hi_jump",
                // Bus: READ target address HSB from PC.
                // Internal: Combine HSB with LSB (stored in cpu.base) and update PC.
                micro_fn: |cpu, bus, ctx| {
                    // Read HSB of target address from the current PC
                    let hi_byte = bus.mem_read(cpu.pc, cpu, ctx) as u16;

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
                micro_fn: |cpu, bus, ctx| {
                    // NOTE: The address is typically PC+1, where PC is the address of the RTI instruction.
                    // We read the effective address of the next instruction, which is often PC_Start + 1.
                    // In a cycle-accurate model, this T2 read should be PC + 1.
                    bus.mem_read(cpu.pc.wrapping_add(1), cpu, ctx);
                },
            },
            // T3: Dummy Read (Stack Pointer S)
            MicroOp {
                name: "rti_dummy_read_stack",
                // Bus: READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: S remains unchanged. This is the empty stack cycle.
                micro_fn: |cpu, bus, ctx| {
                    // Read from the current stack pointer location. Value is ignored.
                    bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
                },
            },
            // T4: Pop Status Register P
            MicroOp {
                name: "rti_pop_p",
                // Bus: READ Status Register P from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. Status Register P is updated.
                micro_fn: |cpu, bus, ctx| {
                    // Stack Pop: S increments (Post-Increment), then read.
                    let p_bits = cpu.pull(bus, ctx);

                    // Set P register (0x20 UNUSED flag must be restored/set)
                    cpu.p = Status::from_bits_truncate(p_bits);
                    cpu.p.remove(Status::UNUSED | Status::BREAK);
                },
            },
            // T5: Pop PC Low Byte
            MicroOp {
                name: "rti_pop_pc_lo",
                // Bus: READ PC_L from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC_L stored in cpu.base.
                micro_fn: |cpu, bus, ctx| {
                    // Read PC Low byte and store it temporarily in cpu.base
                    cpu.base = cpu.pull(bus, ctx);
                },
            },
            // T6: Pop PC High Byte (Final Jump)
            MicroOp {
                name: "rti_pop_pc_hi_jump",
                // Bus: READ PC_H from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC is updated to the restored address.
                micro_fn: |cpu, bus, ctx| {
                    // Read PC High byte
                    let hi_byte = cpu.pull(bus, ctx) as u16;

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
                micro_fn: |cpu, bus, ctx| {
                    // Read from the current stack pointer location. Value is always ignored.
                    bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
                },
            },
            // T3: Dummy Read 2 (Stack Address)
            MicroOp {
                name: "rts_dummy_read_2",
                // Bus: READ from the current stack address (0x0100 + S). Value is discarded.
                // Internal: S remains unchanged. This is a characteristic delay cycle.
                micro_fn: |cpu, bus, ctx| {
                    // Read again from the current stack pointer location. Value is ignored.
                    bus.mem_read(STACK_ADDR + cpu.s as u16, cpu, ctx);
                },
            },
            // T4: Pop PC Low Byte (R)
            MicroOp {
                name: "rts_pop_pc_lo",
                // Bus: READ PC_L from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented (Post-Increment). PC_L stored in cpu.base.
                micro_fn: |cpu, bus, ctx| {
                    // Read PC Low byte and store it temporarily in cpu.base
                    cpu.base = cpu.pull(bus, ctx);
                },
            },
            // T5: Pop PC High Byte (R)
            MicroOp {
                name: "rts_pop_pc_hi",
                // Bus: READ PC_H from Stack (0x0100 + S + 1).
                // Internal: Stack Pointer (S) is incremented. PC_H stored in cpu.effective_addr.
                micro_fn: |cpu, bus, ctx| {
                    // Read PC High byte and store it in effective_addr's high byte
                    let hi_byte = cpu.pull(bus, ctx) as u16;

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
                micro_fn: |cpu, bus, ctx| {
                    // Dummy read from the saved PC (PC_saved) address
                    bus.mem_read(cpu.effective_addr, cpu, ctx);

                    // Internal calculation: Increment the saved address and update PC
                    cpu.pc = cpu.effective_addr.wrapping_add(1);

                    // Note: The instruction execution ends here. The next cycle will be the fetch
                    // from the new PC (PC_saved + 1).
                },
            },
        ]
    }
}
