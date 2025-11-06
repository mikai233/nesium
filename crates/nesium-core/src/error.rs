use std::fmt;

use crate::cartridge::header::NES_HEADER_LEN;
use crate::cartridge::header::RomFormat;

#[derive(Debug)]
pub enum Error {
    /// Provided buffer is shorter than the 16-byte header.
    TooShort { actual: usize },
    /// Magic number ("NES<EOF>") is missing.
    InvalidMagic,
    /// Header advertises a format we do not implement yet.
    UnsupportedFormat(RomFormat),
    /// A ROM section (trainer/PRG/CHR/...) is shorter than advertised.
    SectionTooShort {
        section: &'static str,
        expected: usize,
        actual: usize,
    },
    /// Wrapper for I/O errors raised while reading ROMs from disk.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort { actual } => {
                write!(f, "header expected {NES_HEADER_LEN} bytes, got {actual}")
            }
            Self::InvalidMagic => write!(f, "missing NES magic bytes"),
            Self::UnsupportedFormat(format) => {
                write!(f, "unsupported iNES header format: {format:?}")
            }
            Self::SectionTooShort {
                section,
                expected,
                actual,
            } => write!(
                f,
                "{section} section expected {expected} bytes, got {actual}"
            ),
            Self::Io(err) => write!(f, "i/o error while reading cartridge: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
