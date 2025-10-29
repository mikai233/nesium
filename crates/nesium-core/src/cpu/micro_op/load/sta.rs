use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Absolute: STA $nnnn     $8D    3 bytes, 4 cycles
// ================================================================
pub const fn sta_absolute() -> Instruction {
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
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  2. Absolute,X: STA $nnnn,X $9D    3 bytes, 5 cycles
// ================================================================
pub const fn sta_absolute_x() -> Instruction {
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
            cpu.effective_addr = base.wrapping_add(cpu.x as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_base",
        micro_fn: |cpu, bus| {
            // Dummy read from base (without X) to consume the extra cycle
            let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
            let _ = bus.read(base);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  3. Absolute,Y: STA $nnnn,Y $99    3 bytes, 5 cycles
// ================================================================
pub const fn sta_absolute_y() -> Instruction {
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
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);
            cpu.incr_pc();
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_base",
        micro_fn: |cpu, bus| {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let _ = bus.read(base);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  4. Zero Page: STA $nn      $85    2 bytes, 3 cycles
// ================================================================
pub const fn sta_zero_page() -> Instruction {
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
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.tmp as u16, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  5. Zero Page,X: STA $nn,X  $95    2 bytes, 4 cycles
// ================================================================
pub const fn sta_zero_page_x() -> Instruction {
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
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  6. (Indirect,X): STA ($nn,X) $81   2 bytes, 6 cycles
// ================================================================
pub const fn sta_indirect_x() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "add_x_dummy",
        micro_fn: |cpu, _| {
            let _ = cpu.tmp.wrapping_add(cpu.x);
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_lo",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16);
            cpu.tmp = bus.read(ptr);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "read_hi",
        micro_fn: |cpu, bus| {
            let ptr = (cpu.tmp as u16).wrapping_add(cpu.x as u16).wrapping_add(1);
            let hi = bus.read(ptr);
            cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  7. (Indirect),Y: STA ($nn),Y $91   2 bytes, 6 cycles
// ================================================================
pub const fn sta_indirect_y() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc);
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "read_lo",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.tmp as u16);
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_hi",
        micro_fn: |cpu, bus| {
            let hi = bus.read((cpu.tmp as u16).wrapping_add(1));
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            cpu.effective_addr = base.wrapping_add(cpu.y as u16);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "dummy_read_base",
        micro_fn: |cpu, bus| {
            let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
            let _ = bus.read(base);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "write_a",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::STA,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
