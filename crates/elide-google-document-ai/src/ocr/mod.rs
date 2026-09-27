//! [`DocumentAiOcr`]: an [`OcrBackend`] backed by Google Cloud Document AI.
//!
//! [`OcrBackend`]: elide_image::ocr::OcrBackend

mod response;

use async_trait::async_trait;
use elide_core::Result;
use elide_core::entity::audit::ModelEvent;
use elide_image::ocr::{OcrBackend, OcrRequest, OcrResponse};
use google_cloud_documentai_v1::client::DocumentProcessorService;
use google_cloud_documentai_v1::model::RawDocument;
use hipstr::HipStr;

use crate::error::DocumentAiBackendError;

/// An [`OcrBackend`] backed by Google Cloud Document AI.
#[derive(Clone)]
pub struct DocumentAiOcr {
    client: DocumentProcessorService,
    processor: String,
}

impl DocumentAiOcr {
    /// Build from a client and the full resource name of the processor to
    /// call, `projects/{p}/locations/{l}/processors/{id}`.
    #[must_use]
    pub fn new(client: DocumentProcessorService, processor: impl Into<String>) -> Self {
        Self {
            client,
            processor: processor.into(),
        }
    }
}

/// The MIME type to declare for the request.
///
/// Always `application/octet-stream`, which leaves Document AI to sniff
/// the bytes — it does so when the type is generic.
///
/// [`ImageFormat`] would be the better source, but its variants are
/// feature-gated on `elide-image` for the formats that crate can *decode*,
/// and this backend decodes nothing, so it enables none of them and the
/// enum is empty here. Wiring the hint through would mean pulling an image
/// codec to name a MIME type.
///
/// [`ImageFormat`]: elide_image::modality::ImageFormat
const fn mime_type() -> &'static str {
    "application/octet-stream"
}

#[async_trait]
impl OcrBackend for DocumentAiOcr {
    fn provenance(&self) -> ModelEvent {
        ModelEvent {
            name: HipStr::borrowed("google/document-ai"),
            version: None,
            contextual: false,
        }
    }

    async fn recognize(&self, request: OcrRequest<'_>) -> Result<OcrResponse> {
        // Document AI dispatches on the MIME type, and sniffs the bytes
        // when given the generic one; see `mime_type`.
        let raw = RawDocument::new()
            .set_content(request.image.to_vec())
            .set_mime_type(mime_type());

        let response = self
            .client
            .process_document()
            .set_name(&self.processor)
            .set_raw_document(raw)
            .send()
            .await
            .map_err(DocumentAiBackendError::from)?;

        let document = response.document.ok_or_else(|| {
            DocumentAiBackendError::Protocol("response carried no document".into())
        })?;

        Ok(self::response::decode(&document))
    }
}
