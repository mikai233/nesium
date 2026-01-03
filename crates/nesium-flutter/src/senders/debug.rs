use std::any::Any;

use nesium_runtime::{DebugState, Event, RuntimeEventSender};

use crate::api::events::DebugStateNotification;
use crate::frb_generated::StreamSink;

/// Sender that forwards DebugState events to Flutter.
pub struct FlutterDebugEventSender {
    pub(crate) sink: StreamSink<DebugStateNotification>,
}

impl FlutterDebugEventSender {
    pub fn new(sink: StreamSink<DebugStateNotification>) -> Self {
        Self { sink }
    }
}

impl RuntimeEventSender for FlutterDebugEventSender {
    fn send(&self, event: Box<dyn Event>) -> bool {
        let any: Box<dyn Any> = event;
        if let Ok(state) = any.downcast::<DebugState>() {
            let notification = DebugStateNotification {
                cpu_pc: state.cpu.pc,
                cpu_a: state.cpu.a,
                cpu_x: state.cpu.x,
                cpu_y: state.cpu.y,
                cpu_sp: state.cpu.sp,
                cpu_status: state.cpu.status,
                cpu_cycle: state.cpu.cycle,
                ppu_scanline: state.ppu.scanline,
                ppu_cycle: state.ppu.cycle,
                ppu_frame: state.ppu.frame,
                ppu_ctrl: state.ppu.ctrl,
                ppu_mask: state.ppu.mask,
                ppu_status: state.ppu.status,
            };
            let _ = self.sink.add(notification);
        }
        true
    }
}
