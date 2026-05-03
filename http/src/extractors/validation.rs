use crate::{
    error::{invalid_json_problem, validation_failed_problem, FieldError, ProblemDetails},
    middleware::trace::TraceId,
};
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::Query;
use forma::Validate;
use serde::de::DeserializeOwned;

/// Wraps a T that implements `Deserialize + forma::Validate`.
/// Rejects the request with 400 and a RFC 7807 JSON body on failure.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = (StatusCode, Json<crate::error::ProblemDetails>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let trace_id = req.extensions().get::<TraceId>().map(|t| t.0.clone());

        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|_| {
            let mut problem = invalid_json_problem();
            if let Some(tid) = &trace_id {
                problem = problem.with_trace_id(tid.clone());
            }
            (StatusCode::BAD_REQUEST, Json(problem))
        })?;

        if let Err(errors) = value.validate() {
            tracing::debug!("Validation failed");
            let mut problem = ProblemDetails::from(errors);
            if let Some(tid) = trace_id {
                problem = problem.with_trace_id(tid);
            }
            return Err((StatusCode::BAD_REQUEST, Json(problem)));
        }

        Ok(ValidatedJson(value))
    }
}

/// Wraps a T that implements `Deserialize + forma::Validate` for query parameters.
/// Rejects the request with 400 and a RFC 7807 JSON body on failure.
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = (StatusCode, Json<crate::error::ProblemDetails>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let trace_id = parts.extensions.get::<TraceId>().map(|t| t.0.clone());

        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                let mut problem =
                    validation_failed_problem().with_field_errors(vec![FieldError {
                        field: "_query".to_string(),
                        message: e.body_text(),
                    }]);
                if let Some(tid) = &trace_id {
                    problem = problem.with_trace_id(tid.clone());
                }
                (StatusCode::BAD_REQUEST, Json(problem))
            })?;

        if let Err(errors) = value.validate() {
            tracing::debug!("Validation failed");
            let mut problem = ProblemDetails::from(errors);
            if let Some(tid) = trace_id {
                problem = problem.with_trace_id(tid);
            }
            return Err((StatusCode::BAD_REQUEST, Json(problem)));
        }

        Ok(ValidatedQuery(value))
    }
}
