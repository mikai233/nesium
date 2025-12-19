//! CPU data-bus open-bus latch (Mesen2-style).
//!
//! The 2A03 data bus floats when no device is actively driving it. Reads from
//! write-only or unmapped addresses therefore return whatever value was last on
//! the bus ("open bus").
//!
//! In Mesen2 this is implemented as an `OpenBusHandler` with separate external
//! and internal latches. The only quirk we currently model is `$4015` (APU
//! status) reads updating the CPU *internal* data bus without updating the
//! external bus.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct OpenBus {
    external: u8,
    internal: u8,
}

impl OpenBus {
    #[inline]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Resets the open-bus state to its power-on value.
    #[inline]
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Returns the external open-bus value (what floating reads see).
    #[inline]
    pub(crate) fn sample(&self) -> u8 {
        self.external
    }

    /// Returns a deterministic "fake open bus" value for debugger peeks.
    ///
    /// Mirrors Mesen2's `OpenBusHandler::PeekRam`, which returns the address
    /// high byte.
    #[inline]
    pub(crate) fn peek(addr: u16) -> u8 {
        (addr >> 8) as u8
    }

    #[inline]
    pub(crate) fn internal_sample(&self) -> u8 {
        self.internal
    }

    /// Sets the open-bus latches (mirrors Mesen2's `SetOpenBus(value, setInternalOnly)`).
    #[inline]
    pub(crate) fn set(&mut self, value: u8, internal_only: bool) {
        if !internal_only {
            self.external = value;
        }
        self.internal = value;
    }

    /// Latches a freshly driven value onto the bus (updates both latches).
    #[inline]
    pub(crate) fn latch(&mut self, value: u8) {
        self.set(value, false);
    }

    /// Updates only the internal CPU data-bus latch (used for `$4015` reads).
    #[inline]
    pub(crate) fn set_internal_only(&mut self, value: u8) {
        self.set(value, true);
    }
}
