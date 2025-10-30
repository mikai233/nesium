use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Immediate: EOR #$nn     $49    2 bytes, 2 cycles
// ================================================================
pub const fn eor_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "eor_imm",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.pc); // fetch immediate value
            cpu.a ^= operand; // A = A XOR operand
            cpu.p.set_zn(cpu.a); // N = bit7, Z = A == 0
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: EOR $nn      $45    2 bytes, 3 cycles
// ================================================================
pub const fn eor_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_zp_addr",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.pc); // fetch ZP address
            cpu.incr_pc();
        },
    };
    const OP3: MicroOp = MicroOp {
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.tmp as u16);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,X: EOR $nn,X  $55    2 bytes, 4 cycles
// ================================================================
pub const fn eor_zero_page_x() -> Instruction {
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: EOR $nnnn     $4D    3 bytes, 4 cycles
// ================================================================
pub const fn eor_absolute() -> Instruction {
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,X: EOR $nnnn,X $5D    3 bytes, 4(+p) cycles
// ================================================================
pub const fn eor_absolute_x() -> Instruction {
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
        name: "fetch_hi_calc_addr",
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
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
        opcode: Mnemonic::EOR,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  6. Absolute,Y: EOR $nnnn,Y $59    3 bytes, 4(+p) cycles
// ================================================================
pub const fn eor_absolute_y() -> Instruction {
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
        name: "fetch_hi_calc_addr",
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base);
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  7. (Indirect,X): EOR ($nn,X) $41   2 bytes, 6 cycles
// ================================================================
pub const fn eor_indirect_x() -> Instruction {
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  8. (Indirect),Y: EOR ($nn),Y $51   2 bytes, 5(+p) cycles
// ================================================================
pub const fn eor_indirect_y() -> Instruction {
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
            cpu.tmp = bus.read(cpu.tmp as u16); // low byte of pointer
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_hi_calc_addr",
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
        name: "eor_mem",
        micro_fn: |cpu, bus| {
            let operand = bus.read(cpu.effective_addr);
            cpu.a ^= operand;
            cpu.p.set_zn(cpu.a);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let _ = bus.read(base);
            }
            cpu.check_cross_page = false;
        },
    };
    Instruction {
        opcode: Mnemonic::EOR,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
