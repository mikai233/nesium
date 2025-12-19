/// Layout mirroring type for the PPU nametables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mirroring {
    /// Two horizontal nametables that mirror vertically (common for NTSC games).
    Horizontal,
    /// Two vertical nametables that mirror horizontally.
    Vertical,
    /// Cartridge supplies its own four nametables.
    FourScreen,
    /// Single-screen mirroring using the first nametable (`$2000` region).
    SingleScreenLower,
    /// Single-screen mirroring using the second nametable (`$2400` region).
    SingleScreenUpper,
    /// Nametable mapping is fully controlled by the mapper via `map_nametable`.
    MapperControlled,
}
