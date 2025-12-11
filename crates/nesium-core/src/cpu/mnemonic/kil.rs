use crate::{
    bus::Bus,
    cpu::{
        Cpu,
        micro_op::{MicroOp, empty_micro_fn},
        mnemonic::Mnemonic,
        unreachable_step,
    },
};

/// N V - B D I Z C
/// - - - - - - - -
///
/// JAM - Halt the CPU
/// Operation: Stop execution
///
/// This undocumented instruction stops execution. The microprocessor will not
/// fetch further instructions, and will neither handle IRQs nor NMIs. It will
/// handle a RESET though.
///
/// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// --------------- | ---------------------- | ------ | --------- | ----------
/// Implied         | JAM                    | $02*   | 1         | X
/// Implied         | JAM                    | $12*   | 1         | X
/// Implied         | JAM                    | $22*   | 1         | X
/// Implied         | JAM                    | $32*   | 1         | X
/// Implied         | JAM                    | $42*   | 1         | X
/// Implied         | JAM                    | $52*   | 1         | X
/// Implied         | JAM                    | $62*   | 1         | X
/// Implied         | JAM                    | $72*   | 1         | X
/// Implied         | JAM                    | $92*   | 1         | X
/// Implied         | JAM                    | $B2*   | 1         | X
/// Implied         | JAM                    | $D2*   | 1         | X
/// Implied         | JAM                    | $F2*   | 1         | X
/// *Undocumented.
#[inline]
pub fn exec_jam<B: Bus>(cpu: &mut Cpu, bus: &mut B, step: u8) {
    match step {
        0 => {
            // Match the legacy empty_micro_fn: burn a cycle and effectively halt.
            bus.internal_cycle(cpu);
        }
        _ => unreachable_step!("invalid JAM step {step}"),
    }
}

impl Mnemonic {
    /// N V - B D I Z C
    /// - - - - - - - -
    ///
    /// JAM - Halt the CPU
    /// Operation: Stop execution
    ///
    /// This undocumented instruction stops execution. The microprocessor will not
    /// fetch further instructions, and will neither handle IRQs nor NMIs. It will
    /// handle a RESET though.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ---------------------- | ------ | --------- | ----------
    /// Implied         | JAM                    | $02*   | 1         | X
    /// Implied         | JAM                    | $12*   | 1         | X
    /// Implied         | JAM                    | $22*   | 1         | X
    /// Implied         | JAM                    | $32*   | 1         | X
    /// Implied         | JAM                    | $42*   | 1         | X
    /// Implied         | JAM                    | $52*   | 1         | X
    /// Implied         | JAM                    | $62*   | 1         | X
    /// Implied         | JAM                    | $72*   | 1         | X
    /// Implied         | JAM                    | $92*   | 1         | X
    /// Implied         | JAM                    | $B2*   | 1         | X
    /// Implied         | JAM                    | $D2*   | 1         | X
    /// Implied         | JAM                    | $F2*   | 1         | X
    /// *Undocumented.
    pub(crate) const fn jam() -> &'static [MicroOp] {
        &[MicroOp {
            name: "jam",
            micro_fn: empty_micro_fn,
        }]
    }
}
