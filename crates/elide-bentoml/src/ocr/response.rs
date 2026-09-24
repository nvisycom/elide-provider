//! Incoming wire types for the OCR `/recognize` endpoint.
//!
//! Mirrors `bento_core.ocr.v1.OcrResponse` from the inference
//! repository. The wire tree is `Page -> Block -> Line -> Word`;
//! elide's [`Layout`] is a flat list of regions, so
//! [`WireOcrResponse::decode`] keeps the words and discards the
//! groupings above them. The response-level `modelId`, per-page
//! `width`/`height`, per-block `kind`, and any rotated polygons are
//! deserialised-and-discarded for now.
//!
//! [`Layout`]: elide_image::modality::Layout

use elide_core::primitive::Confidence;
use elide_image::modality::{ImageLocation, LayoutRegion};
use elide_image::ocr::OcrResponse;
use elide_image::primitive::{BoundingBox, Point};
use serde::Deserialize;

/// Incoming per-call response body element.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireOcrResponse {
    #[serde(default)]
    pub pages: Vec<WirePage>,
    // `modelId` ignored: provenance comes from `BentoOcr::model_id`
    // (the deployment-level id the operator wired at construction).
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WirePage {
    /// 1-based page index; flows straight onto [`ImageLocation::page`].
    pub page_number: Option<u32>,
    #[serde(default)]
    pub blocks: Vec<WireBlock>,
    // `width`, `height` ignored: elide's layout does not carry page
    // dimensions today.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireBlock {
    #[serde(default)]
    pub lines: Vec<WireLine>,
    // `text`, `bbox` and `kind` (text / table / figure / other) are
    // deserialised-and-discarded: `Layout` is a flat list of regions, so
    // the block is a grouping the wire has and elide does not. Its words
    // carry the geometry and the confidence that survive.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireLine {
    #[serde(default)]
    pub words: Vec<WireWord>,
    // Per-line text and bbox are discarded for the same reason as the
    // block's: the words beneath carry both.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireWord {
    pub text: String,
    pub confidence: Option<f32>,
    pub bbox: WireBoundingBox,
    // `polygon` ignored: `ImageLocation` has a `polygon` field, but the
    // service reports one only for rotated regions, and nothing here
    // requests that mode yet.
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireBoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl From<WireBoundingBox> for BoundingBox<f64> {
    fn from(b: WireBoundingBox) -> Self {
        BoundingBox::new(
            Point::new(b.x, b.y),
            Point::new(b.x + b.width, b.y + b.height),
        )
    }
}

impl WireOcrResponse {
    /// Translate into the elide [`OcrResponse`] the backend trait expects.
    ///
    /// [`Layout`] is a flat list of regions, so the wire's
    /// pages → blocks → lines → words nesting collapses into one. Words
    /// are what survive: they are the finest granularity the service
    /// reports, and the only level carrying a confidence, which a
    /// redaction pipeline needs per span rather than per paragraph.
    ///
    /// [`Layout`]: elide_image::modality::Layout
    pub(super) fn decode(self) -> OcrResponse {
        let regions = self
            .pages
            .into_iter()
            .flat_map(|page| {
                let page_number = page.page_number;
                page.blocks
                    .into_iter()
                    .flat_map(|block| block.lines)
                    .flat_map(|line| line.words)
                    .map(move |word| word.decode(page_number))
            })
            .collect();
        OcrResponse::new(regions)
    }
}

impl WireWord {
    fn decode(self, page_number: Option<u32>) -> LayoutRegion {
        let region = ImageLocation {
            bounding_box: self.bbox.into(),
            polygon: None,
            page: page_number,
        };
        let mut layout = LayoutRegion::new(region, self.text);
        if let Some(c) = self.confidence {
            layout = layout.with_confidence(Confidence::clamped(c));
        }
        layout
    }
}
