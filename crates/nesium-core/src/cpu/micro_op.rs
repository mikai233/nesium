use crate::{
    bus::{Bus, BusImpl},
    cpu::Cpu,
};

pub mod arith;
pub mod bra;
pub mod ctrl;
pub mod flags;
pub mod inc;
pub mod kill;
pub mod load;
pub mod logic;
pub mod nop;
pub mod shift;
pub mod stack;
pub mod trans;

type MicroFn = fn(&mut Cpu, bus: &mut BusImpl);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MicroOp {
    pub(crate) name: &'static str,
    pub(crate) micro_fn: MicroFn,
}

impl MicroOp {
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut BusImpl) {
        (self.micro_fn)(cpu, bus)
    }

    // ───────────────────────────────────────────────
    //  Fetch & Program Counter Operations
    // ───────────────────────────────────────────────

    /// Cycle 1: The opcode has already been fetched externally.
    /// We simply advance the program counter to prepare for the next byte.
    pub(crate) const fn advance_pc_after_opcode() -> Self {
        MicroOp {
            name: "advance_pc_after_opcode",
            micro_fn: |cpu, _| cpu.incr_pc(),
        }
    }

    /// Cycle 2: Read zero-page address from PC, then increment PC.
    pub(crate) const fn fetch_zp_addr_lo() -> Self {
        MicroOp {
            name: "fetch_zp_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.zp_addr = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 2: Fetch low byte of absolute address from PC.
    pub(crate) const fn fetch_abs_addr_lo() -> Self {
        MicroOp {
            name: "fetch_abs_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.base_lo = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address from PC and form full address.
    pub(crate) const fn fetch_abs_addr_hi() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                cpu.effective_addr = ((hi as u16) << 8) | cpu.base_lo as u16;
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address, add X index, check page crossing.
    pub(crate) const fn fetch_abs_addr_hi_add_x() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_x",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.x as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.incr_pc();
                cpu.check_cross_page = true;
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address, add Y index, check page crossing.
    pub(crate) const fn fetch_abs_addr_hi_add_y() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_y",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.y as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.incr_pc();
                cpu.check_cross_page = true;
            },
        }
    }

    // ───────────────────────────────────────────────
    //  Zero Page & Indirect Operations
    // ───────────────────────────────────────────────

    pub(crate) const fn read_indirect_x_dummy() -> Self {
        MicroOp {
            name: "read_indirect_x_dummy",
            micro_fn: |cpu, bus| {
                // Cycle 3 of (Indirect,X):
                // Calculate ($nn + X) with zero-page wrap, perform dummy read
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                let _ = bus.read(ptr); // Timing dummy read (no result used)
            },
        }
    }

    /// Cycle 4: Read low byte of address from ($nn + X), with zero-page wrap-around.
    pub(crate) const fn read_indirect_x_lo() -> Self {
        MicroOp {
            name: "read_indirect_x_lo",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                cpu.base_lo = bus.read(ptr);
            },
        }
    }

    /// Cycle 5: Read high byte of address from ($nn + X + 1), with zero-page wrap.
    pub(crate) const fn read_indirect_x_hi() -> Self {
        MicroOp {
            name: "read_indirect_x_hi",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16 + 1) & 0x00FF;
                let hi = bus.read(ptr);
                cpu.effective_addr = ((hi as u16) << 8) | cpu.base_lo as u16;
            },
        }
    }

    /// Read a byte from zero-page address ($nn).
    pub(crate) const fn read_zero_page() -> Self {
        MicroOp {
            name: "read_zero_page",
            micro_fn: |cpu, bus| {
                cpu.base_lo = bus.read(cpu.zp_addr as u16);
            },
        }
    }

    /// Indexed Indirect (Indirect),Y addressing:
    /// Read high byte from ($nn + 1), add Y, detect page crossing.
    pub(crate) const fn read_indirect_y_hi() -> Self {
        MicroOp {
            name: "read_indirect_y_hi",
            micro_fn: |cpu, bus| {
                let hi_addr = (cpu.zp_addr as u16 + 1) & 0x00FF;
                let hi = bus.read(hi_addr);
                let base = ((hi as u16) << 8) | cpu.base_lo as u16;
                let addr = base.wrapping_add(cpu.y as u16);
                cpu.crossed_page = (base & 0xFF00) != (addr & 0xFF00);
                cpu.effective_addr = addr;
                cpu.check_cross_page = true;
            },
        }
    }

    /// Cycle 3: Zero Page,Y addressing - add Y index with zero-page wrap-around.
    pub(crate) const fn read_zero_page_add_y_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_y_dummy",
            micro_fn: |cpu, bus| {
                let addr = (cpu.zp_addr as u16 + cpu.y as u16) & 0x00FF;
                let _ = bus.read(addr); // dummy read (for timing)
                cpu.effective_addr = addr;
            },
        }
    }

    pub(crate) const fn dummy_read_cross_x() -> Self {
        MicroOp {
            name: "dummy_read_cross_x",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr);
            },
        }
    }

    pub(crate) const fn dummy_read_cross_y() -> Self {
        MicroOp {
            name: "dummy_read_cross_y",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr);
            },
        }
    }
}
