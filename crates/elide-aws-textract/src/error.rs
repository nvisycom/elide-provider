//! Error translation: Textract failures → [`elide_core::Error`].
//!
//! Crate-private — the public API reports [`elide_core::Error`]; this is
//! the internal seam the backend uses before bubbling up.

use elide_core::{Error, ErrorKind};

/// Errors surfaced internally by the Textract backend.
#[derive(Debug, thiserror::Error)]
pub(crate) enum TextractError {
    /// The call failed: transport, credentials, or a rejected request.
    #[error("textract error: {0}")]
    Sdk(String),
    /// The API answered, but not with blocks this backend can read.
    #[error("textract returned no usable blocks: {0}")]
    Protocol(String),
}

impl From<TextractError> for Error {
    /// Both map onto [`ErrorKind::Provider`]: the SDK wraps transport and
    /// service failures in one opaque error, and attributing a rejected
    /// request to `Transport` would make it look retryable when it is not.
    fn from(err: TextractError) -> Self {
        Error::new(ErrorKind::Provider, err)
    }
}
