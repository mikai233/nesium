use crate::{bus::CpuBus, cpu::Cpu, interceptor::Interceptor};

#[derive(Debug, Clone, Copy)]
pub struct LogInterceptor;

impl Interceptor for LogInterceptor {
    fn debug(&self, cpu: &mut Cpu, _bus: &mut CpuBus) {
        tracing::debug!("{}", cpu.a);
    }
}
