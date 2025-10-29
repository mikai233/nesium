use crate::{bus::BusImpl, cpu::Cpu};
pub mod arith;
pub mod bra;
pub mod ctrl;
pub mod flags;
pub mod inc;
pub mod kill;
pub mod load;
pub mod logic;
pub mod nop;
pub mod shift;
pub mod stack;
pub mod trans;

type MicroFn = fn(&mut Cpu, bus: &mut BusImpl);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MicroOp {
    pub(crate) name: &'static str,
    pub(crate) micro_fn: MicroFn,
}

impl MicroOp {
    pub(crate) fn exec(&self, cpu: &mut Cpu, bus: &mut BusImpl) {
        (self.micro_fn)(cpu, bus)
    }
}
