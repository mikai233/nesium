use crate::{
    bus::{CpuBus, STACK_ADDR},
    cartridge::CpuBusAccessKind,
    context::Context,
    cpu::{Cpu, status::Status},
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
pub fn exec_brk(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            let pc_hi = ((cpu.pc + 1) >> 8) as u8;
            cpu.push_stack(bus, ctx, pc_hi);
        }
        2 => {
            let pc_lo = ((cpu.pc + 1) & 0xFF) as u8;
            cpu.push_stack(bus, ctx, pc_lo);
            if cpu.nmi_latch {
                cpu.nmi_latch = false;
                cpu.effective_addr = NMI_VECTOR_LO;
            } else {
                cpu.effective_addr = IRQ_VECTOR_LO;
            }
        }
        3 => {
            let p_with_b_u = cpu.p | Status::BREAK | Status::UNUSED;
            cpu.push_stack(bus, ctx, p_with_b_u.bits());
            cpu.p.insert(Status::INTERRUPT);
        }
        4 => {
            cpu.tmp = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
            if cpu.effective_addr == NMI_VECTOR_LO {
                cpu.effective_addr = NMI_VECTOR_HI;
            } else {
                cpu.effective_addr = IRQ_VECTOR_HI;
            }
        }
        5 => {
            let high_byte = cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::Read);
            cpu.pc = ((high_byte as u16) << 8) | (cpu.tmp as u16);
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
pub fn exec_jmp(_: &mut Cpu, _: &mut CpuBus, _: &mut Context, step: u8) {
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
pub fn exec_jsr(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_operand_u8(bus, ctx);
        }
        1 => {
            let return_pc = cpu.pc;
            cpu.effective_addr = return_pc;
            cpu.read(
                STACK_ADDR + cpu.s as u16,
                bus,
                ctx,
                CpuBusAccessKind::DummyRead,
            );
        }
        2 => {
            let pc_hi = (cpu.effective_addr >> 8) as u8;
            cpu.push_stack(bus, ctx, pc_hi);
        }
        3 => {
            let pc_lo = cpu.effective_addr as u8;
            cpu.push_stack(bus, ctx, pc_lo);
        }
        4 => {
            let hi_byte = cpu.read(cpu.pc, bus, ctx, CpuBusAccessKind::ExecOperand) as u16;
            let lo_byte = cpu.tmp as u16;
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
pub fn exec_rti(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            cpu.read(
                STACK_ADDR + cpu.s as u16,
                bus,
                ctx,
                CpuBusAccessKind::DummyRead,
            );
        }
        2 => {
            let p_bits = cpu.pop_stack(bus, ctx);
            cpu.p = Status::from_bits_truncate(p_bits);
            cpu.p.remove(Status::UNUSED | Status::BREAK);
        }
        3 => {
            cpu.tmp = cpu.pop_stack(bus, ctx);
        }
        4 => {
            let hi_byte = cpu.pop_stack(bus, ctx) as u16;
            cpu.pc = (hi_byte << 8) | (cpu.tmp as u16);
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
pub fn exec_rts(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        1 => {
            cpu.read(
                STACK_ADDR + cpu.s as u16,
                bus,
                ctx,
                CpuBusAccessKind::DummyRead,
            );
        }
        2 => {
            cpu.tmp = cpu.pop_stack(bus, ctx);
        }
        3 => {
            let hi_byte = cpu.pop_stack(bus, ctx) as u16;
            cpu.effective_addr = (hi_byte << 8) | (cpu.tmp as u16);
        }
        4 => {
            cpu.read(cpu.effective_addr, bus, ctx, CpuBusAccessKind::DummyRead);
            cpu.pc = cpu.effective_addr.wrapping_add(1);
        }
        _ => unreachable_step!("invalid RTS step {step}"),
    }
}
