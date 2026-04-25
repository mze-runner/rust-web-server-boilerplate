use uuid::Uuid;

use crate::{auth::Subject, error::app_error_to_response, middleware::trace::TraceId};
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use servicez_application::error::AppError;
use tracing;

/// Authentication middleware — validates Bearer token and stores the caller's
/// `Subject` (sub claim) in request extensions for downstream extractors.
pub async fn authenticate(mut req: Request, next: Next) -> Result<Response, Response> {
    let trace_id = req
        .extensions()
        .get::<TraceId>()
        .map(|t| t.0.as_str())
        .unwrap_or("unknown");

    let auth_header = match extract_bearer_token(req.headers()) {
        Some(token) => token,
        None => {
            tracing::warn!(trace_id = %trace_id, "auth middleware: no Authorization header");
            return Err(app_error_to_response(
                AppError::Unauthorized("Missing Authorization header".to_string()),
                trace_id,
            ));
        }
    };

    let sub = match validate_token(&auth_header, trace_id).await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::warn!(trace_id = %trace_id, "auth middleware: token validation failed");
            return Err(app_error_to_response(e, trace_id));
        }
    };

    tracing::info!(trace_id = %trace_id, sub = %sub, "auth middleware: authenticated");

    req.extensions_mut().insert(Subject(sub));
    Ok(next.run(req).await)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("authorization")?;
    let s = value.to_str().ok()?;
    s.strip_prefix("Bearer ").map(|t| t.to_owned())
}

/// Placeholder token validation — always succeeds for non-empty, non-"invalid" tokens.
/// Replace with real JWT verification when auth is implemented.
/// The token must be a valid UUID (user ID) — this is the boundary where identity is established.
async fn validate_token(token: &str, trace_id: &str) -> Result<Uuid, AppError> {
    tracing::debug!(trace_id = %trace_id, "auth middleware: validating token (placeholder)");

    if token.is_empty() {
        return Err(AppError::Unauthorized("Empty token".to_string()));
    }
    if token == "invalid" || token == "expired" {
        return Err(AppError::Unauthorized("Token rejected".to_string()));
    }

    Uuid::parse_str(token).map_err(|_| AppError::Unauthorized("Token is not a valid user ID".to_string()))
}
