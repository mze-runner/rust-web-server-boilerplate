use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use servicez_application::error::AppError;

/// RFC 7807 Problem Details (minimal MVP version)
#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    /// URI reference that identifies the problem type
    #[serde(rename = "type")]
    pub type_uri: String,

    /// Short, human-readable summary
    pub title: String,

    /// HTTP status code
    pub status: u16,

    /// Human-readable explanation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// Stable machine-readable error code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Field-level validation errors (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,

    // TODO: Add when implementing trace_id middleware
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    // TODO: Add when implementing request tracking
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub instance: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl ProblemDetails {
    /// Create a new ProblemDetails with required fields
    pub fn new(status: StatusCode, title: impl Into<String>) -> Self {
        Self {
            type_uri: "about:blank".to_string(),
            title: title.into(),
            status: status.as_u16(),
            detail: None,
            code: None,
            errors: None,
            trace_id: None,
        }
    }

    /// Builder: add detail message
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Builder: add error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Builder: add field-level errors
    pub fn with_field_errors(mut self, errors: Vec<FieldError>) -> Self {
        self.errors = Some(errors);
        self
    }

    // TODO: Add when implementing trace_id
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (
            status,
            [("content-type", "application/problem+json")],
            Json(self),
        )
            .into_response()
    }
}

/// Convert AppError to ProblemDetails with appropriate HTTP status
impl From<AppError> for ProblemDetails {
    fn from(err: AppError) -> Self {
        match err {
            // ========== Domain Errors (Pattern Match Inner) ==========
            AppError::Domain(domain_err) => {
                // Access DomainError through pattern matching, no direct import needed
                use servicez_domain::error::DomainError;
                match domain_err {
                    DomainError::NotFound {
                        resource_type,
                        identifier,
                    } => ProblemDetails::new(StatusCode::NOT_FOUND, "Not Found")
                        .with_code("domain.not_found")
                        .with_detail(format!("{} not found: {}", resource_type, identifier)),
                    DomainError::AlreadyExists {
                        resource_type,
                        identifier,
                    } => ProblemDetails::new(StatusCode::CONFLICT, "Already Exists")
                        .with_code("domain.already_exists")
                        .with_detail(format!("{} already exists: {}", resource_type, identifier)),
                    DomainError::Forbidden { reason } => {
                        ProblemDetails::new(StatusCode::FORBIDDEN, "Forbidden")
                            .with_code("domain.forbidden")
                            .with_detail(reason)
                    }
                    DomainError::InvariantViolation { message } => {
                        ProblemDetails::new(StatusCode::BAD_REQUEST, "Invariant Violation")
                            .with_code("domain.invariant_violation")
                            .with_detail(message)
                    }
                    DomainError::BusinessRuleViolation { rule } => {
                        ProblemDetails::new(StatusCode::BAD_REQUEST, "Business Rule Violation")
                            .with_code("domain.business_rule")
                            .with_detail(rule)
                    }
                    DomainError::InvalidStateTransition { from, to, reason } => {
                        ProblemDetails::new(StatusCode::CONFLICT, "Invalid State Transition")
                            .with_code("domain.invalid_transition")
                            .with_detail(format!(
                                "Cannot transition from {} to {}: {}",
                                from, to, reason
                            ))
                    }
                    DomainError::ImmutableResource => ProblemDetails::new(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Unprocessable Entity",
                    )
                    .with_code("domain.immutable_resource")
                    .with_detail("cannot mutate a task in Done or Cancelled status".to_owned()),
                    DomainError::Repository { .. } => {
                        // Infrastructure details must not reach the client.
                        ProblemDetails::new(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal Server Error",
                        )
                        .with_code("internal_error")
                        .with_detail("An unexpected error occurred. Please try again later.")
                    }
                }
            }

            // ========== Validation Errors ==========
            AppError::InvalidInput(msg) => {
                ProblemDetails::new(StatusCode::BAD_REQUEST, "Invalid Input")
                    .with_code("invalid_input")
                    .with_detail(msg)
            }

            AppError::ValidationFailed { field, message } => {
                ProblemDetails::new(StatusCode::BAD_REQUEST, "Validation Failed")
                    .with_code("validation_failed")
                    .with_field_errors(vec![FieldError { field, message }])
            }

            // ========== Request-Level Errors ==========
            AppError::BadRequest(msg) => {
                ProblemDetails::new(StatusCode::BAD_REQUEST, "Bad Request")
                    .with_code("bad_request")
                    .with_detail(msg)
            }

            AppError::Unauthorized(msg) => {
                ProblemDetails::new(StatusCode::UNAUTHORIZED, "Unauthorized")
                    .with_code("unauthorized")
                    .with_detail(msg)
            }

            AppError::NotFound { resource_type } => {
                ProblemDetails::new(StatusCode::NOT_FOUND, "Not Found")
                    .with_code("not_found")
                    .with_detail(format!("Resource not found: {}", resource_type))
            }

            AppError::Conflict(msg) => ProblemDetails::new(StatusCode::CONFLICT, "Conflict")
                .with_code("conflict")
                .with_detail(msg),

            AppError::UnprocessableEntity(detail) => {
                ProblemDetails::new(StatusCode::UNPROCESSABLE_ENTITY, "Unprocessable entity")
                    .with_code("urn:problem:unprocessable")
                    .with_detail(detail)
            }

            // ========== Repository/Infrastructure Errors ==========
            AppError::Repository {
                operation, details, ..
            } => {
                tracing::error!(
                    operation = %operation,
                    details = %details,
                    "Repository error occurred"
                );
                ProblemDetails::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .with_code("repository_error")
                    .with_detail("An unexpected error occurred. Please try again later.")
            }

            AppError::Transaction(msg) => {
                tracing::error!(error = %msg, "Transaction error");
                ProblemDetails::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .with_code("transaction_error")
                    .with_detail("An unexpected error occurred. Please try again later.")
            }

            AppError::ExternalService {
                service, details, ..
            } => {
                tracing::error!(
                    service = %service,
                    details = %details,
                    "External service error"
                );
                ProblemDetails::new(StatusCode::BAD_GATEWAY, "External Service Error")
                    .with_code("external_service_error")
                    .with_detail(format!("Unable to communicate with {}", service))
            }

            AppError::Timeout {
                operation,
                duration_ms,
            } => {
                tracing::warn!(
                    operation = %operation,
                    duration_ms = %duration_ms,
                    "Operation timed out"
                );
                ProblemDetails::new(StatusCode::GATEWAY_TIMEOUT, "Request Timeout")
                    .with_code("timeout")
                    .with_detail(format!("Operation timed out after {}ms", duration_ms))
            }

            AppError::InternalServerError(msg) => {
                tracing::error!(error = %msg, "Internal server error");
                ProblemDetails::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .with_code("internal_error")
                    .with_detail("An unexpected error occurred. Please try again later.")
            }

            AppError::NotImplemented(feature) => {
                ProblemDetails::new(StatusCode::NOT_IMPLEMENTED, "Not Implemented")
                    .with_code("not_implemented")
                    .with_detail(format!("Feature not yet implemented: {}", feature))
            }
        }
    }
}

// /// Implement IntoResponse for AppError so handlers can return Result<T, AppError>
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         let problem: ProblemDetails = self.into();
//         problem.into_response()
//     }
// }

// /// Helper method to convert AppError to Response with trace_id
// impl AppError {
//     /// Convert AppError to HTTP Response with trace_id included in ProblemDetails
//     /// # Example
//     /// ```
//     /// return Err(AppError::InternalServerError("DB error".to_string())
//     ///     .into_response_with_trace_id(trace_id));
//     /// ```
//     pub fn into_response_with_trace_id(self, trace_id: impl Into<String>) -> Response {
//         let mut problem: ProblemDetails = self.into();
//         problem = problem.with_trace_id(trace_id);
//         problem.into_response()
//     }
// }

impl From<forma::FieldErrors> for ProblemDetails {
    fn from(errors: forma::FieldErrors) -> Self {
        let field_errors: Vec<FieldError> = errors
            .into_inner()
            .into_iter()
            .map(|(path, v)| FieldError {
                field: path,
                message: v.message,
            })
            .collect();
        let mut problem = validation_failed_problem();
        if !field_errors.is_empty() {
            problem = problem.with_field_errors(field_errors);
        }
        problem
    }
}

pub fn invalid_json_problem() -> ProblemDetails {
    ProblemDetails::new(StatusCode::BAD_REQUEST, "Invalid JSON")
        .with_code("invalid_json")
        .with_detail("Request body must be valid JSON")
}

pub fn validation_failed_problem() -> ProblemDetails {
    ProblemDetails::new(StatusCode::BAD_REQUEST, "Validation Failed").with_code("validation_failed")
}

pub fn app_error_to_response(error: AppError, trace_id: impl Into<String>) -> Response {
    let mut problem: ProblemDetails = error.into();
    problem = problem.with_trace_id(trace_id);
    problem.into_response()
}
