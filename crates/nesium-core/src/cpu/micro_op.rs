use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::{bus::CpuBus, context::Context, cpu::Cpu};

type MicroFn = fn(&mut Cpu, bus: &mut CpuBus, ctx: &mut Context);

#[derive(Clone, Copy, Eq)]
pub struct MicroOp {
    pub(crate) name: &'static str,
    pub(crate) micro_fn: MicroFn,
}

impl MicroOp {
    // ─────────────────────────────────────────────────────────────────────────────
    //  Execution
    // ─────────────────────────────────────────────────────────────────────────────
    /// Execute this micro operation.
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context) {
        (self.micro_fn)(cpu, bus, ctx);
    }

    /// Cycle 2: Read zero-page address byte from PC and increment PC.
    pub(crate) const fn fetch_zp_addr_lo() -> Self {
        MicroOp {
            name: "fetch_zp_addr_lo",
            micro_fn: |cpu, bus, ctx| {
                cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
            },
        }
    }

    /// Cycle 2: Fetch low byte of absolute address from PC and increment PC.
    pub(crate) const fn fetch_abs_addr_lo() -> Self {
        MicroOp {
            name: "fetch_abs_addr_lo",
            micro_fn: |cpu, bus, ctx| {
                cpu.effective_addr = cpu.fetch_u8(bus, ctx) as u16;
            },
        }
    }

    /// Cycle 3: Fetch high byte of absolute address, form full 16-bit address, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi",
            micro_fn: |cpu, bus, ctx| {
                let hi = cpu.fetch_u8(bus, ctx);
                cpu.effective_addr |= (hi as u16) << 8;
            },
        }
    }

    /// Cycle 3: Fetch high byte, add X index, detect page crossing, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi_add_x() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_x",
            micro_fn: |cpu, bus, ctx| {
                let hi = cpu.fetch_u8(bus, ctx);
                let base = ((hi as u16) << 8) | cpu.effective_addr;
                // SHX
                if cpu.opcode_in_flight == Some(0x9C) {
                    cpu.tmp = hi;
                }
                let addr = base.wrapping_add(cpu.x as u16);

                cpu.skip_optional_dummy_read_cycle(base, addr);
                cpu.effective_addr = addr;
            },
        }
    }

    /// Cycle 3: Fetch high byte, add Y index, detect page crossing, and increment PC.
    pub(crate) const fn fetch_abs_addr_hi_add_y() -> Self {
        MicroOp {
            name: "fetch_abs_addr_hi_add_y",
            micro_fn: |cpu, bus, ctx| {
                let hi = cpu.fetch_u8(bus, ctx);
                let base = ((hi as u16) << 8) | cpu.effective_addr;
                // SHA(0x9F) SHX(0x9E) SHS(0x9B)
                if cpu.opcode_in_flight == Some(0x9F)
                    || cpu.opcode_in_flight == Some(0x9E)
                    || cpu.opcode_in_flight == Some(0x9B)
                {
                    cpu.tmp = hi;
                }
                let addr = base.wrapping_add(cpu.y as u16);

                cpu.skip_optional_dummy_read_cycle(base, addr);
                cpu.effective_addr = addr;
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
            micro_fn: |cpu, bus, ctx| {
                cpu.tmp = bus.mem_read(cpu.effective_addr, cpu, ctx);
            },
        }
    }

    /// Cycle 3 (ZeroPage,X): Add X to zero-page address with wrap-around and perform dummy read.
    pub(crate) const fn read_zero_page_add_x_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_x_dummy",
            micro_fn: |cpu, bus, ctx| {
                let addr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
                let _ = bus.mem_read(addr, cpu, ctx); // dummy read for timing
                cpu.effective_addr = addr;
            },
        }
    }

    /// Cycle 3 (ZeroPage,Y): Add Y to zero-page address with wrap-around and perform dummy read.
    pub(crate) const fn read_zero_page_add_y_dummy() -> Self {
        MicroOp {
            name: "read_zero_page_add_y_dummy",
            micro_fn: |cpu, bus, ctx| {
                let addr = (cpu.effective_addr + cpu.y as u16) & 0x00FF;
                let _ = bus.mem_read(addr, cpu, ctx); // dummy read for timing
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
            micro_fn: |cpu, bus, ctx| {
                let ptr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
                let _ = bus.mem_read(ptr, cpu, ctx); // dummy read for timing
            },
        }
    }

    /// Cycle 4 (Indirect,X): Read low byte of effective address from `(zp + X) & 0xFF`.
    pub(crate) const fn read_indirect_x_lo() -> Self {
        MicroOp {
            name: "read_indirect_x_lo",
            micro_fn: |cpu, bus, ctx| {
                let ptr = (cpu.effective_addr + cpu.x as u16) & 0x00FF;
                cpu.tmp = bus.mem_read(ptr, cpu, ctx);
            },
        }
    }

    /// Cycle 5 (Indirect,X): Read high byte from `(zp + X + 1) & 0xFF` and form address.
    pub(crate) const fn read_indirect_x_hi() -> Self {
        MicroOp {
            name: "read_indirect_x_hi",
            micro_fn: |cpu, bus, ctx| {
                let ptr = (cpu.effective_addr + cpu.x as u16 + 1) & 0x00FF;
                let hi = bus.mem_read(ptr, cpu, ctx);
                cpu.effective_addr = ((hi as u16) << 8) | cpu.tmp as u16;
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
            micro_fn: |cpu, bus, ctx| {
                let hi_addr = (cpu.effective_addr + 1) & 0x00FF;
                let hi = bus.mem_read(hi_addr, cpu, ctx);
                let base = ((hi as u16) << 8) | (cpu.tmp as u16);
                // SHA
                if cpu.opcode_in_flight == Some(0x93) {
                    cpu.tmp = hi;
                }
                let addr = base.wrapping_add(cpu.y as u16);

                cpu.skip_optional_dummy_read_cycle(base, addr);
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
            micro_fn: |cpu, bus, ctx| {
                cpu.tmp = bus.mem_read(cpu.effective_addr, cpu, ctx);
            },
        }
    }

    /// Cycle 5 (JMP Indirect): Read high byte of target address.
    /// **6502 bug**: if pointer ends in `$FF`, high byte is read from `$xx00` of the same page.
    pub(crate) const fn read_indirect_hi_buggy() -> Self {
        MicroOp {
            name: "read_indirect_hi_buggy",
            micro_fn: |cpu, bus, ctx| {
                let hi_addr = if (cpu.effective_addr & 0xFF) == 0xFF {
                    cpu.effective_addr & 0xFF00
                } else {
                    cpu.effective_addr + 1
                };
                let hi = bus.mem_read(hi_addr, cpu, ctx);
                cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
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
            micro_fn: |cpu, bus, ctx| {
                let base = cpu.effective_addr.wrapping_sub(cpu.x as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.mem_read(dummy_addr, cpu, ctx); // dummy read for timing
            },
        }
    }

    /// Dummy read for Absolute,Y or (Indirect),Y when a page boundary is crossed.
    pub(crate) const fn dummy_read_cross_y() -> Self {
        MicroOp {
            name: "dummy_read_cross_y",
            micro_fn: |cpu, bus, ctx| {
                let base = cpu.effective_addr.wrapping_sub(cpu.y as u16);
                let dummy_addr = (base & 0xFF00) | (cpu.effective_addr & 0x00FF);
                let _ = bus.mem_read(dummy_addr, cpu, ctx); // dummy read for timing
            },
        }
    }
}

impl Debug for MicroOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#[{}.{:?}]", self.name, self.micro_fn)
    }
}

impl PartialEq for MicroOp {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && std::ptr::fn_addr_eq(self.micro_fn, other.micro_fn)
    }
}

impl Hash for MicroOp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.micro_fn.hash(state);
    }
}

pub(crate) fn empty_micro_fn(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context) {
    cpu.dummy_read(bus, ctx);
}
