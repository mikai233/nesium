use crate::{
    bus::CpuBus,
    context::Context,
    cpu::{Cpu, micro_op::MicroOp, mnemonic::Mnemonic},
};

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
#[inline]
pub fn exec_bcc(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(!cpu.p.c());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BCC step {step}"),
    }
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
#[inline]
pub fn exec_bcs(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(cpu.p.c());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BCS step {step}"),
    }
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
#[inline]
pub fn exec_beq(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(cpu.p.z());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BEQ step {step}"),
    }
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
#[inline]
pub fn exec_bmi(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(cpu.p.n());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BMI step {step}"),
    }
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
#[inline]
pub fn exec_bne(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(!cpu.p.z());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BNE step {step}"),
    }
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
#[inline]
pub fn exec_bpl(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(!cpu.p.n());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BPL step {step}"),
    }
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
#[inline]
pub fn exec_bvc(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(!cpu.p.v());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BVC step {step}"),
    }
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
#[inline]
pub fn exec_bvs(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.tmp = cpu.fetch_u8(bus, ctx);
            cpu.test_branch(cpu.p.v());
        }
        1 => {
            cpu.dummy_read(bus, ctx);
            let old_pc = cpu.pc;
            let offset = cpu.tmp as i8;
            let new_pc = old_pc.wrapping_add(offset as u16);
            cpu.pc = new_pc;
            cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
        }
        2 => {
            bus.mem_read(cpu.pc, cpu, ctx);
        }
        _ => unreachable_step!("invalid BVS step {step}"),
    }
}

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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(!cpu.p.c());
                },
            },
            MicroOp {
                name: "bcc_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bcc_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(cpu.p.c());
                },
            },
            MicroOp {
                name: "bcs_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bcs_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(cpu.p.z());
                },
            },
            MicroOp {
                name: "beq_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "beq_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(cpu.p.n());
                },
            },
            MicroOp {
                name: "bmi_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bmi_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(!cpu.p.z());
                },
            },
            MicroOp {
                name: "bne_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bne_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(!cpu.p.n());
                },
            },
            MicroOp {
                name: "bpl_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bpl_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(!cpu.p.v());
                },
            },
            MicroOp {
                name: "bvc_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bvc_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
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
                micro_fn: |cpu, bus, ctx| {
                    cpu.tmp = cpu.fetch_u8(bus, ctx);
                    cpu.test_branch(cpu.p.v());
                },
            },
            MicroOp {
                name: "bvs_add_branch_offset",
                micro_fn: |cpu, bus, ctx| {
                    cpu.dummy_read(bus, ctx);
                    let old_pc = cpu.pc;
                    let offset = cpu.tmp as i8;
                    let new_pc = old_pc.wrapping_add(offset as u16);
                    cpu.pc = new_pc;
                    cpu.skip_optional_dummy_read_cycle(old_pc, new_pc);
                },
            },
            MicroOp {
                name: "bvs_add_dummy_read",
                micro_fn: |cpu, bus, ctx| {
                    bus.mem_read(cpu.pc, cpu, ctx);
                },
            },
        ]
    }
}
