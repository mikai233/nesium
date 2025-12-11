use crate::{bus::CpuBus, cpu::Cpu, interceptor::Interceptor};

#[derive(Debug, Clone, Copy)]
pub struct LogInterceptor;

impl Interceptor for LogInterceptor {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(std::any::type_name::<Self>())
    }

    fn debug(&self, cpu: &mut Cpu, _bus: &mut CpuBus<'_>) {
        tracing::debug!("{}", cpu.a);
    }
}
