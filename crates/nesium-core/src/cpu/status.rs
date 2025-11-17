use std::fmt::{Debug, Display};

use bitflags::bitflags;

pub(crate) const BIT_0: u8 = 1 << 0;
// pub(crate) const BIT_1: u8 = 1 << 1;
// pub(crate) const BIT_2: u8 = 1 << 2;
// pub(crate) const BIT_3: u8 = 1 << 3;
// pub(crate) const BIT_4: u8 = 1 << 4;
pub(crate) const BIT_5: u8 = 1 << 5;
pub(crate) const BIT_6: u8 = 1 << 6;
pub(crate) const BIT_7: u8 = 1 << 7;

bitflags! {
    /// Represents the 8-bit processor status register (P) of the NES CPU.
    ///
    /// Bit layout:
    /// 7 6 5 4 3 2 1 0
    /// N V _ B D I Z C
    ///
    /// Each bit is a flag that reflects CPU state after arithmetic,
    /// logical, or control operations.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Status: u8 {
        /// Carry flag (C)
        /// Set when an addition produces a carry out of bit 7,
        /// or a subtraction requires a borrow.
        const CARRY     = 0b0000_0001;

        /// Zero flag (Z)
        /// Set when the result of an operation is zero.
        const ZERO      = 0b0000_0010;

        /// Interrupt Disable flag (I)
        /// When set, maskable interrupts (IRQ) are disabled.
        const INTERRUPT = 0b0000_0100;

        /// Decimal Mode flag (D)
        /// Has no effect on the NES CPU (since decimal mode is not implemented),
        /// but still exists for compatibility with the 6502 instruction set.
        const DECIMAL   = 0b0000_1000;

        /// Break Command flag (B)
        /// Set when a BRK instruction is executed, indicating a software interrupt.
        const BREAK     = 0b0001_0000;

        /// Unused bit (always 1 in the status pushed to stack)
        /// The NES hardware ignores this, but emulators often set it for consistency.
        const UNUSED    = 0b0010_0000;

        /// Overflow flag (V)
        /// Set when signed arithmetic overflows.
        const OVERFLOW  = 0b0100_0000;

        /// Negative flag (N)
        /// Reflects the sign bit (bit 7) of the result of the last operation.
        const NEGATIVE  = 0b1000_0000;
    }
}

impl Status {
    /// Set or clear the Zero flag based on a value.
    pub fn update_zero(&mut self, value: u8) {
        if value == 0 {
            self.insert(Status::ZERO);
        } else {
            self.remove(Status::ZERO);
        }
    }

    /// Set or clear the Negative flag based on bit 7 of a value.
    pub fn update_negative(&mut self, value: u8) {
        if value & 0x80 != 0 {
            self.insert(Status::NEGATIVE);
        } else {
            self.remove(Status::NEGATIVE);
        }
    }

    #[inline]
    pub fn set_zn(&mut self, value: u8) {
        self.update_zero(value);
        self.update_negative(value);
    }

    /// Update carry flag (C) using bitflags API
    #[inline]
    pub fn set_c(&mut self, value: bool) {
        self.set(Status::CARRY, value);
    }

    /// Update decimal flag (D) using bitflags API
    #[inline]
    pub fn set_d(&mut self, value: bool) {
        self.set(Status::DECIMAL, value);
    }

    /// Update interrupt flag (I) using bitflags API
    #[inline]
    pub fn set_i(&mut self, value: bool) {
        self.set(Status::INTERRUPT, value);
    }

    /// Update unused flag (U) using bitflags API
    #[inline]
    pub fn set_u(&mut self, value: bool) {
        self.set(Status::UNUSED, value);
    }

    /// Update break flag (B) using bitflags API
    #[inline]
    pub fn set_b(&mut self, value: bool) {
        self.set(Status::BREAK, value);
    }

    /// Set N flag to a specific bit (bit 7 of memory)
    #[inline]
    pub fn set_n(&mut self, value: bool) {
        self.set(Status::NEGATIVE, value);
    }

    /// Set V flag to a specific bit (bit 6 of memory)
    #[inline]
    pub fn set_v(&mut self, value: bool) {
        self.set(Status::OVERFLOW, value);
    }

    #[inline]
    pub fn set_z(&mut self, value: bool) {
        self.set(Status::ZERO, value);
    }

    pub fn n(&self) -> bool {
        self.contains(Status::NEGATIVE)
    }

    pub fn z(&self) -> bool {
        self.contains(Status::ZERO)
    }

    pub(crate) fn u(&self) -> bool {
        self.contains(Status::UNUSED)
    }

    pub fn v(&self) -> bool {
        self.contains(Status::OVERFLOW)
    }

    pub fn c(&self) -> bool {
        self.contains(Status::CARRY)
    }

    pub fn i(&self) -> bool {
        self.contains(Status::INTERRUPT)
    }

    pub fn b(&self) -> bool {
        self.contains(Status::BREAK)
    }

    pub fn d(&self) -> bool {
        self.contains(Status::DECIMAL)
    }
}

impl Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let flags = [
            (self.n(), "N"),
            (self.v(), "V"),
            (self.u(), "U"),
            (self.b(), "B"),
            (self.d(), "D"),
            (self.i(), "I"),
            (self.z(), "Z"),
            (self.c(), "C"),
        ];
        let mut iter = flags
            .into_iter()
            .filter_map(|(t, n)| if t { Some(n) } else { None });
        if let Some(n) = iter.next() {
            write!(f, "{n}")?;
            for n in iter {
                write!(f, ".{}", n)?;
            }
        }
        write!(f, "]")
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn symbol(b: bool) -> &'static str {
            if b { "✓" } else { "✗" }
        }
        let n = symbol(self.n());
        let v = symbol(self.v());
        let u = symbol(self.u());
        let b = symbol(self.b());
        let d = symbol(self.d());
        let i = symbol(self.i());
        let z = symbol(self.z());
        let c = symbol(self.c());
        write!(f, "N:{n}|V:{v}|U:{u}|B:{b}|D:{d}|I:{i}|Z:{z}|C:{c}")
    }
}
