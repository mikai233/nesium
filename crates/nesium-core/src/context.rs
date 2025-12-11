use crate::interceptor::EmuInterceptor;

#[derive(Debug)]
pub enum Context<'a> {
    None,
    Some { interceptor: &'a mut EmuInterceptor },
}
