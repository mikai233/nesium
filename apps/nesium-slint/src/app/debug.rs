use std::any::Any;

use anyhow::Result;
use crossbeam_channel::{Receiver, bounded};
use nesium_runtime::{DebugState, Event, EventTopic, RuntimeEventSender};
use slint::SharedString;

use crate::{DebuggerWindow, runtime::RuntimeSession};

#[derive(Debug, Clone)]
struct DebugEventSender {
    tx: crossbeam_channel::Sender<DebugState>,
}

impl RuntimeEventSender for DebugEventSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<DebugState>() {
            let _ = self.tx.try_send(*state);
            return true;
        }
        false
    }
}

pub struct DebugPanelController {
    rx: Option<Receiver<DebugState>>,
    latest: Option<DebugState>,
    subscribed: bool,
}

impl DebugPanelController {
    pub fn new() -> Self {
        Self {
            rx: None,
            latest: None,
            subscribed: false,
        }
    }

    pub fn set_enabled(&mut self, session: &RuntimeSession, enabled: bool) -> Result<()> {
        if enabled == self.subscribed {
            return Ok(());
        }

        if enabled {
            let (tx, rx) = bounded(1);
            session.subscribe_event(EventTopic::DebugState, Box::new(DebugEventSender { tx }))?;
            self.rx = Some(rx);
            self.subscribed = true;
        } else {
            session.unsubscribe_event(EventTopic::DebugState)?;
            self.rx = None;
            self.latest = None;
            self.subscribed = false;
        }

        Ok(())
    }

    pub fn drain(&mut self) {
        let Some(rx) = &self.rx else {
            return;
        };

        while let Ok(state) = rx.try_recv() {
            self.latest = Some(state);
        }
    }

    pub fn apply_to_window(&self, window: &DebuggerWindow) {
        if let Some(state) = &self.latest {
            window.set_debug_summary(SharedString::from("Live per-frame debug state"));
            window.set_debug_cpu_pc(SharedString::from(format!("${:04X}", state.cpu.pc)));
            window.set_debug_cpu_a(SharedString::from(format!("${:02X}", state.cpu.a)));
            window.set_debug_cpu_x(SharedString::from(format!("${:02X}", state.cpu.x)));
            window.set_debug_cpu_y(SharedString::from(format!("${:02X}", state.cpu.y)));
            window.set_debug_cpu_sp(SharedString::from(format!("${:02X}", state.cpu.sp)));
            window.set_debug_cpu_status(SharedString::from(format!("${:02X}", state.cpu.status)));
            window.set_debug_cpu_cycle(SharedString::from(state.cpu.cycle.to_string()));

            window.set_debug_ppu_scanline(SharedString::from(state.ppu.scanline.to_string()));
            window.set_debug_ppu_cycle(SharedString::from(state.ppu.cycle.to_string()));
            window.set_debug_ppu_frame(SharedString::from(state.ppu.frame.to_string()));
            window.set_debug_ppu_ctrl(SharedString::from(format!("${:02X}", state.ppu.ctrl)));
            window.set_debug_ppu_mask(SharedString::from(format!("${:02X}", state.ppu.mask)));
            window.set_debug_ppu_status(SharedString::from(format!("${:02X}", state.ppu.status)));
            window.set_debug_ppu_vram(SharedString::from(format!("${:04X}", state.ppu.vram_addr)));
            return;
        }

        let placeholder = SharedString::from("--");
        window.set_debug_summary(SharedString::from(
            if self.subscribed {
                "Waiting for debug data..."
            } else {
                "Debugger disabled"
            },
        ));
        window.set_debug_cpu_pc(placeholder.clone());
        window.set_debug_cpu_a(placeholder.clone());
        window.set_debug_cpu_x(placeholder.clone());
        window.set_debug_cpu_y(placeholder.clone());
        window.set_debug_cpu_sp(placeholder.clone());
        window.set_debug_cpu_status(placeholder.clone());
        window.set_debug_cpu_cycle(placeholder.clone());
        window.set_debug_ppu_scanline(placeholder.clone());
        window.set_debug_ppu_cycle(placeholder.clone());
        window.set_debug_ppu_frame(placeholder.clone());
        window.set_debug_ppu_ctrl(placeholder.clone());
        window.set_debug_ppu_mask(placeholder.clone());
        window.set_debug_ppu_status(placeholder.clone());
        window.set_debug_ppu_vram(placeholder);
    }
}
