use crate::{bus::CpuBus, context::Context, cpu::Cpu};

/// N V - B D I Z C
/// - - - - - - - -
///
/// NOP - No Operation
/// Operation: No operation
///
/// Addressing Mode        | Assembly Language Form | Opcode | No. Bytes | No. Cycles
/// ---------------------- | ---------------------- | ------ | --------- | ----------
/// Implied                | NOP                    | $1A*   | 1         | 2
/// Implied                | NOP                    | $3A*   | 1         | 2
/// Implied                | NOP                    | $5A*   | 1         | 2
/// Implied                | NOP                    | $7A*   | 1         | 2
/// Implied                | NOP                    | $DA*   | 1         | 2
/// Implied                | NOP                    | $EA    | 1         | 2
/// Implied                | NOP                    | $FA*   | 1         | 2
/// Immediate              | NOP #$nn               | $80*   | 2         | 2
/// Immediate              | NOP #$nn               | $82*   | 2         | 2
/// Immediate              | NOP #$nn               | $89*   | 2         | 2
/// Immediate              | NOP #$nn               | $C2*   | 2         | 2
/// Immediate              | NOP #$nn               | $E2*   | 2         | 2
/// Absolute               | NOP $nnnn              | $0C*   | 3         | 4
/// X-Indexed Absolute     | NOP $nnnn,X            | $1C*   | 3         | 4+p
/// X-Indexed Absolute     | NOP $nnnn,X            | $3C*   | 3         | 4+p
/// X-Indexed Absolute     | NOP $nnnn,X            | $5C*   | 3         | 4+p
/// X-Indexed Absolute     | NOP $nnnn,X            | $7C*   | 3         | 4+p
/// X-Indexed Absolute     | NOP $nnnn,X            | $DC*   | 3         | 4+p
/// X-Indexed Absolute     | NOP $nnnn,X            | $FC*   | 3         | 4+p
/// Zero Page              | NOP $nn                | $04*   | 2         | 3
/// Zero Page              | NOP $nn                | $44*   | 2         | 3
/// Zero Page              | NOP $nn                | $64*   | 2         | 3
/// X-Indexed Zero Page    | NOP $nn,X              | $14*   | 2         | 4
/// X-Indexed Zero Page    | NOP $nn,X              | $34*   | 2         | 4
/// X-Indexed Zero Page    | NOP $nn,X              | $54*   | 2         | 4
/// X-Indexed Zero Page    | NOP $nn,X              | $74*   | 2         | 4
/// X-Indexed Zero Page    | NOP $nn,X              | $D4*   | 2         | 4
/// X-Indexed Zero Page    | NOP $nn,X              | $F4*   | 2         | 4
/// *Undocumented.
/// p: =1 if page is crossed.
#[inline]
pub fn exec_nop(cpu: &mut Cpu, bus: &mut CpuBus, ctx: &mut Context, step: u8) {
    match step {
        0 => {
            cpu.dummy_read(bus, ctx);
        }
        _ => unreachable_step!("invalid NOP step {step}"),
    }
}
