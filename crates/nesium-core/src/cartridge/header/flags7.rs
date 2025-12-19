use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags7: u8 {
        const VS_UNISYSTEM     = 0b0000_0001;
        const PLAYCHOICE_10    = 0b0000_0010;
        const NES2_DETECTION   = 0b0000_1100;
        const MAPPER_HIGH_MASK = 0b1111_0000;
    }
}
