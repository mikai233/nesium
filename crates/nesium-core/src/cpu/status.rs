use bitflags::bitflags;

bitflags! {
    /// Represents the 8-bit processor status register (P) of the NES CPU.
    ///
    /// Bit layout:
    /// 7 6 5 4 3 2 1 0
    /// N V _ B D I Z C
    ///
    /// Each bit is a flag that reflects CPU state after arithmetic,
    /// logical, or control operations.
    #[derive(Debug, Clone, Copy)]
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
    /// Create a new Status with default power-up state (usually 0x34 or 0x24 depending on emulator).
    pub fn new() -> Self {
        Status::from_bits_truncate(0x24)
    }

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

    /// Convert the flags to a byte.
    pub fn to_byte(&self) -> u8 {
        self.bits()
    }

    /// Load flags from a byte value.
    pub fn from_byte(byte: u8) -> Self {
        Status::from_bits_truncate(byte)
    }
}
