use crate::ppu::registers::Control;

/// Delayed VRAM auto-increment kind after a `$2007` access.
///
/// Hardware supports only two increments (1 or 32), and in many cases there is
/// no pending increment at all, so we can encode this in a compact enum instead
/// of a separate `bool + u16` pair.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum PendingVramIncrement {
    #[default]
    None,
    By1,
    By32,
}

impl PendingVramIncrement {
    pub(crate) fn from_control(control: Control) -> Self {
        match control.vram_increment() {
            1 => PendingVramIncrement::By1,
            32 => PendingVramIncrement::By32,
            _ => PendingVramIncrement::None,
        }
    }

    #[inline]
    pub(crate) fn is_pending(self) -> bool {
        !matches!(self, PendingVramIncrement::None)
    }

    #[inline]
    pub(crate) fn amount(self) -> u16 {
        match self {
            PendingVramIncrement::None => 0,
            PendingVramIncrement::By1 => 1,
            PendingVramIncrement::By32 => 32,
        }
    }
}
