//! Authentication extractors for protected routes.

use crate::auth::Subject;
use crate::error::app_error_to_response;
use crate::middleware::trace::TraceId;
use axum::{extract::FromRequestParts, http::request::Parts, response::Response};
use servicez_application::error::AppError;

/// Extractor for the authenticated caller's identity.
///
/// Fails with 401 if the auth middleware did not run or the token was rejected.
///
/// ```rust
/// pub async fn handler(
///     AuthenticatedUser(subject): AuthenticatedUser,
/// ) -> impl IntoResponse {
///     format!("caller: {}", subject.as_str())
/// }
/// ```
pub struct AuthenticatedUser(pub Subject);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let subject = parts.extensions.get::<Subject>().ok_or_else(|| {
            let trace_id = parts
                .extensions
                .get::<TraceId>()
                .map(|t| t.0.as_str())
                .unwrap_or("unknown");
            tracing::error!(
                trace_id = %trace_id,
                "AuthenticatedUser extractor: no Subject in extensions \
                 — authenticate middleware not applied to this route"
            );
            app_error_to_response(
                AppError::Unauthorized("Not authenticated".to_string()),
                trace_id,
            )
        })?;

        Ok(AuthenticatedUser(subject.clone()))
    }
}
