use std::fmt::Display;

use crate::{
    bus::Bus,
    cpu::{
        CPU,
        micro_op::{MicroOp, MicroOp2},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Addressing {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    XIndexedAbsolute,
    YIndexedAbsolute,
    AbsoluteIndirect,
    ZeroPage,
    XIndexedZeroPage,
    YIndexedZeroPage,
    XIndexedZeroPageIndirect,
    ZeroPageIndirectYIndexed,
    Relative,
}

impl Addressing {
    pub(crate) const fn micro_ops2(&self) -> &'static [MicroOp2] {
        match self {
            // ---------------------------------------------------------------------
            // Implied
            // ---------------------------------------------------------------------
            Addressing::Implied => Self::implied(),

            // ---------------------------------------------------------------------
            // Accumulator
            // ---------------------------------------------------------------------
            Addressing::Accumulator => Self::accumulator(),

            // ---------------------------------------------------------------------
            // Immediate (#$xx)
            // ---------------------------------------------------------------------
            Addressing::Immediate => Self::immediate(),

            // ---------------------------------------------------------------------
            // Absolute ($HHLL)
            // ---------------------------------------------------------------------
            Addressing::Absolute => Self::absolute(),

            // ---------------------------------------------------------------------
            // Absolute,X ($HHLL,X)
            // ---------------------------------------------------------------------
            Addressing::XIndexedAbsolute => Self::absx(),

            // ---------------------------------------------------------------------
            // Absolute,Y ($HHLL,Y)
            // ---------------------------------------------------------------------
            Addressing::YIndexedAbsolute => Self::absy(),

            // ---------------------------------------------------------------------
            // Absolute Indirect ($HHLL)
            // Used by JMP only.
            // ---------------------------------------------------------------------
            Addressing::AbsoluteIndirect => Self::absind(),

            // ---------------------------------------------------------------------
            // Zero Page ($LL)
            // ---------------------------------------------------------------------
            Addressing::ZeroPage => Self::zp(),

            // ---------------------------------------------------------------------
            // Zero Page,X
            // ---------------------------------------------------------------------
            Addressing::XIndexedZeroPage => Self::zpx(),

            // ---------------------------------------------------------------------
            // Zero Page,Y
            // ---------------------------------------------------------------------
            Addressing::YIndexedZeroPage => Self::zpy(),

            // ---------------------------------------------------------------------
            // (Indirect,X)
            // ---------------------------------------------------------------------
            Addressing::XIndexedZeroPageIndirect => Self::indx(),

            // ---------------------------------------------------------------------
            // (Indirect),Y
            // ---------------------------------------------------------------------
            Addressing::ZeroPageIndirectYIndexed => &[
                MicroOp2 {
                    name: "indy_cycle1",
                    op: |cpu, _| cpu.incr_pc(),
                },
                MicroOp2 {
                    name: "indy_cycle2",
                    op: |cpu, bus| {
                        let zp = bus.read(cpu.pc);
                        cpu.incr_pc();
                        let lo = bus.read(zp as u16);
                        let hi = bus.read(zp.wrapping_add(1) as u16);
                        let base = ((hi as u16) << 8) | (lo as u16);
                        let eff = base.wrapping_add(cpu.y as u16);
                        cpu.crossed_page = (base & 0xFF00) != (eff & 0xFF00);
                        cpu.effective_addr = eff;
                    },
                },
                MicroOp2 {
                    name: "indy_cycle3",
                    op: |cpu, bus| {
                        if cpu.crossed_page {
                            let _ = bus.read(
                                (cpu.effective_addr & 0xFF00)
                                    | ((cpu.effective_addr.wrapping_sub(0x100)) & 0x00FF),
                            );
                        }
                    },
                },
            ],

            // ---------------------------------------------------------------------
            // Relative (for branches)
            // ---------------------------------------------------------------------
            Addressing::Relative => &[
                MicroOp2 {
                    name: "rel_cycle1",
                    op: |cpu, _| cpu.incr_pc(),
                },
                MicroOp2 {
                    name: "rel_cycle2",
                    op: |cpu, bus| {
                        let offset = bus.read(cpu.pc) as i8;
                        cpu.incr_pc();
                        let target = cpu.pc.wrapping_add(offset as u16);
                        cpu.effective_addr = target;
                    },
                },
            ],
        }
    }

    const fn implied() -> &'static [MicroOp2] {
        &[MicroOp2 {
            name: "implied_cycle1",
            op: |cpu, _| {
                cpu.incr_pc();
            },
        }]
    }

    const fn accumulator() -> &'static [MicroOp2] {
        &[MicroOp2 {
            name: "accumulator_cycle1",
            op: |cpu, _| {
                cpu.incr_pc();
            },
        }]
    }

    const fn immediate() -> &'static [MicroOp2] {
        &[MicroOp2 {
            name: "immediate_cycle1",
            op: |cpu, _| {
                cpu.incr_pc();
            },
        }]
    }

    const fn absolute() -> &'static [MicroOp2] {
        &[
            MicroOp2 {
                name: "absolute_cycle1",
                op: |cpu, _| {
                    cpu.incr_pc();
                },
            },
            MicroOp2 {
                name: "absolute_cycle2",
                op: |cpu, bus| {
                    let lo = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = lo;
                },
            },
            MicroOp2 {
                name: "absolute_cycle3",
                op: |cpu, bus| {
                    let hi = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
                },
            },
        ]
    }

    const fn absx() -> &'static [MicroOp2] {
        &[
            MicroOp2 {
                name: "absx_cycle1",
                op: |cpu, _| cpu.incr_pc(),
            },
            MicroOp2 {
                name: "absx_cycle2",
                op: |cpu, bus| {
                    let lo = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = lo;
                },
            },
            MicroOp2 {
                name: "absx_cycle3",
                op: |cpu, bus| {
                    let hi = bus.read(cpu.pc);
                    cpu.incr_pc();
                    let base = ((hi as u16) << 8) | (cpu.tmp as u16);
                    let eff = base.wrapping_add(cpu.x as u16);
                    cpu.crossed_page = (base & 0xFF00) != (eff & 0xFF00);
                    cpu.effective_addr = eff;
                },
            },
            MicroOp2 {
                name: "absx_cycle4",
                op: |cpu, bus| {
                    if cpu.crossed_page {
                        // Dummy read if crossed page
                        let _ = bus.read(
                            (cpu.effective_addr & 0xFF00)
                                | ((cpu.effective_addr.wrapping_sub(0x100)) & 0x00FF),
                        );
                    }
                },
            },
        ]
    }

    const fn absy() -> &'static [MicroOp2] {
        &[
            MicroOp2 {
                name: "absy_cycle1",
                op: |cpu, _| cpu.incr_pc(),
            },
            MicroOp2 {
                name: "absy_cycle2",
                op: |cpu, bus| {
                    let lo = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = lo;
                },
            },
            MicroOp2 {
                name: "absy_cycle3",
                op: |cpu, bus| {
                    let hi = bus.read(cpu.pc);
                    cpu.incr_pc();
                    let base = ((hi as u16) << 8) | (cpu.tmp as u16);
                    let eff = base.wrapping_add(cpu.y as u16);
                    cpu.crossed_page = (base & 0xFF00) != (eff & 0xFF00);
                    cpu.effective_addr = eff;
                },
            },
            MicroOp2 {
                name: "absy_cycle4",
                op: |cpu, bus| {
                    if cpu.crossed_page {
                        let _ = bus.read(
                            (cpu.effective_addr & 0xFF00)
                                | ((cpu.effective_addr.wrapping_sub(0x100)) & 0x00FF),
                        );
                    }
                },
            },
        ]
    }

    const fn absind() -> &'static [MicroOp2] {
        &[
            // --------------------------------------------------------------
            // Cycle 1: Fetch opcode ($6C), PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "absind_cycle1_fetch_opcode",
                op: |cpu, _bus| {
                    cpu.incr_pc(); // PC now points to first operand byte
                },
            },
            // --------------------------------------------------------------
            // Cycle 2: Fetch low byte of indirect address → store in tmp, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "absind_cycle2_fetch_ptr_lo",
                op: |cpu, bus| {
                    let lo = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = lo; // Save low byte of pointer address
                },
            },
            // --------------------------------------------------------------
            // Cycle 3: Fetch high byte of indirect address, form full ptr, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "absind_cycle3_fetch_ptr_hi",
                op: |cpu, bus| {
                    let hi = bus.read(cpu.pc);
                    cpu.incr_pc();
                    // Build full 16-bit indirect address: $xxFF
                    cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
                },
            },
            // --------------------------------------------------------------
            // Cycle 4: Read low byte of jump target from [ptr]
            // --------------------------------------------------------------
            MicroOp2 {
                name: "absind_cycle4_read_target_lo",
                op: |cpu, bus| {
                    let ptr = cpu.effective_addr;
                    let lo = bus.read(ptr);
                    cpu.tmp = lo; // Reuse tmp to hold target low byte
                },
            },
            // --------------------------------------------------------------
            // Cycle 5: Read high byte of jump target (with 6502 page-wrap bug!)
            //         Then set PC to final jump address
            // --------------------------------------------------------------
            MicroOp2 {
                name: "absind_cycle5_read_target_hi_with_bug",
                op: |cpu, bus| {
                    let ptr = cpu.effective_addr;

                    // 6502 bug: if low byte of ptr is $FF, high byte is read
                    // from $xx00 instead of $xx+1 (no carry)
                    let hi_addr = if (ptr & 0x00FF) == 0x00FF {
                        ptr & 0xFF00 // Stay on same page (bug)
                    } else {
                        ptr.wrapping_add(1) // Normal increment
                    };

                    let hi = bus.read(hi_addr);
                    cpu.effective_addr = ((hi as u16) << 8) | (cpu.tmp as u16);
                },
            },
        ]
    }

    const fn zp() -> &'static [MicroOp2] {
        &[
            MicroOp2 {
                name: "zp_cycle1",
                op: |cpu, _| cpu.incr_pc(),
            },
            MicroOp2 {
                name: "zp_cycle2",
                op: |cpu, bus| {
                    let addr = bus.read(cpu.pc) as u16;
                    cpu.incr_pc();
                    cpu.effective_addr = addr;
                },
            },
        ]
    }

    const fn zpx() -> &'static [MicroOp2] {
        &[
            // --------------------------------------------------------------
            // Cycle 1: Fetch opcode, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpx_cycle1_fetch_opcode",
                op: |cpu, _| {
                    cpu.incr_pc(); // PC now points to base address
                },
            },
            // --------------------------------------------------------------
            // Cycle 2: Fetch base zero-page address → store in tmp, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpx_cycle2_fetch_base",
                op: |cpu, bus| {
                    let base = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = base; // Save base address in tmp
                },
            },
            // --------------------------------------------------------------
            // Cycle 3: Compute effective address = (tmp + X) & 0xFF
            //          No memory access - internal operation only!
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpx_cycle3_compute_address",
                op: |cpu, bus| {
                    let base = cpu.tmp as u16;
                    // dummy read
                    let _ = bus.read(base);
                    let offset = cpu.x as u16;
                    cpu.effective_addr = (base + offset) & 0xFF; // Wrap within zero page
                },
            },
        ]
    }

    const fn zpy() -> &'static [MicroOp2] {
        &[
            // --------------------------------------------------------------
            // Cycle 1: Fetch opcode, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpy_cycle1_fetch_opcode",
                op: |cpu, _| {
                    cpu.incr_pc(); // PC now points to base address
                },
            },
            // --------------------------------------------------------------
            // Cycle 2: Fetch base zero-page address → store in tmp, PC++
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpy_cycle2_fetch_base",
                op: |cpu, bus| {
                    let base = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = base; // Save base ZP address
                },
            },
            // --------------------------------------------------------------
            // Cycle 3: Compute effective address = (tmp + Y) & 0xFF
            //          No memory access - internal only!
            // --------------------------------------------------------------
            MicroOp2 {
                name: "zpy_cycle3_compute_address",
                op: |cpu, bus| {
                    let base = cpu.tmp as u16;
                    // dummy read
                    let _ = bus.read(base);
                    let offset = cpu.y as u16;
                    cpu.effective_addr = (base + offset) & 0xFF; // Wrap within zero page
                },
            },
        ]
    }

    const fn indx() -> &'static [MicroOp2] {
        &[
            // --------------------------------------------------------------
            // C1: Fetch opcode, advance PC
            // --------------------------------------------------------------
            MicroOp2 {
                name: "c1_fetch_op",
                op: |cpu, _| cpu.incr_pc(),
            },
            // --------------------------------------------------------------
            // C2: Fetch zero-page base address → tmp
            // --------------------------------------------------------------
            MicroOp2 {
                name: "c2_fetch_zp_base",
                op: |cpu, bus| {
                    let base = bus.read(cpu.pc);
                    cpu.incr_pc();
                    cpu.tmp = base; // tmp = ZP base (e.g., $34)
                },
            },
            // --------------------------------------------------------------
            // C3: Compute ZP pointer = (base + X) & $FF + dummy read
            // --------------------------------------------------------------
            MicroOp2 {
                name: "c3_calc_zp_ptr_dummy",
                op: |cpu, bus| {
                    let _ = bus.read(cpu.tmp as u16); // Dummy read (critical!)
                    let ptr = cpu.tmp.wrapping_add(cpu.x); // Wrap in ZP
                    cpu.tmp = ptr; // tmp = final ZP pointer
                },
            },
            // --------------------------------------------------------------
            // C4: Read low byte of effective address from [ptr]
            // --------------------------------------------------------------
            MicroOp2 {
                name: "c4_read_addr_lo",
                op: |cpu, bus| {
                    let lo = bus.read(cpu.tmp as u16);
                    cpu.effective_addr = lo as u16; // Store low byte
                },
            },
            // --------------------------------------------------------------
            // C5: Read high byte from [ptr+1], form full address
            // --------------------------------------------------------------
            MicroOp2 {
                name: "c5_read_addr_hi_form",
                op: |cpu, bus| {
                    let ptr_hi = (cpu.tmp as u16).wrapping_add(1);
                    let hi = bus.read(ptr_hi);
                    cpu.effective_addr = ((hi as u16) << 8) | (cpu.effective_addr & 0xFF);
                },
            },
        ]
    }
}

impl Display for Addressing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Addressing::Implied => "implied".fmt(f),
            Addressing::Accumulator => "accumulator".fmt(f),
            Addressing::Immediate => "immediate".fmt(f),
            Addressing::Absolute => "absolute".fmt(f),
            Addressing::XIndexedAbsolute => "x_indexed_absolute".fmt(f),
            Addressing::YIndexedAbsolute => "y_indexed_absolute".fmt(f),
            Addressing::AbsoluteIndirect => "absolute_indirect".fmt(f),
            Addressing::ZeroPage => "zero_page".fmt(f),
            Addressing::XIndexedZeroPage => "x_indexed_zero_page".fmt(f),
            Addressing::YIndexedZeroPage => "y_indexed_zero_page".fmt(f),
            Addressing::XIndexedZeroPageIndirect => "x_indexed_zero_page_indirect".fmt(f),
            Addressing::ZeroPageIndirectYIndexed => "zero_page_indirect_y_indexed".fmt(f),
            Addressing::Relative => "relative".fmt(f),
        }
    }
}
