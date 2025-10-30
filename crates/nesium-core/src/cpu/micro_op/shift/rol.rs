use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
        status::Status,
    },
};

// ================================================================
//  1. Accumulator: ROL A      $2A    1 byte, 2 cycles
// ================================================================
pub const fn rol_accumulator() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "rol_a",
        micro_fn: |cpu, _| {
            let old_c = cpu.p.contains(Status::CARRY) as u8;
            cpu.p.set_c((cpu.a & 0x80) != 0); // C = old bit 7
            cpu.a = (cpu.a << 1) | old_c; // A = (A << 1) | old_C
            cpu.p.set_zn(cpu.a); // N = result bit7, Z = A == 0
        },
    };
    Instruction {
        opcode: Mnemonic::ROL,
        addressing: Addressing::Accumulator,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: ROL $nn      $26    2 bytes, 5 cycles
// ================================================================
pub const fn rol_zero_page() -> Instruction {
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
        name: "read_old_value",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.tmp as u16); // read old value
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "write_old_value",
        micro_fn: |cpu, bus| {
            bus.write(cpu.tmp as u16 - 0x100, cpu.tmp); // write back old
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "rol_and_write",
        micro_fn: |cpu, bus| {
            let old = cpu.tmp;
            let old_c = cpu.p.contains(Status::CARRY) as u8;
            cpu.p.set_c((old & 0x80) != 0); // C = old bit 7
            let result = (old << 1) | old_c; // rotate left with carry
            cpu.p.set_zn(result); // N = result bit7, Z = result == 0
            bus.write(cpu.tmp as u16 - 0x100, result);
        },
    };
    Instruction {
        opcode: Mnemonic::ROL,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  3. Zero Page,X: ROL $nn,X  $36    2 bytes, 6 cycles
// ================================================================
pub const fn rol_zero_page_x() -> Instruction {
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
        name: "read_old_value",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_old_value",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.tmp);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "rol_and_write",
        micro_fn: |cpu, bus| {
            let old = cpu.tmp;
            let old_c = cpu.p.contains(Status::CARRY) as u8;
            cpu.p.set_c((old & 0x80) != 0);
            let result = (old << 1) | old_c;
            cpu.p.set_zn(result);
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::ROL,
        addressing: Addressing::ZeroPageX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  4. Absolute: ROL $nnnn     $2E    3 bytes, 6 cycles
// ================================================================
pub const fn rol_absolute() -> Instruction {
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
        name: "read_old_value",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_old_value",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.tmp);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "rol_and_write",
        micro_fn: |cpu, bus| {
            let old = cpu.tmp;
            let old_c = cpu.p.contains(Status::CARRY) as u8;
            cpu.p.set_c((old & 0x80) != 0);
            let result = (old << 1) | old_c;
            cpu.p.set_zn(result);
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::ROL,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  5. Absolute,X: ROL $nnnn,X $3E    3 bytes, 7 cycles
// ================================================================
pub const fn rol_absolute_x() -> Instruction {
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
        name: "read_old_value",
        micro_fn: |cpu, bus| {
            cpu.tmp = bus.read(cpu.effective_addr);
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "write_old_value",
        micro_fn: |cpu, bus| {
            bus.write(cpu.effective_addr, cpu.tmp);
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "extra_cycle_if_crossed",
        micro_fn: |cpu, bus| {
            if cpu.check_cross_page && cpu.crossed_page {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let _ = bus.read(base); // dummy read
            }
            cpu.check_cross_page = false;
        },
    };
    const OP7: MicroOp = MicroOp {
        name: "rol_and_write",
        micro_fn: |cpu, bus| {
            let old = cpu.tmp;
            let old_c = cpu.p.contains(Status::CARRY) as u8;
            cpu.p.set_c((old & 0x80) != 0);
            let result = (old << 1) | old_c;
            cpu.p.set_zn(result);
            bus.write(cpu.effective_addr, result);
        },
    };
    Instruction {
        opcode: Mnemonic::ROL,
        addressing: Addressing::AbsoluteX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6, OP7],
    }
}
