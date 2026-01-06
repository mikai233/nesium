//! Debug interceptor implementation for breakpoint and stepping support.

use std::collections::HashSet;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use nesium_core::CpuSnapshot;
use nesium_core::bus::CpuBus;
use nesium_core::context::Context;
use nesium_core::cpu::Cpu;
use nesium_core::interceptor::Interceptor;

use super::control::ControlMessage;
use super::debug::{DebugCommand, DebugEvent, PauseReason};

/// Step mode determines when to pause after resuming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum StepMode {
    #[default]
    None,
    /// Pause after one CPU instruction.
    Instruction,
    /// Pause after the current frame completes.
    Frame,
    /// Pause when reaching the target scanline.
    Scanline { target: i16 },
}

/// Debug interceptor that handles breakpoints and stepping.
#[derive(Debug)]
pub struct DebugInterceptor {
    /// Control message receiver (shared with Runner).
    ctrl_rx: Receiver<ControlMessage>,
    /// Control message sender (for forwarding messages back to Runner).
    ctrl_tx: Sender<ControlMessage>,
    /// Debug command receiver.
    debug_rx: Receiver<DebugCommand>,
    /// Debug event sender.
    debug_tx: Sender<DebugEvent>,

    /// Whether execution is currently paused.
    paused: bool,
    /// Step mode for single-step operations.
    step_mode: StepMode,
    /// Whether to break immediately after a reset.
    break_on_reset: bool,
    /// Set of active breakpoint addresses.
    breakpoints: HashSet<u16>,
}

impl DebugInterceptor {
    /// Creates a new debug interceptor with the given channels.
    pub fn new(
        ctrl_rx: Receiver<ControlMessage>,
        ctrl_tx: Sender<ControlMessage>,
        debug_rx: Receiver<DebugCommand>,
        debug_tx: Sender<DebugEvent>,
    ) -> Self {
        Self {
            ctrl_rx,
            ctrl_tx,
            debug_rx,
            debug_tx,
            paused: false,
            step_mode: StepMode::None,
            break_on_reset: false,
            breakpoints: HashSet::new(),
        }
    }

    /// Checks if we should break at the current CPU state.
    fn should_break(&self, cpu: &Cpu) -> bool {
        // Check breakpoints
        if self.breakpoints.contains(&cpu.pc) {
            return true;
        }

        // Check step modes
        match self.step_mode {
            StepMode::Instruction => true,
            StepMode::Frame => false, // Handled elsewhere via on_ppu_frame_start
            StepMode::Scanline { .. } => false, // Handled via on_ppu_scanline_dot
            StepMode::None => false,
        }
    }

    /// Enters the pause loop, processing commands until resumed.
    fn enter_pause_loop(&mut self, cpu: &mut Cpu, bus: &mut CpuBus, reason: PauseReason) {
        self.paused = true;
        self.step_mode = StepMode::None;

        // Notify UI that we're paused
        let _ = self
            .debug_tx
            .send(DebugEvent::Paused { pc: cpu.pc, reason });

        while self.paused {
            // Try debug commands first (higher priority)
            if let Ok(cmd) = self.debug_rx.try_recv() {
                self.handle_debug_command(cmd, cpu, bus);
                continue;
            }

            // Try control messages
            if let Ok(msg) = self.ctrl_rx.try_recv() {
                self.handle_control_while_paused(msg);
                continue;
            }

            // Sleep briefly to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(1));
        }

        // Notify UI that we're resuming
        let _ = self.debug_tx.send(DebugEvent::Resumed);
    }

    /// Handles a debug command during the pause loop.
    fn handle_debug_command(&mut self, cmd: DebugCommand, cpu: &mut Cpu, bus: &mut CpuBus) {
        match cmd {
            DebugCommand::Pause => {
                // Already paused, ignore
            }
            DebugCommand::Continue => {
                self.paused = false;
            }
            DebugCommand::StepInstruction => {
                self.step_mode = StepMode::Instruction;
                self.paused = false;
            }
            DebugCommand::StepFrame => {
                self.step_mode = StepMode::Frame;
                self.paused = false;
            }
            DebugCommand::StepScanline { target } => {
                self.step_mode = StepMode::Scanline { target };
                self.paused = false;
            }
            DebugCommand::ReadMemory { addr, len, reply } => {
                let mut data = Vec::with_capacity(len);
                for i in 0..len {
                    let byte = bus.peek(addr.wrapping_add(i as u16), cpu, &mut Context::None);
                    data.push(byte);
                }
                let _ = reply.send(Ok(data));
            }
            DebugCommand::WriteByte { addr, value, reply } => {
                bus.write(addr, value, cpu, &mut Context::None);
                let _ = reply.send(Ok(()));
            }
            DebugCommand::GetCpuState { reply } => {
                let snapshot = CpuSnapshot {
                    pc: cpu.pc,
                    a: cpu.a,
                    x: cpu.x,
                    y: cpu.y,
                    s: cpu.s,
                    p: cpu.status_bits(),
                };
                let _ = reply.send(Ok(snapshot));
            }
            DebugCommand::AddBreakpoint { addr } => {
                self.breakpoints.insert(addr);
            }
            DebugCommand::RemoveBreakpoint { addr } => {
                self.breakpoints.remove(&addr);
            }
            DebugCommand::ListBreakpoints { reply } => {
                let list: Vec<u16> = self.breakpoints.iter().copied().collect();
                let _ = reply.send(Ok(list));
            }
        }
    }

    /// Handles a control message while paused.
    ///
    /// For messages that need Runner-level access (like Reset), we forward
    /// them back to the control channel and exit the pause loop.
    fn handle_control_while_paused(&mut self, msg: ControlMessage) {
        match &msg {
            ControlMessage::Reset(_, _) => {
                // Set break-on-reset flag, forward the message, and exit pause
                self.break_on_reset = true;
                self.paused = false;
                let _ = self.ctrl_tx.send(msg);
            }
            ControlMessage::Stop => {
                // Forward and exit
                self.paused = false;
                let _ = self.ctrl_tx.send(msg);
            }
            ControlMessage::LoadRom(_, _) => {
                // Forward and exit (will trigger reset anyway)
                self.break_on_reset = true;
                self.paused = false;
                let _ = self.ctrl_tx.send(msg);
            }
            _ => {
                // For other messages, just forward them
                let _ = self.ctrl_tx.send(msg);
            }
        }
    }

    /// Returns whether debugging is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns a reference to the breakpoint set.
    pub fn breakpoints(&self) -> &HashSet<u16> {
        &self.breakpoints
    }
}

impl Interceptor for DebugInterceptor {
    fn debug(&mut self, cpu: &mut Cpu, bus: &mut CpuBus) {
        // Check break-on-reset flag
        if self.break_on_reset {
            self.break_on_reset = false;
            self.enter_pause_loop(cpu, bus, PauseReason::Reset);
            return;
        }

        // Check if we should break
        if self.should_break(cpu) {
            let reason = if self.breakpoints.contains(&cpu.pc) {
                PauseReason::Breakpoint
            } else {
                PauseReason::Step
            };
            self.enter_pause_loop(cpu, bus, reason);
        }

        // Check for pause request (non-blocking)
        if let Ok(DebugCommand::Pause) = self.debug_rx.try_recv() {
            self.enter_pause_loop(cpu, bus, PauseReason::UserRequest);
        }
    }

    fn on_ppu_frame_start(&mut self, cpu: &mut Cpu, bus: &mut CpuBus) {
        if self.step_mode == StepMode::Frame {
            self.enter_pause_loop(cpu, bus, PauseReason::Step);
        }
    }

    fn on_ppu_scanline_dot(&mut self, cpu: &mut Cpu, bus: &mut CpuBus, scanline: i16, dot: u16) {
        if let StepMode::Scanline { target } = self.step_mode {
            if scanline == target && dot == 0 {
                self.enter_pause_loop(cpu, bus, PauseReason::Step);
            }
        }
    }
}
