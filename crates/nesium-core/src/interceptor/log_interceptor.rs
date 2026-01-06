use crate::interceptor::Interceptor;

#[derive(Debug, Clone, Copy)]
pub struct LogInterceptor;

impl Interceptor for LogInterceptor {}
