use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

// pub const fn lax_immediate() -> Instruction {
//     // Cycle 1: opcode already fetched, increment PC to point to immediate value
//     const OP1: MicroOp = MicroOp {
//         name: "inc_pc",
//         micro_fn: |cpu, _| {
//             // The opcode byte has already been fetched.
//             // Move to the next byte which holds the immediate operand.
//             cpu.incr_pc();
//         },
//     };

//     // Cycle 2: read immediate byte, load into A and X
//     const OP2: MicroOp = MicroOp {
//         name: "read_imm_load",
//         micro_fn: |cpu, bus| {
//             // Read the immediate operand from the current PC
//             let data = bus.read(cpu.pc);
//             cpu.data = data;

//             // Perform LAX operation: A = X = data
//             cpu.a = data;
//             cpu.x = data;

//             // Update processor flags (Zero and Negative)
//             cpu.p.set_zn(data);

//             // Advance PC to next instruction
//             cpu.incr_pc();
//         },
//     };

//     Instruction {
//         opcode: Mnemonic::LAX,
//         addressing: Addressing::Immediate,
//         micro_ops: &[OP1, OP2],
//     }
// }

// pub const fn lax_absolute() -> Instruction {
//     // Cycle 1: opcode already fetched, increment PC to point to low address byte
//     const OP1: MicroOp = MicroOp {
//         name: "inc_pc",
//         micro_fn: |cpu, _| {
//             // Opcode was already fetched externally.
//             // Move PC to the low byte of the absolute address.
//             cpu.incr_pc();
//         },
//     };

//     // Cycle 2: fetch low byte of absolute address
//     const OP2: MicroOp = MicroOp {
//         name: "fetch_lo",
//         micro_fn: |cpu, bus| {
//             // Read the low byte of the absolute address
//             let lo = bus.read(cpu.pc);
//             cpu.tmp = lo; // store low byte temporarily
//             cpu.incr_pc(); // advance to next byte
//         },
//     };

//     // Cycle 3: fetch high byte of absolute address
//     const OP3: MicroOp = MicroOp {
//         name: "fetch_hi",
//         micro_fn: |cpu, bus| {
//             // Read the high byte of the absolute address
//             let hi = bus.read(cpu.pc);
//             cpu.effective_addr = ((hi as u16) << 8) | cpu.tmp as u16;
//             cpu.incr_pc(); // PC now points to next instruction
//         },
//     };

//     // Cycle 4: read memory at effective address, perform LAX (A = X = M)
//     const OP4: MicroOp = MicroOp {
//         name: "read_and_lax",
//         micro_fn: |cpu, bus| {
//             // Read the data from the computed effective address
//             let data = bus.read(cpu.effective_addr);
//             cpu.data = data;

//             // Perform LAX operation: load both A and X
//             cpu.a = data;
//             cpu.x = data;

//             // Update Zero and Negative flags
//             cpu.p.set_zn(data);
//         },
//     };

//     Instruction {
//         opcode: Mnemonic::LAX,
//         addressing: Addressing::Absolute,
//         micro_ops: &[OP1, OP2, OP3, OP4],
//     }
// }

// ================================================================
//  1. Immediate: LAX #$nn     $AB    2 bytes, 2 cycles
// ================================================================
pub const fn lax_immediate() -> Instruction {
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| cpu.incr_pc(),
    };
    const OP2: MicroOp = MicroOp {
        name: "fetch_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.pc);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
            cpu.incr_pc();
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::Immediate,
        micro_ops: &[OP1, OP2],
    }
}

// ================================================================
//  2. Zero Page: LAX $nn      $A7    2 bytes, 3 cycles
// ================================================================
pub const fn lax_zero_page() -> Instruction {
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
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.tmp as u16);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  3. Zero Page,Y: LAX $nn,Y  $B7    2 bytes, 4 cycles
// ================================================================
pub const fn lax_zero_page_y() -> Instruction {
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
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::ZeroPageY,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  4. Absolute: LAX $nnnn     $AF    3 bytes, 4 cycles
// ================================================================
pub const fn lax_absolute() -> Instruction {
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
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

// ================================================================
//  5. Absolute,Y: LAX $nnnn,Y $BF    3 bytes, 4(+p) cycles
// ================================================================
pub const fn lax_absolute_y() -> Instruction {
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
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}

// ================================================================
//  6. (Indirect,X): LAX ($nn,X) $A3   2 bytes, 6 cycles
// ================================================================
pub const fn lax_indirect_x() -> Instruction {
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
            // dummy cycle: address calculation
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
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::IndirectX,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}

// ================================================================
//  7. (Indirect),Y: LAX ($nn),Y $B3   2 bytes, 5(+p) cycles
// ================================================================
pub const fn lax_indirect_y() -> Instruction {
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
        name: "read_hi_add_y",
        micro_fn: |cpu, bus| {
            let hi = bus.read((cpu.tmp as u16).wrapping_add(1));
            let base = ((hi as u16) << 8) | (cpu.tmp as u16);
            let addr = base.wrapping_add(cpu.y as u16);
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.check_cross_page = true;
        },
    };
    const OP5: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            if cpu.crossed_page {
                let wrong = (cpu.effective_addr & 0xFF)
                    | ((cpu.effective_addr.wrapping_sub(cpu.y as u16)) & 0xFF00);
                let _ = bus.read(wrong);
            }
        },
    };
    const OP6: MicroOp = MicroOp {
        name: "read_and_lax",
        micro_fn: |cpu, bus| {
            let data = bus.read(cpu.effective_addr);
            cpu.a = data;
            cpu.x = data;
            cpu.p.set_zn(data);
        },
    };
    Instruction {
        opcode: Mnemonic::LAX,
        addressing: Addressing::IndirectY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5, OP6],
    }
}
