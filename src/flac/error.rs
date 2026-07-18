#[derive(thiserror::Error, Debug)]
pub enum FlacError {
    #[error("not a FLAC file: missing \"fLaC\" marker")]
    NotFlac,
    #[error("FLAC metadata is truncated or malformed")]
    Truncated,
    #[error("invalid UTF-8 in FLAC metadata")]
    InvalidUtf8,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
