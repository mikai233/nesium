use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// ================================================================
//  1. Immediate: LDX #$nn     $A2    2 bytes, 2 cycles
// ================================================================
pub const fn ldx_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.pc);
            cpu.x = data;
            cpu.p.set_zn(data);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: LDX $nn      $A6    2 bytes, 3 cycles
// ================================================================
pub const fn ldx_zero_page() -> Instruction {
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
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.tmp as u16);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,Y: LDX $nn,Y  $B6    2 bytes, 4 cycles
// ================================================================
pub const fn ldx_zero_page_y() -> Instruction {
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
        name: "add_y",
        micro_fn: |cpu, _| {
            cpu.effective_addr = (cpu.tmp as u16).wrapping_add(cpu.y as u16);
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: LDX $nnnn     $AE    3 bytes, 4 cycles
// ================================================================
pub const fn ldx_absolute() -> Instruction {
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
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,Y: LDX $nnnn,Y $BE    3 bytes, 4(+p) cycles
// ================================================================
pub const fn ldx_absolute_y() -> Instruction {
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
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();
            cpu.check_cross_page = true;
        },
    };
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            if cpu.crossed_page {
                let wrong = (cpu.effective_addr & 0xFF)
                    | ((cpu.effective_addr.wrapping_sub(cpu.y as u16)) & 0xFF00);
                let _ = bus.read(wrong);
            }
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "read_and_ldx",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LDX,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}
