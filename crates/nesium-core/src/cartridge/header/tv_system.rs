/// Video timing hints embedded in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TvSystem {
    /// NTSC (60Hz) timing.
    Ntsc,
    /// PAL (50Hz) timing.
    Pal,
    /// Cartridge can run on either timing without modification.
    Dual, // region free: supports NTSC and PAL timing
    Dendy, // hybrid timing used by some Famiclones
    Unknown,
}
