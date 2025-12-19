/// NES 2.0: number of additional "miscellaneous" ROM regions.
///
/// This count is stored in header byte 14 bits 0..=1 and ranges from 0..=3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2MiscRomCount(pub u8);
