use crate::{bus::Bus, interceptor::Interceptor};

#[derive(Debug, Clone, Copy)]
pub struct LogInterceptor;

impl Interceptor for LogInterceptor {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(std::any::type_name::<Self>())
    }

    fn debug(&self, cpu: &mut crate::cpu::Cpu, bus: &mut dyn Bus) {
        tracing::debug!("{}", cpu.a);
    }
}
