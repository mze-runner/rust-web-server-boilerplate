//! Trace ID extractor for request correlation

use crate::middleware::trace::TraceId;
use axum::{extract::FromRequestParts, http::request::Parts};

/// Extractor for TraceId from request extensions
///
/// This extractor always succeeds and returns a trace_id for request correlation.
/// If no trace_id is found in extensions, it returns "unknown" as fallback.
///
/// # Example
/// ```rust,ignore
/// pub async fn handler(
///     ExtractTraceId(trace_id): ExtractTraceId,
/// ) -> impl IntoResponse {
///     tracing::info!(trace_id = %trace_id, "Processing request");
///     "OK"
/// }
/// ```
pub struct ExtractTraceId(pub String);

impl<S> FromRequestParts<S> for ExtractTraceId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible; // Never fails

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let trace_id = parts
            .extensions
            .get::<TraceId>()
            .map(|t| t.0.clone())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(ExtractTraceId(trace_id))
    }
}
