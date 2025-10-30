use crate::{
    bus::Bus,
    cpu::{
        Cpu,
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
        status::Status,
    },
};

// Helper: Perform ADC operation (binary or decimal) and set flags
fn adc_core(cpu: &mut Cpu, mem: u8) {
    let c = if cpu.p.contains(Status::CARRY) { 1 } else { 0 };
    let a = cpu.a as u16;
    let m = mem as u16;
    let sum = a + m + c;

    if cpu.p.contains(Status::DECIMAL) {
        // ---------- BCD Mode (NES uses illegal 6502 BCD) ----------
        let mut low = (cpu.a & 0x0F) + (mem & 0x0F) + c as u8;
        if low > 0x09 {
            low = (low + 0x06) & 0x0F;
        }
        let mut high = (cpu.a >> 4) + (mem >> 4) + (low > 0x0F) as u8;
        if high > 0x09 {
            high += 0x06;
        }
        let result = (high << 4) | low;
        cpu.a = result;

        // Flags in BCD mode:
        cpu.p.set_c(sum >= 0x100); // Carry if binary sum >= 256
        // V is undefined in BCD on NES, but many emulators set it like binary
        let v = (!(a ^ m) & (a ^ sum) & 0x80) != 0;
        cpu.p.set_v(v);
        cpu.p.set_zn(result);
    } else {
        // ---------- Binary Mode ----------
        cpu.a = sum as u8;
        cpu.p.set_c(sum > 0xFF);
        // Overflow: sign change without carry-in matching
        let v = (!(a ^ m) & (a ^ sum) & 0x80) != 0;
        cpu.p.set_v(v);
        cpu.p.set_zn(cpu.a);
    }
}

// ================================================================
// 1. Immediate: ADC #$nn $69 2 bytes, 2 cycles
// ================================================================
pub const fn adc_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "adc_imm",
        micro_fn: |cpu, bus| {
            let imm = bus.read(cpu.pc);
            adc_core(cpu, imm);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
// 2. Zero Page: ADC $nn $65 2 bytes, 3 cycles
// ================================================================
pub const fn adc_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_addr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.tmp as u16);
            adc_core(cpu, mem);
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
// 3. Zero Page,X: ADC $nn,X $75 2 bytes, 4 cycles
// ================================================================
pub const fn adc_zero_page_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_base",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x",
        micro_fn: |cpu, _| {
            cpu.effective_addr = (cpu.tmp as u16).wrapping_add(cpu.x as u16);
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 4. Absolute: ADC $nnnn $6D 3 bytes, 4 cycles
// ================================================================
pub const fn adc_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
// 5. Absolute,X: ADC $nnnn,X $7D 3 bytes, 4+p cycles
// ================================================================
pub const fn adc_absolute_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_add_x",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.x as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let _ = bus.read(base); // dummy read
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 6. Absolute,Y: ADC $nnnn,Y $79 3 bytes, 4+p cycles
// ================================================================
pub const fn adc_absolute_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_add_y",
        micro_fn: |cpu, bus| {
            let hi = bus.read(cpu.pc);
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base); // dummy read
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
// 7. (Indirect,X): ADC ($nn,X) $61 2 bytes, 6 cycles
// ================================================================
pub const fn adc_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_ptr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x_discard",
        micro_fn: |cpu, _| {
            let _ = cpu.tmp.wrapping_add(cpu.x); // dummy cycle
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16);
            cpu.tmp = bus.read(ptr & 0xFF);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "fetch_hi",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16).wrapping_add(1);
            let hi = bus.read(ptr & 0xFF);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
// 8. (Indirect),Y: ADC ($nn),Y $71 2 bytes, 5+p cycles
// ================================================================
pub const fn adc_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_ptr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.tmp as u16); // base low
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "fetch_hi_add_y",
        micro_fn: |cpu, bus| {
            let hi = bus.read((cpu.tmp as u16).wrapping_add(1));
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.check_cross_page = true;
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "adc_mem",
        micro_fn: |cpu, bus| {
            let mem = bus.read(cpu.effective_addr);
            adc_core(cpu, mem);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base); // dummy read
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::ADC,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
