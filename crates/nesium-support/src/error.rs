use thiserror::Error;

#[derive(Error, Debug)]
pub enum SupportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid TAS movie data: {0}")]
    InvalidTasData(String),

    #[error("Unsupported TAS movie format: {0}")]
    UnsupportedTasFormat(String),
}
