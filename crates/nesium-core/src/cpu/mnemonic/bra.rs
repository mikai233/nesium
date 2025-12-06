use crate::cpu::{micro_op::MicroOp, mnemonic::Mnemonic};

impl Mnemonic {
    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BCC - Branch on Carry Clear
    /// Operation: Branch on C = 0
    ///
    /// This instruction tests the state of the carry bit and takes a conditional
    /// branch if the carry bit is reset.
    ///
    /// It affects no flags or registers other than the program counter and then
    /// only if the C flag is not on.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BCC $nnnn              | $90    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bcc() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bcc_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(!cpu.p.c());
                },
            },
            MicroOp {
                name: "bcc_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bcc_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BCS - Branch on Carry Set
    /// Operation: Branch on C = 1
    ///
    /// This instruction takes the conditional branch if the carry flag is on.
    ///
    /// BCS does not affect any of the flags or registers except for the program
    /// counter and only then if the carry flag is on.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BCS $nnnn              | $B0    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bcs() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bcs_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(cpu.p.c());
                },
            },
            MicroOp {
                name: "bcs_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bcs_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BEQ - Branch on Result Zero
    /// Operation: Branch on Z = 1
    ///
    /// This instruction could also be called "Branch on Equal."
    ///
    /// It takes a conditional branch whenever the Z flag is on or the previous
    /// result is equal to 0.
    ///
    /// BEQ does not affect any of the flags or registers other than the program
    /// counter and only then when the Z flag is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BEQ $nnnn              | $F0    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn beq() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "beq_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(cpu.p.z());
                },
            },
            MicroOp {
                name: "beq_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "beq_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BMI - Branch on Result Minus
    /// Operation: Branch on N = 1
    ///
    /// This instruction takes the conditional branch if the N bit is set.
    ///
    /// BMI does not affect any of the flags or any other part of the machine
    /// other than the program counter and then only if the N bit is on.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BMI $nnnn              | $30    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bmi() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bmi_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(cpu.p.n());
                },
            },
            MicroOp {
                name: "bmi_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bmi_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BNE - Branch on Result Not Zero
    /// Operation: Branch on Z = 0
    ///
    /// This instruction could also be called "Branch on Not Equal." It tests the
    /// Z flag and takes the conditional branch if the Z flag is not on, indicating
    /// that the previous result was not zero.
    ///
    /// BNE does not affect any of the flags or registers other than the program
    /// counter and only then if the Z flag is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BNE $nnnn              | $D0    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bne() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bne_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(!cpu.p.z());
                },
            },
            MicroOp {
                name: "bne_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bne_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BPL - Branch on Result Plus
    /// Operation: Branch on N = 0
    ///
    /// This instruction is the complementary branch to branch on result minus. It
    /// is a conditional branch which takes the branch when the N bit is reset (0).
    /// BPL is used to test if the previous result bit 7 was off (0) and branch on
    /// result minus is used to determine if the previous result was minus or bit 7
    /// was on (1).
    ///
    /// The instruction affects no flags or other registers other than the P counter
    /// and only affects the P counter when the N bit is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BPL $nnnn              | $10    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bpl() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bpl_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(!cpu.p.n());
                },
            },
            MicroOp {
                name: "bpl_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bpl_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BVC - Branch on Overflow Clear
    /// Operation: Branch on V = 0
    ///
    /// This instruction tests the status of the V flag and takes the conditional
    /// branch if the flag is not set.
    ///
    /// BVC does not affect any of the flags and registers other than the program
    /// counter and only when the overflow flag is reset.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BVC $nnnn              | $50    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bvc() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bvc_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(!cpu.p.v());
                },
            },
            MicroOp {
                name: "bvc_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bvc_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }

    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// BVS - Branch on Overflow Set
    /// Operation: Branch on V = 1
    ///
    /// This instruction tests the V flag and takes the conditional branch if V is on.
    ///
    /// BVS does not affect any flags or registers other than the program, counter
    /// and only when the overflow flag is set.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Relative        | BVS $nnnn              | $70    | 2         | 2+t+p
    ///
    /// p: =1 if page is crossed.
    /// t: =1 if branch is taken.
    pub(crate) const fn bvs() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "bvs_fetch_branch_offset",
                micro_fn: |cpu, bus| {
                    cpu.base = bus.mem_read(cpu.pc);
                    cpu.incr_pc();
                    cpu.test_branch(cpu.p.v());
                },
            },
            MicroOp {
                name: "bvs_add_branch_offset",
                micro_fn: |cpu, bus| {
                    bus.internal_cycle();
                    let old_pc = cpu.pc;
                    let offset = cpu.base as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.check_cross_page(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bvs_add_dummy_read",
                micro_fn: |cpu, bus| {
                    bus.mem_read(cpu.pc);
                },
            },
        ]
    }
}

#[cfg(test)]
mod bra_tests {
    use crate::cpu::mnemonic::{Mnemonic, tests::InstrTest};

    #[test]
    fn test_bcc() {
        InstrTest::new(Mnemonic::BCC).test_branch(|verify, cpu, bus| {
            let old_carry = verify.cpu.p.c();
            let branch_taken = !old_carry;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }
            branch_taken
        });
    }

    #[test]
    fn test_bcs() {
        InstrTest::new(Mnemonic::BCS).test_branch(|verify, cpu, bus| {
            let old_carry = verify.cpu.p.c();
            let branch_taken = old_carry;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_beq() {
        InstrTest::new(Mnemonic::BEQ).test_branch(|verify, cpu, bus| {
            let old_zero = verify.cpu.p.z();
            let branch_taken = old_zero;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_bmi() {
        InstrTest::new(Mnemonic::BMI).test_branch(|verify, cpu, bus| {
            let old_negative = verify.cpu.p.n();
            let branch_taken = old_negative;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_bne() {
        InstrTest::new(Mnemonic::BNE).test_branch(|verify, cpu, bus| {
            let old_zero = verify.cpu.p.z();
            let branch_taken = !old_zero;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_bpl() {
        InstrTest::new(Mnemonic::BPL).test_branch(|verify, cpu, bus| {
            let old_negative = verify.cpu.p.n();
            let branch_taken = !old_negative;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_bvc() {
        InstrTest::new(Mnemonic::BVC).test_branch(|verify, cpu, bus| {
            let old_overflow = verify.cpu.p.v();
            let branch_taken = !old_overflow;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }

    #[test]
    fn test_bvs() {
        InstrTest::new(Mnemonic::BVS).test_branch(|verify, cpu, bus| {
            let old_overflow = verify.cpu.p.v();
            let branch_taken = old_overflow;

            if branch_taken {
                let offset = bus.mem_read(verify.cpu.pc.wrapping_add(1)) as i8;
                let expected_pc = verify.cpu.pc.wrapping_add(2).wrapping_add(offset as u16);
                assert_eq!(
                    cpu.pc, expected_pc,
                    "PC not updated correctly after branch taken"
                );
            } else {
                let expected_pc = verify.cpu.pc.wrapping_add(2);
                assert_eq!(cpu.pc, expected_pc, "PC not correct after branch not taken");
            }

            branch_taken
        });
    }
}
