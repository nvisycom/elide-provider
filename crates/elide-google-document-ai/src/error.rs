//! Error translation: [`google_cloud_documentai_v1::Error`] →
//! [`elide_core::Error`].
//!
//! Crate-private — the public API reports [`elide_core::Error`]; this is
//! the internal seam the backend uses before bubbling up.

use elide_core::{Error, ErrorKind};
use google_cloud_documentai_v1::Error as DocumentAiError;

/// Errors surfaced internally by the Document AI backend.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DocumentAiBackendError {
    /// The SDK failed: transport, auth, or a rejected request.
    #[error("document ai error: {0}")]
    Sdk(#[from] DocumentAiError),
    /// The API answered, but not with a document this backend can read.
    #[error("document ai returned no usable document: {0}")]
    Protocol(String),
}

impl From<DocumentAiBackendError> for Error {
    /// Anything the transport did maps onto [`ErrorKind::Transport`];
    /// anything the service answered onto [`ErrorKind::Provider`].
    fn from(err: DocumentAiBackendError) -> Self {
        let kind = match &err {
            // The gax error type does not expose a stable transport/service
            // discriminant, so a failed call is attributed to the provider:
            // over-reporting Transport would make a rejected request look
            // retryable when it is not.
            DocumentAiBackendError::Sdk(_) | DocumentAiBackendError::Protocol(_) => {
                ErrorKind::Provider
            }
        };
        Error::new(kind, err)
    }
}
