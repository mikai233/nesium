#[cfg(feature = "savestate-serde")]
use serde::{Deserialize, Serialize};

use super::OpenBus;

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OpenBusState {
    pub external: u8,
    pub internal: u8,
}

impl OpenBusState {
    pub(crate) fn from_open_bus(bus: OpenBus) -> Self {
        Self {
            external: bus.sample(),
            internal: bus.internal_sample(),
        }
    }

    pub(crate) fn apply_to(self, bus: &mut OpenBus) {
        bus.set(self.external, false);
        bus.set(self.internal, true);
    }
}
