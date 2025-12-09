use crate::error::Error;
use core::convert::TryFrom;
use core::fmt;

/// PPU chip model (Ricoh variants).
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum PpuModel {
    #[default]
    Ricoh2C02 = 0,
    Ricoh2C03 = 1,
    Ricoh2C04A = 2,
    Ricoh2C04B = 3,
    Ricoh2C04C = 4,
    Ricoh2C04D = 5,
    Ricoh2C05A = 6,
    Ricoh2C05B = 7,
    Ricoh2C05C = 8,
    Ricoh2C05D = 9,
    Ricoh2C05E = 10,
}

impl PpuModel {
    /// Human-readable name (useful for logs / debug UI).
    pub const fn as_str(self) -> &'static str {
        match self {
            PpuModel::Ricoh2C02 => "Ricoh 2C02",
            PpuModel::Ricoh2C03 => "Ricoh 2C03",
            PpuModel::Ricoh2C04A => "Ricoh 2C04A",
            PpuModel::Ricoh2C04B => "Ricoh 2C04B",
            PpuModel::Ricoh2C04C => "Ricoh 2C04C",
            PpuModel::Ricoh2C04D => "Ricoh 2C04D",
            PpuModel::Ricoh2C05A => "Ricoh 2C05A",
            PpuModel::Ricoh2C05B => "Ricoh 2C05B",
            PpuModel::Ricoh2C05C => "Ricoh 2C05C",
            PpuModel::Ricoh2C05D => "Ricoh 2C05D",
            PpuModel::Ricoh2C05E => "Ricoh 2C05E",
        }
    }
}

impl fmt::Display for PpuModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convert from the raw numeric id (must match the C++ enum values).
impl TryFrom<u8> for PpuModel {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let model = match value {
            0 => PpuModel::Ricoh2C02,
            1 => PpuModel::Ricoh2C03,
            2 => PpuModel::Ricoh2C04A,
            3 => PpuModel::Ricoh2C04B,
            4 => PpuModel::Ricoh2C04C,
            5 => PpuModel::Ricoh2C04D,
            6 => PpuModel::Ricoh2C05A,
            7 => PpuModel::Ricoh2C05B,
            8 => PpuModel::Ricoh2C05C,
            9 => PpuModel::Ricoh2C05D,
            10 => PpuModel::Ricoh2C05E,
            _ => return Err(Error::UnsupportedPpuModel(value)),
        };
        Ok(model)
    }
}

/// Convert back to the raw numeric id (for state, serialization, FFI, etc.).
impl From<PpuModel> for u8 {
    fn from(model: PpuModel) -> Self {
        model as u8
    }
}
