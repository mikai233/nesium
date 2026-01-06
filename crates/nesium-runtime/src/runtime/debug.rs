//! Debug interceptor types and communication structures.

use crossbeam_channel::Sender;

use super::types::RuntimeError;

/// Reply sender for debug commands that return a value.
pub type DebugReplySender<T> = Sender<Result<T, RuntimeError>>;

/// Debug command sent from UI to DebugInterceptor.
#[derive(Debug)]
pub enum DebugCommand {
    /// Request to pause execution.
    Pause,
    /// Resume execution from paused state.
    Continue,
    /// Execute one CPU instruction then pause.
    StepInstruction,
    /// Execute until next frame then pause.
    StepFrame,
    /// Execute until specified scanline then pause.
    StepScanline { target: i16 },
    /// Read memory from CPU address space.
    ReadMemory {
        addr: u16,
        len: usize,
        reply: DebugReplySender<Vec<u8>>,
    },
    /// Write a single byte to CPU address space.
    WriteByte {
        addr: u16,
        value: u8,
        reply: DebugReplySender<()>,
    },
    /// Get current CPU state snapshot.
    GetCpuState {
        reply: DebugReplySender<nesium_core::CpuSnapshot>,
    },
    /// Add a breakpoint at the specified address.
    AddBreakpoint { addr: u16 },
    /// Remove a breakpoint at the specified address.
    RemoveBreakpoint { addr: u16 },
    /// List all active breakpoints.
    ListBreakpoints { reply: DebugReplySender<Vec<u16>> },
}

/// Debug event sent from DebugInterceptor to UI.
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Execution has paused.
    Paused { pc: u16, reason: PauseReason },
    /// Execution has resumed.
    Resumed,
    /// A breakpoint was hit.
    BreakpointHit { addr: u16 },
    /// A step operation completed.
    StepCompleted { pc: u16 },
}

/// Reason for pausing execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseReason {
    /// Hit a user-defined breakpoint.
    Breakpoint,
    /// Completed a step operation.
    Step,
    /// User requested pause.
    UserRequest,
    /// Paused after reset (break-on-reset feature).
    Reset,
}
