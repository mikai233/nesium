use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags6: u8 {
        const MIRRORING        = 0b0000_0001;
        const BATTERY          = 0b0000_0010;
        const TRAINER          = 0b0000_0100;
        const FOUR_SCREEN      = 0b0000_1000;
        const MAPPER_LOW_MASK  = 0b1111_0000;
    }
}
