use crate::cpu::{
    micro_op::{MicroOp, empty_micro_fn},
    mnemonic::Mnemonic,
};

impl Mnemonic {
    // ================================================================
    //  NOP - No Operation
    // ================================================================
    /// 🕹️ Purpose:
    ///     Does nothing, consumes CPU cycles.
    ///
    /// ⚙️ Operation:
    ///     None
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn nop() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "nop",
            micro_fn: empty_micro_fn,
        };
        &[OP1]
    }

    // ================================================================
    //  JAM - CPU Lock / Halt
    // ================================================================
    /// 🕹️ Purpose:
    ///     Represents execution of an illegal opcode that halts the CPU.
    ///
    /// ⚙️ Operation:
    ///     Conceptually, the CPU would enter an infinite loop.
    ///     In this implementation, the effect is handled outside the MicroOp.
    ///
    /// 🧩 Flags Affected:
    ///     None
    ///
    /// 💡 Note:
    ///     This instruction is handled externally because we avoid adding
    ///     a dedicated halt flag in the `Cpu` structure.
    pub(crate) const fn jam() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "jam",
            micro_fn: empty_micro_fn,
        };
        &[OP1]
    }
}
