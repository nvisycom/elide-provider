//! Outgoing wire types for the STT `/transcribe` endpoint.
//!
//! Mirrors `bento_core.stt.v1.SttRequest` from the inference
//! repository: base64-encoded audio bytes plus an optional filename
//! and language hint.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use elide_audio::stt::SttRequest;
use serde::Serialize;

/// Outgoing per-call request body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireSttRequest {
    /// Base64-encoded audio bytes.
    pub audio: String,
    /// Original filename, when the caller supplied one.
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Always `None`: elide's `SttRequest` no longer carries a filename,
    /// so there is nothing to fill it from. Kept on the wire because the
    /// service still accepts it and uses it for container detection.
    pub filename: Option<String>,
    /// Caller-asserted language as a BCP-47 tag, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl WireSttRequest {
    pub(super) fn from_request(request: &SttRequest<'_>) -> Self {
        Self {
            audio: BASE64.encode(request.audio),
            // elide's `SttRequest` no longer carries a filename, so the
            // service falls back to sniffing the container from the bytes.
            filename: None,
            language: request.language.map(ToString::to_string),
        }
    }
}
