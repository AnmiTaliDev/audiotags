#[derive(thiserror::Error, Debug)]
pub enum Id3Error {
    #[error("missing \"ID3\" file identifier")]
    NoTag,
    #[error("unsupported ID3v2 major version: {0}")]
    UnsupportedVersion(u8),
    #[error("tag is truncated or malformed")]
    Truncated,
    #[error("invalid text encoding byte: {0:#x}")]
    InvalidEncoding(u8),
    #[error("invalid UTF-8 in frame content")]
    InvalidUtf8,
    #[error("invalid UTF-16 in frame content")]
    InvalidUtf16,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
