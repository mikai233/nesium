use std::fmt::Debug;

use crate::{
    bus::{Bus, BusImpl},
    cpu::Cpu,
};

type MicroFn = fn(&mut Cpu, bus: &mut BusImpl);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MicroOp {
    pub(crate) name: &'static str,
    pub(crate) micro_fn: MicroFn,
}

impl MicroOp {
    // ─────────────────────────────────────────────────────────────────────────────
    //  Execution
    // ─────────────────────────────────────────────────────────────────────────────
    /// Execute this micro operation.
    #[cfg(not(test))]
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut BusImpl) {
        (self.micro_fn)(cpu, bus);
    }

    #[cfg(test)]
    #[tracing::instrument(skip(bus))]
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut BusImpl) {
        (self.micro_fn)(cpu, bus);
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Fetch & Program Counter Operations
    // ─────────────────────────────────────────────────────────────────────────────
    /// Cycle 1: Advance PC after fetching the opcode.
    pub(crate) const fn advance_pc_after_opcode() -> Self {
        MicroOp {
            name: "advance_pc_after_opcode",
            micro_fn: |cpu, _| cpu.incr_pc(),
        }
    }

    /// Cycle 2: Read zero-page address byte from PC and increment PC.
    pub(crate) const fn fetch_zp_addr_lo() -> Self {
        MicroOp {
            name: "fetch_zp_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.zp_addr = bus.read(cpu.pc);
                #[cfg(test)]
                tracing::trace!("zp_addr: 0x{:02X}", cpu.zp_addr);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 2: Read relative branch offset (signed 8-bit) from PC and increment PC.
    pub(crate) const fn fetch_rel_offset() -> Self {
        MicroOp {
            name: "fetch_rel_offset",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 2: Fetch low byte of absolute address from PC and increment PC.
    pub(crate) const fn fetch_abs_addr_lo() -> Self {
        MicroOp {
            name: "fetch_abs_addr_lo",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.pc);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address, form full 16-bit address, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                cpu.effective_addr = ((hi as u16) << 8) | (cpu.base as u16);
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte, add X index, detect page crossing, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi_add_x() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_x",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | (cpu.base as u16);
                cpu.base = hi; // Store high byte for some unofficial instructions
                let addr = base.wrapping_add(cpu.x as u16);

                cpu.check_cross_page(base, addr);
                cpu.effective_addr = addr;
                cpu.incr_pc();
            },
        }
    }

    /// Cycle 3: Fetch high byte, add Y index, detect page crossing, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi_add_y() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_y",
            micro_fn: |cpu, bus| {
                let hi = bus.read(cpu.pc);
                let base = ((hi as u16) << 8) | (cpu.base as u16);
                cpu.base = hi; // Store high byte for some unofficial instructions
                let addr = base.wrapping_add(cpu.y as u16);

                cpu.check_cross_page(base, addr);
                cpu.effective_addr = addr;
                cpu.incr_pc();
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Zero-Page & Indexed Zero-Page Operations
    // ─────────────────────────────────────────────────────────────────────────────
    /// Read byte from zero-page address (`$nn`).
    pub(crate) const fn read_zero_page() -> Self {
        MicroOp {
            name: "read_zero_page",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.zp_addr as u16);
            },
        }
    }

    /// Cycle 3 (ZeroPage,X): Add X to zero-page address with wrap-around and perform dummy read.
    pub(crate) const fn read_zero_page_add_x_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_x_dummy",
            micro_fn: |cpu, bus| {
                let addr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                let _ = bus.read(addr); // dummy read for timing
                cpu.effective_addr = addr;
            },
        }
    }

    /// Cycle 3 (ZeroPage,Y): Add Y to zero-page address with wrap-around and perform dummy read.
    pub(crate) const fn read_zero_page_add_y_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_y_dummy",
            micro_fn: |cpu, bus| {
                let addr = (cpu.zp_addr as u16 + cpu.y as u16) & 0x00FF;
                let _ = bus.read(addr); // dummy read for timing
                cpu.effective_addr = addr;
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Indirect,X  (pre-indexed indirect)
    // ─────────────────────────────────────────────────────────────────────────────
    /// Cycle 3 (Indirect,X): Dummy read at `(zp + X) & 0xFF`.
    pub(crate) const fn read_indirect_x_dummy() -> Self {
        MicroOp {
            name: "read_indirect_x_dummy",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                let _ = bus.read(ptr); // dummy read for timing
            },
        }
    }

    /// Cycle 4 (Indirect,X): Read low byte of effective address from `(zp + X) & 0xFF`.
    pub(crate) const fn read_indirect_x_lo() -> Self {
        MicroOp {
            name: "read_indirect_x_lo",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16) & 0x00FF;
                cpu.base = bus.read(ptr);
            },
        }
    }

    /// Cycle 5 (Indirect,X): Read high byte from `(zp + X + 1) & 0xFF` and form address.
    pub(crate) const fn read_indirect_x_hi() -> Self {
        MicroOp {
            name: "read_indirect_x_hi",
            micro_fn: |cpu, bus| {
                let ptr = (cpu.zp_addr as u16 + cpu.x as u16 + 1) & 0x00FF;
                let hi = bus.read(ptr);
                cpu.effective_addr = ((hi as u16) << 8) | (cpu.base as u16);
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Indirect,Y  (post-indexed indirect)
    // ─────────────────────────────────────────────────────────────────────────────
    /// Cycle 4 (Indirect),Y: Read high byte from `(zp + 1) & 0xFF`, add Y, detect page crossing.
    pub(crate) const fn read_indirect_y_hi() -> Self {
        MicroOp {
            name: "read_indirect_y_hi",
            micro_fn: |cpu, bus| {
                let hi_addr = (cpu.zp_addr as u16 + 1) & 0x00FF;
                let hi = bus.read(hi_addr);
                let base = ((hi as u16) << 8) | (cpu.base as u16);
                // Store high byte for some unofficial instructions
                cpu.base = hi;
                let addr = base.wrapping_add(cpu.y as u16);

                cpu.check_cross_page(base, addr);
                cpu.effective_addr = addr;
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Absolute Indirect (JMP only) – with 6502 page-boundary bug
    // ─────────────────────────────────────────────────────────────────────────────
    /// Cycle 4 (JMP Indirect): Read low byte of target address from pointer.
    pub(crate) const fn read_indirect_lo() -> Self {
        MicroOp {
            name: "read_indirect_lo",
            micro_fn: |cpu, bus| {
                cpu.base = bus.read(cpu.effective_addr);
            },
        }
    }

    /// Cycle 5 (JMP Indirect): Read high byte of target address.
    /// **6502 bug**: if pointer ends in `$FF`, high byte is read from `$xx00` of the same page.
    pub(crate) const fn read_indirect_hi_buggy() -> Self {
        MicroOp {
            name: "read_indirect_hi_buggy",
            micro_fn: |cpu, bus| {
                let hi_addr = if (cpu.effective_addr & 0xFF) == 0xFF {
                    cpu.effective_addr & 0xFF00
                } else {
                    cpu.effective_addr + 1
                };
                let hi = bus.read(hi_addr);
                cpu.effective_addr = ((hi as u16) << 8) | (cpu.base as u16);
            },
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    //  Page-Crossing Dummy Reads
    // ─────────────────────────────────────────────────────────────────────────────
    /// Dummy read for Absolute,X when a page boundary is crossed.
    pub(crate) const fn dummy_read_cross_x() -> Self {
        MicroOp {
            name: "dummy_read_cross_x",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr); // dummy read for timing
            },
        }
    }

    /// Dummy read for Absolute,Y or (Indirect),Y when a page boundary is crossed.
    pub(crate) const fn dummy_read_cross_y() -> Self {
        MicroOp {
            name: "dummy_read_cross_y",
            micro_fn: |cpu, bus| {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.read(dummy_addr); // dummy read for timing
            },
        }
    }
}

impl Debug for MicroOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#[{}.{:?}]", self.name, self.micro_fn)
    }
}

pub(crate) fn empty_micro_fn(_: &mut Cpu, _: &mut BusImpl) {}
