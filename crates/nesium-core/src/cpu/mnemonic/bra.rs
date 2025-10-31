use crate::cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status};

impl Mnemonic {
    // ================================================================
    //  BCC - Branch if Carry Clear
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches to a relative address if the Carry flag (C) is clear.
    ///
    /// ⚙️ Operation:
    ///     If C == 0 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bcc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bcc",
            micro_fn: |cpu, _| {
                if !cpu.p.contains(Status::CARRY) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BCS - Branch if Carry Set
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches to a relative address if the Carry flag (C) is set.
    ///
    /// ⚙️ Operation:
    ///     If C == 1 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bcs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bcs",
            micro_fn: |cpu, _| {
                if cpu.p.contains(Status::CARRY) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BEQ - Branch if Equal (Zero Set)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Zero flag (Z) is set.
    ///
    /// ⚙️ Operation:
    ///     If Z == 1 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn beq() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "beq",
            micro_fn: |cpu, _| {
                if cpu.p.contains(Status::ZERO) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BMI - Branch if Minus (Negative Set)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Negative flag (N) is set.
    ///
    /// ⚙️ Operation:
    ///     If N == 1 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bmi() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bmi",
            micro_fn: |cpu, _| {
                if cpu.p.contains(Status::NEGATIVE) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BNE - Branch if Not Equal (Zero Clear)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Zero flag (Z) is clear.
    ///
    /// ⚙️ Operation:
    ///     If Z == 0 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bne() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bne",
            micro_fn: |cpu, _| {
                if !cpu.p.contains(Status::ZERO) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BPL - Branch if Plus (Negative Clear)
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Negative flag (N) is clear.
    ///
    /// ⚙️ Operation:
    ///     If N == 0 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bpl() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bpl",
            micro_fn: |cpu, _| {
                if !cpu.p.contains(Status::NEGATIVE) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BVC - Branch if Overflow Clear
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Overflow flag (V) is clear.
    ///
    /// ⚙️ Operation:
    ///     If V == 0 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bvc() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bvc",
            micro_fn: |cpu, _| {
                if !cpu.p.contains(Status::OVERFLOW) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }

    // ================================================================
    //  BVS - Branch if Overflow Set
    // ================================================================
    /// 🕹️ Purpose:
    ///     Branches if the Overflow flag (V) is set.
    ///
    /// ⚙️ Operation:
    ///     If V == 1 → PC ← PC + offset
    ///
    /// 🧩 Flags Affected:
    ///     None
    pub(crate) const fn bvs() -> &'static [MicroOp] {
        const OP1: MicroOp = MicroOp {
            name: "bvs",
            micro_fn: |cpu, _| {
                if cpu.p.contains(Status::OVERFLOW) {
                    cpu.branch();
                }
            },
        };
        &[OP1]
    }
}
