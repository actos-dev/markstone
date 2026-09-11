use std::fmt;

/// Errors returned by the markstone core engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkstoneError {
    /// Input size exceeds the 4 MiB limit.
    InputTooLarge,
    /// Block nesting depth exceeds the limit of 64.
    DepthExceeded,
    /// Input contains invalid UTF-8 byte sequences.
    InvalidUtf8,
    /// Internal processing error.
    Internal(String),
}

impl fmt::Display for MarkstoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge => write!(f, "input exceeds maximum size limit of 4 MiB"),
            Self::DepthExceeded => write!(f, "block nesting depth exceeds limit of 64"),
            Self::InvalidUtf8 => write!(f, "invalid utf-8 byte sequence"),
            Self::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for MarkstoneError {}
