/// Console type advertised by the iNES / NES 2.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleType {
    /// Standard NES/Famicom cartridge.
    NesFamicom,
    /// Vs. System arcade hardware.
    VsSystem,
    /// PlayChoice-10 hardware.
    PlayChoice10,
    /// NES 2.0 extended console type (see [`super::Nes2ConsoleTypeData`]).
    Extended,
}

impl ConsoleType {
    pub(super) fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::NesFamicom,
            1 => Self::VsSystem,
            2 => Self::PlayChoice10,
            3 => Self::Extended,
            _ => unreachable!("masked to 2 bits"),
        }
    }
}
