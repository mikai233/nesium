use crate::cpu::{micro_op::MicroOp, mnemonic::Mnemonic};

impl Mnemonic {
    /// N V - B D I Z C
    /// - - - - - - - 0
    ///
    /// CLC - Clear Carry Flag
    /// Operation: 0 → C
    ///
    /// This instruction initializes the carry flag to a 0. This operation should
    /// normally precede an ADC loop. It is also useful when used with a ROL
    /// instruction to clear a bit in memory.
    ///
    /// This instruction affects no registers in the microprocessor and no flags
    /// other than the carry flag which is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLC                    | $18    | 1         | 2
    pub(crate) const fn clc() -> &'static [MicroOp] {
        &[MicroOp {
            name: "clc_clear_carry",
            micro_fn: |cpu, _| {
                // Cycle 2: C = 0
                cpu.p.set_c(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - 0 - - -
    ///
    /// CLD - Clear Decimal Mode
    /// Operation: 0 → D
    ///
    /// This instruction sets the decimal mode flag to a 0. This all subsequent
    /// ADC and SBC instructions to operate as simple operations.
    ///
    /// CLD affects no registers in the microprocessor and no flags other than the
    /// decimal mode flag which is set to a 0.
    ///
    /// **Note on the MOS 6502:**
    ///
    /// The value of the decimal mode flag is indeterminate after a RESET.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLD                    | $D8    | 1         | 2
    pub(crate) const fn cld() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cld_clear_decimal",
            micro_fn: |cpu, _| {
                // Cycle 2: D = 0
                cpu.p.set_d(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - 0 - -
    ///
    /// CLI - Clear Interrupt Disable
    /// Operation: 0 → I
    ///
    /// This instruction initializes the interrupt disable to a 0. This allows the
    /// microprocessor to receive interrupts.
    ///
    /// It affects no registers in the microprocessor and no flags other than the
    /// interrupt disable which is cleared.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLI                    | $58    | 1         | 2
    pub(crate) const fn cli() -> &'static [MicroOp] {
        &[MicroOp {
            name: "cli_clear_interrupt",
            micro_fn: |cpu, _| {
                // Cycle 2: I = 0. When interrupts were previously disabled,
                // the 6502 delays servicing a pending IRQ until *after* the
                // next instruction completes. Model this with a one-boundary
                // suppression flag plus an I-flag pipeline update.
                let was_disabled = cpu.p.i();
                cpu.queue_i_update(false);
                if was_disabled {
                    cpu.irq_inhibit_next = true;
                }
            },
        }]
    }

    /// N V - B D I Z C
    /// - 0 - - - - - -
    ///
    /// CLV - Clear Overflow Flag
    /// Operation: 0 → V
    ///
    /// This instruction clears the overflow flag to a 0. This command is used in
    /// conjunction with the set overflow pin which can change the state of the
    /// overflow flag with an external signal.
    ///
    /// CLV affects no registers in the microprocessor and no flags other than the
    /// overflow flag which is set to a 0.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | CLV                    | $B8    | 1         | 2
    pub(crate) const fn clv() -> &'static [MicroOp] {
        &[MicroOp {
            name: "clv_clear_overflow",
            micro_fn: |cpu, _| {
                // Cycle 2: V = 0
                cpu.p.set_v(false);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - - - 1
    ///
    /// SEC - Set Carry Flag
    /// Operation: 1 → C
    ///
    /// This instruction initializes the carry flag to a 1. This operation should
    /// normally precede a SBC loop. It is also useful when used with a ROL
    /// instruction to initialize a bit in memory to a 1.
    ///
    /// This instruction affects no registers in the microprocessor and no flags
    /// other than the carry flag which is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SEC                    | $38    | 1         | 2
    pub(crate) const fn sec() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sec_set_carry",
            micro_fn: |cpu, _| {
                // Cycle 2: C = 1
                cpu.p.set_c(true);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - 1 - - -
    ///
    /// SED - Set Decimal Mode
    /// Operation: 1 → D
    ///
    /// This instruction sets the decimal mode flag D to a 1. This makes all
    /// subsequent ADC and SBC instructions operate as a decimal arithmetic
    /// operation.
    ///
    /// SED affects no registers in the microprocessor and no flags other than the
    /// decimal mode which is set to a 1.
    ///
    /// **Note on the MOS 6502:**
    ///
    /// The value of the decimal mode flag is indeterminate after a RESET.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SED                    | $F8    | 1         | 2
    pub(crate) const fn sed() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sed_set_decimal",
            micro_fn: |cpu, _| {
                // Cycle 2: D = 1
                cpu.p.set_d(true);
            },
        }]
    }

    /// N V - B D I Z C
    /// - - - - - 1 - -
    ///
    /// SEI - Set Interrupt Disable
    /// Operation: 1 → I
    ///
    /// This instruction initializes the interrupt disable to a 1. It is used to
    /// mask interrupt requests during system reset operations and during interrupt
    /// commands.
    ///
    /// It affects no registers in the microprocessor and no flags other than the
    /// interrupt disable which is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | SEI                    | $78    | 1         | 2
    pub(crate) const fn sei() -> &'static [MicroOp] {
        &[MicroOp {
            name: "sei_set_interrupt",
            micro_fn: |cpu, _| {
                // Cycle 2: I = 1. If interrupts were previously enabled when
                // SEI executes, a pending IRQ is still allowed to fire "just
                // after" SEI. Approximate this with a one-shot override that
                // permits a single IRQ even though I is now set.
                let was_enabled = !cpu.p.i();
                cpu.queue_i_update(true);
                if was_enabled {
                    cpu.allow_irq_once = true;
                }
            },
        }]
    }
}

#[cfg(test)]
mod flags_test {
    use crate::cpu::mnemonic::{Mnemonic, tests::InstrTest};

    #[test]
    fn test_clc() {
        InstrTest::new(Mnemonic::CLC).test(|_, cpu, _| {
            assert!(!cpu.p.c(), "Carry flag should be cleared");
        });
    }

    #[test]
    fn test_cld() {
        InstrTest::new(Mnemonic::CLD).test(|_, cpu, _| {
            assert!(!cpu.p.d(), "Decimal Mode flag should be cleared");
        });
    }

    #[test]
    fn test_cli() {
        InstrTest::new(Mnemonic::CLI).test(|_, cpu, _| {
            assert!(!cpu.p.i(), "Interrupt Disable flag should be cleared");
        });
    }

    #[test]
    fn test_clv() {
        InstrTest::new(Mnemonic::CLV).test(|_, cpu, _| {
            assert!(!cpu.p.v(), "Overflow flag should be cleared");
        });
    }

    #[test]
    fn test_sec() {
        InstrTest::new(Mnemonic::SEC).test(|_, cpu, _| {
            assert!(cpu.p.c(), "Carry flag should be set");
        });
    }

    #[test]
    fn test_sed() {
        InstrTest::new(Mnemonic::SED).test(|_, cpu, _| {
            assert!(cpu.p.d(), "Decimal Mode flag should be set");
        });
    }

    #[test]
    fn test_sei() {
        InstrTest::new(Mnemonic::SEI).test(|_, cpu, _| {
            assert!(cpu.p.i(), "Interrupt Disable flag should be set");
        });
    }
}
