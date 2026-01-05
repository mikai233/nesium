use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use crate::{bus::CpuBus, cpu::Cpu};

pub mod capture_point;
pub mod log_interceptor;
pub mod palette_interceptor;
pub mod sprite_interceptor;
pub mod tile_viewer_interceptor;
pub mod tilemap_interceptor;

pub trait Interceptor: Any + Send + Debug + 'static {
    fn debug(&self, cpu: &mut Cpu, bus: &mut CpuBus);

    fn on_ppu_frame_start(&mut self, _cpu: &mut Cpu, _bus: &mut CpuBus) {}

    fn on_ppu_vblank_start(&mut self, _cpu: &mut Cpu, _bus: &mut CpuBus) {}

    fn on_ppu_scanline_dot(
        &mut self,
        _cpu: &mut Cpu,
        _bus: &mut CpuBus,
        _scanline: i16,
        _dot: u16,
    ) {
    }
}

#[derive(Debug, Default)]
pub struct EmuInterceptor {
    layers: HashMap<TypeId, Box<dyn Interceptor>>,
}

impl EmuInterceptor {
    /// Create an empty interceptor stack.
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    /// Add a new interceptor to the end of the stack.
    pub fn add<I>(&mut self, interceptor: I) -> Option<Box<dyn Interceptor>>
    where
        I: Interceptor,
    {
        self.layers.insert(TypeId::of::<I>(), Box::new(interceptor))
    }

    pub fn remove<I>(&mut self) -> Option<Box<dyn Interceptor>>
    where
        I: Interceptor,
    {
        self.layers.remove(&TypeId::of::<I>())
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

    pub fn layer<T: Interceptor>(&self) -> Option<&T> {
        let layer = self.layers.get(&TypeId::of::<T>())?;
        (layer.as_ref() as &dyn Any).downcast_ref::<T>()
    }

    pub fn layer_mut<T: Interceptor>(&mut self) -> Option<&mut T> {
        let layer = self.layers.get_mut(&TypeId::of::<T>())?;
        (layer.as_mut() as &mut dyn Any).downcast_mut::<T>()
    }
}

impl Interceptor for EmuInterceptor {
    fn debug(&self, cpu: &mut Cpu, bus: &mut CpuBus) {
        for interceptor in self.layers.values() {
            interceptor.debug(cpu, bus);
        }
    }

    fn on_ppu_frame_start(&mut self, cpu: &mut Cpu, bus: &mut CpuBus) {
        for interceptor in self.layers.values_mut() {
            interceptor.on_ppu_frame_start(cpu, bus);
        }
    }

    fn on_ppu_vblank_start(&mut self, cpu: &mut Cpu, bus: &mut CpuBus) {
        for interceptor in self.layers.values_mut() {
            interceptor.on_ppu_vblank_start(cpu, bus);
        }
    }

    fn on_ppu_scanline_dot(&mut self, cpu: &mut Cpu, bus: &mut CpuBus, scanline: i16, dot: u16) {
        for interceptor in self.layers.values_mut() {
            interceptor.on_ppu_scanline_dot(cpu, bus, scanline, dot);
        }
    }
}
