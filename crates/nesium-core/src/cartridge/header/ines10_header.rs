use super::{INes10Extension, INesHeader};

/// Parsed iNES 1.0 header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INes10Header {
    pub base: INesHeader,
    pub ext: INes10Extension,
}
