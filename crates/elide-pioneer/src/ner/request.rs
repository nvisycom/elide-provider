//! Outgoing wire types for `POST /gliner-2`.
//!
//! Mirrors the body the `gliner2` Python client sends. Pioneer's own
//! enrichment tasks (classification, structured records, relations) are
//! not modelled: elide's [`NerBackend`] asks for entity spans, and the
//! extra results would be discarded on the way out.
//!
//! [`NerBackend`]: elide_ner::backend::NerBackend

use serde::Serialize;

/// The task this backend asks for. Pioneer routes on it.
const TASK: &str = "extract_entities";

/// One extraction request.
#[derive(Debug, Serialize)]
pub(super) struct WireRequest<'a> {
    pub task: &'static str,
    pub text: &'a str,
    /// The labels to extract. Bare names: Pioneer's wire schema is a list
    /// of strings, and the descriptions elide's `Label` can carry have no
    /// slot in it.
    pub schema: Vec<String>,
    pub threshold: f32,
    /// Both always on: without them the response carries no confidence and
    /// no offsets, which are exactly what a `NerSpan` is made of.
    pub include_confidence: bool,
    pub include_spans: bool,
    /// Keeps the label-keyed shape this crate decodes.
    pub format_results: bool,
}

impl<'a> WireRequest<'a> {
    /// A request over `text` for `labels`.
    pub(super) fn new(text: &'a str, labels: Vec<String>, threshold: f32) -> Self {
        Self {
            task: TASK,
            text,
            schema: labels,
            threshold,
            include_confidence: true,
            include_spans: true,
            format_results: true,
        }
    }
}
