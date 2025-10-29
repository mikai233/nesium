use crate::{
    bus::Bus,
    cpu::{
        addressing::Addressing,
        instruction::{Instruction, Mnemonic},
        micro_op::MicroOp,
    },
};

pub const fn las_absolute_y() -> Instruction {
    // Cycle 1: opcode already fetched, increment PC to point to low byte
    const OP1: MicroOp = MicroOp {
        name: "inc_pc",
        micro_fn: |cpu, _| {
            // The opcode has already been fetched externally.
            // We simply advance the program counter to fetch the low byte next.
            cpu.incr_pc();
        },
    };

    // Cycle 2: fetch low byte of address, increment PC
    const OP2: MicroOp = MicroOp {
        name: "fetch_lo",
        micro_fn: |cpu, bus| {
            // Read the low byte of the address operand from memory
            let lo = bus.read(cpu.pc);
            cpu.tmp = lo; // temporarily store low byte
            cpu.incr_pc(); // advance to high byte
        },
    };

    // Cycle 3: fetch high byte of address, add Y, check for page cross
    const OP3: MicroOp = MicroOp {
        name: "fetch_hi_add_y",
        micro_fn: |cpu, bus| {
            // Read high byte of base address
            let hi = bus.read(cpu.pc);
            // Combine low and high to form the base address
            let base = ((hi as u16) << 8) | cpu.tmp as u16;
            // Add Y register to the address to form effective address
            let addr = base.wrapping_add(cpu.y as u16);

            // Determine if page boundary was crossed
            cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
            cpu.effective_addr = addr;
            cpu.incr_pc();

            // Enable skip logic in case the next cycle (dummy read) is unnecessary
            cpu.check_cross_page = true;
        },
    };

    // Cycle 4: dummy read (only if page boundary was crossed)
    const OP4: MicroOp = MicroOp {
        name: "dummy_read_cross",
        micro_fn: |cpu, bus| {
            // When the address crosses a page boundary, the 6502 performs
            // a dummy read from the wrong page before correcting the high byte.
            let addr = (cpu.effective_addr & 0xFF)
                | ((cpu.effective_addr.wrapping_sub(cpu.y as u16)) & 0xFF00);
            let _ = bus.read(addr); // dummy read, result discarded
        },
    };

    // Cycle 5: read final byte from effective address and execute LAS operation
    const OP5: MicroOp = MicroOp {
        name: "read_and_las",
        micro_fn: |cpu, bus| {
            // Read the actual data from the computed effective address
            let data = bus.read(cpu.effective_addr);

            // Perform LAS operation: A, X, S = M & S
            let result = data & cpu.s;
            cpu.a = result;
            cpu.x = result;
            cpu.s = result;

            // Update processor status flags (Z, N)
            cpu.p.set_zn(result);
        },
    };

    Instruction {
        opcode: Mnemonic::LAS,
        addressing: Addressing::AbsoluteY,
        micro_ops: &[OP1, OP2, OP3, OP4, OP5],
    }
}
