use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

/// Wrapper type for trace_id stored in request extensions.
#[derive(Debug, Clone)]
pub struct TraceId(pub String);

impl TraceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Extracts `x-trace-id` from the request header or generates a new UUID v4.
/// Stores the value in request extensions and echoes it on the response.
pub async fn trace_id(mut request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-trace-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request.extensions_mut().insert(TraceId(id.clone()));
    tracing::debug!(trace_id = %id, "trace id assigned");

    let mut response = next.run(request).await;

    // Echo trace id on response — skip silently if the value contains invalid header bytes.
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert("x-trace-id", value);
    } else {
        tracing::warn!(trace_id = %id, "trace id contains invalid header bytes, omitting from response");
    }

    response
}
