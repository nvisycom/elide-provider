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
    /// A call that never reached the service maps onto
    /// [`ErrorKind::Transport`]; anything the service answered, onto
    /// [`ErrorKind::Provider`].
    fn from(err: DocumentAiBackendError) -> Self {
        let kind = match &err {
            // A rejected document is the service answering, and retrying it
            // would fail the same way. Anything else — a timeout, a failure
            // to connect, a proxy answering in the service's place — never
            // got that far, so the request may be safe to retry.
            //
            // `status()` is the test because it is `Some` only for an RPC
            // status the service itself returned. Enumerating the transport
            // shapes instead would miss the ones gax models as a transport
            // error carrying an HTTP status, and its `is_transport` sits
            // outside the crate's public semver surface.
            DocumentAiBackendError::Sdk(sdk) if sdk.status().is_none() => ErrorKind::Transport,
            DocumentAiBackendError::Sdk(_) | DocumentAiBackendError::Protocol(_) => {
                ErrorKind::Provider
            }
        };
        Error::new(kind, err)
    }
}
