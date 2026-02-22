use bitflags::bitflags;

// Layout (bits 0-14):
//  14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
//  [fine_y][nt][coarse_y   ][coarse_x   ]
//  yyy     NN   YYYYY         XXXXX
bitflags! {
    /// Bit masks for the 15-bit VRAM address (`v`/`t` registers).
    pub(crate) struct VramAddrMask: u16 {
        const COARSE_X = 0x001F;   // bits 0-4
        const COARSE_Y = 0x03E0;   // bits 5-9
        const NAMETABLE = 0x0C00;  // bits 10-11
        const FINE_Y = 0x7000;     // bits 12-14
        const ALL = Self::COARSE_X.bits()
            | Self::COARSE_Y.bits()
            | Self::NAMETABLE.bits()
            | Self::FINE_Y.bits();
    }
}

const COARSE_Y_SHIFT: u16 = 5;
const NAMETABLE_SHIFT: u16 = 10;
const FINE_Y_SHIFT: u16 = 12;

/// 15-bit VRAM address used by the PPU internal `v`/`t` registers.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct VramAddr(pub(crate) u16);

impl VramAddr {
    /// Returns the coarse X scroll component (0..31).
    #[inline]
    pub fn coarse_x(self) -> u8 {
        (self.0 & VramAddrMask::COARSE_X.bits()) as u8
    }

    /// Updates the coarse X scroll component (0..31).
    #[inline]
    pub fn set_coarse_x(&mut self, cx: u8) {
        self.0 = (self.0 & !VramAddrMask::COARSE_X.bits()) | u16::from(cx & 0b1_1111);
    }

    /// Returns the coarse Y scroll component (0..31).
    #[inline]
    pub fn coarse_y(self) -> u8 {
        ((self.0 & VramAddrMask::COARSE_Y.bits()) >> COARSE_Y_SHIFT) as u8
    }

    /// Updates the coarse Y scroll component (0..31).
    #[inline]
    pub fn set_coarse_y(&mut self, cy: u8) {
        self.0 = (self.0 & !VramAddrMask::COARSE_Y.bits())
            | (u16::from(cy & 0b1_1111) << COARSE_Y_SHIFT);
    }

    /// Returns the selected nametable (0..3).
    #[inline]
    pub fn nametable(self) -> u8 {
        ((self.0 & VramAddrMask::NAMETABLE.bits()) >> NAMETABLE_SHIFT) as u8
    }

    /// Updates the selected nametable (0..3).
    #[inline]
    pub fn set_nametable(&mut self, nt: u8) {
        self.0 =
            (self.0 & !VramAddrMask::NAMETABLE.bits()) | (u16::from(nt & 0b11) << NAMETABLE_SHIFT);
    }

    /// Returns the fine Y scroll component (0..7).
    #[inline]
    pub fn fine_y(self) -> u8 {
        ((self.0 & VramAddrMask::FINE_Y.bits()) >> FINE_Y_SHIFT) as u8
    }

    /// Updates the fine Y scroll component (0..7).
    #[inline]
    pub fn set_fine_y(&mut self, fy: u8) {
        self.0 = (self.0 & !VramAddrMask::FINE_Y.bits()) | (u16::from(fy & 0b111) << FINE_Y_SHIFT);
    }

    /// Returns the raw 15-bit value.
    #[inline]
    pub fn raw(self) -> u16 {
        self.0
    }

    /// Replaces the raw address, masking to 15 bits.
    #[inline]
    pub fn set_raw(&mut self, v: u16) {
        self.0 = v & VramAddrMask::ALL.bits();
    }

    /// Returns a copy with coarse X updated.
    #[inline]
    pub fn with_coarse_x(mut self, cx: u8) -> Self {
        self.set_coarse_x(cx);
        self
    }

    /// Returns a copy with coarse Y updated.
    #[inline]
    pub fn with_coarse_y(mut self, cy: u8) -> Self {
        self.set_coarse_y(cy);
        self
    }

    /// Returns a copy with nametable bits updated.
    #[inline]
    pub fn with_nametable(mut self, nt: u8) -> Self {
        self.set_nametable(nt);
        self
    }

    /// Returns a copy with fine Y updated.
    #[inline]
    pub fn with_fine_y(mut self, fy: u8) -> Self {
        self.set_fine_y(fy);
        self
    }

    /// Increments the raw internal 15-bit address (`v`/`t` style register).
    ///
    /// Hardware keeps bit 14 in the internal latch; only external VRAM
    /// accesses are mirrored to `$0000-$3FFF`.
    #[inline]
    pub fn increment(&mut self, step: u16) {
        self.0 = (self.0 + step) & VramAddrMask::ALL.bits();
    }
}

impl core::fmt::Debug for VramAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VramAddr")
            .field("raw", &format_args!("{:#06X}", self.0))
            .field("fine_y", &self.fine_y())
            .field("nametable", &self.nametable())
            .field("coarse_y", &self.coarse_y())
            .field("coarse_x", &self.coarse_x())
            .finish()
    }
}

impl core::fmt::Display for VramAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "raw={:#06X} fy={} nt={} cy={} cx={}",
            self.0,
            self.fine_y(),
            self.nametable(),
            self.coarse_y(),
            self.coarse_x(),
        )
    }
}

impl From<u16> for VramAddr {
    #[inline]
    fn from(v: u16) -> Self {
        VramAddr(v & VramAddrMask::ALL.bits())
    }
}

impl From<VramAddr> for u16 {
    #[inline]
    fn from(v: VramAddr) -> Self {
        v.raw()
    }
}
