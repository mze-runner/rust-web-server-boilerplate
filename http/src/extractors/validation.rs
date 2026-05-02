//! Request validation extractors using garde validation library

use crate::{
    error::{invalid_json_problem, validation_failed_problem, FieldError},
    middleware::trace::TraceId,
};
use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::Query;
use garde::Validate;
use serde::de::DeserializeOwned;

/// Wraps a T that implements Deserialize + garde::Validate.
/// If validation fails, rejects the request with 400 and a JSON body.
pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
    <T as Validate>::Context: Default,
{
    type Rejection = (StatusCode, Json<crate::error::ProblemDetails>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract trace_id from request extensions BEFORE consuming the request
        let trace_id = req.extensions().get::<TraceId>().map(|t| t.0.clone());

        // Parse JSON first
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|_| {
            let mut problem = invalid_json_problem();
            if let Some(tid) = &trace_id {
                problem = problem.with_trace_id(tid.clone());
            }
            (StatusCode::BAD_REQUEST, Json(problem))
        })?;

        if let Err(report) = value.validate() {
            tracing::debug!(?report, "Validation failed");
            let mut problem = validation_failed_problem()
                .with_field_errors(garde_report_to_field_errors(&report));
            if let Some(tid) = trace_id {
                problem = problem.with_trace_id(tid);
            }
            return Err((StatusCode::BAD_REQUEST, Json(problem)));
        }

        Ok(ValidatedJson(value))
    }
}

/// Wraps a T that implements Deserialize + garde::Validate for query parameters.
/// If parsing or validation fails, rejects the request with 400 and a JSON body.
pub struct ValidatedQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
    <T as Validate>::Context: Default,
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

        if let Err(report) = value.validate() {
            tracing::debug!(?report, "Validation failed");
            let mut problem = validation_failed_problem()
                .with_field_errors(garde_report_to_field_errors(&report));
            if let Some(tid) = trace_id {
                problem = problem.with_trace_id(tid);
            }
            return Err((StatusCode::BAD_REQUEST, Json(problem)));
        }

        Ok(ValidatedQuery(value))
    }
}

fn garde_report_to_field_errors(report: &garde::Report) -> Vec<FieldError> {
    let mut errors: Vec<FieldError> = report
        .iter()
        .map(|(path, error)| FieldError {
            field: path.to_string(),
            message: error.to_string(),
        })
        .collect();

    if errors.is_empty() {
        errors.push(FieldError {
            field: "_schema".to_string(),
            message: report.to_string(),
        });
    }

    errors
}
