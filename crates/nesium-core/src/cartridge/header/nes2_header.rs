use super::{INesHeader, Nes2Extension};

/// Parsed NES 2.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2Header {
    pub base: INesHeader,
    pub ext: Nes2Extension,
}
