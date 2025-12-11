use std::{borrow::Cow, fmt::Debug};

use crate::{bus::CpuBus, cpu::Cpu};

pub mod log_interceptor;

pub trait Interceptor: Send + Debug + 'static {
    fn name(&self) -> Cow<'static, str>;

    fn debug(&self, cpu: &mut Cpu, bus: &mut CpuBus<'_>);
}

#[derive(Debug, Default)]
pub struct EmuInterceptor {
    layers: Vec<Box<dyn Interceptor>>,
}

impl EmuInterceptor {
    /// Create an empty interceptor stack.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Create an interceptor stack from an existing list of layers.
    pub fn from_layers(layers: Vec<Box<dyn Interceptor>>) -> Self {
        Self { layers }
    }

    /// Add a new interceptor to the end of the stack.
    pub fn add<I>(&mut self, interceptor: I)
    where
        I: Interceptor,
    {
        self.layers.push(Box::new(interceptor));
    }

    /// Remove and return the first interceptor whose `name()` matches `name`.
    pub fn remove_first_by_name(&mut self, name: &str) -> Option<Box<dyn Interceptor>> {
        if let Some(pos) = self.layers.iter().position(|layer| layer.name() == name) {
            Some(self.layers.remove(pos))
        } else {
            None
        }
    }

    /// Remove all interceptors whose `name()` matches `name`.
    pub fn remove_all_by_name(&mut self, name: &str) {
        self.layers.retain(|layer| layer.name() != name);
    }

    /// Remove all interceptors from the stack.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Number of interceptors currently in the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Returns true if there are no interceptors in the stack.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Immutable view of the underlying interceptor list.
    pub fn layers(&self) -> &[Box<dyn Interceptor>] {
        &self.layers
    }
}

impl Interceptor for EmuInterceptor {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(std::any::type_name::<Self>())
    }

    fn debug(&self, cpu: &mut Cpu, bus: &mut CpuBus<'_>) {
        for interceptor in &self.layers {
            interceptor.debug(cpu, bus);
        }
    }
}
