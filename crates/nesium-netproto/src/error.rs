use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("bad magic")]
    BadMagic,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u8),
    #[error("buffer too short")]
    TooShort,
    #[error("payload length mismatch")]
    LengthMismatch,
    #[error("payload too large: {0}")]
    PayloadTooLarge(usize),
    #[error("frame too large: {0}")]
    FrameTooLarge(usize),
    #[error("unknown msg id: {0}")]
    UnknownMsgId(u8),
    #[error("postcard decode error: {0}")]
    Postcard(#[from] postcard::Error),
}
