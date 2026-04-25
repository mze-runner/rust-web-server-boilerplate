//! Application layer errors - use-case orchestration failures
//!
//! This layer wraps domain errors and maps infrastructure errors
//! to business-meaningful error types.

use servicez_domain::error::DomainError;

/// Application service layer errors
///
/// Handles use-case orchestration failures and maps infrastructure
/// errors to domain semantics. This is the boundary where infrastructure
/// concerns are translated into business-meaningful errors.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // ========== Domain Errors (Wrapped) ==========
    /// Business logic error from domain layer
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    // ========== Validation Errors ==========
    /// Invalid input (static validation failure)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Field-level validation failure (single field)
    #[error("Validation failed for field '{field}': {message}")]
    ValidationFailed { field: String, message: String },

    // ========== Request-Level Errors ==========
    /// Generic bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Authentication failure
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Resource not found (application-level)
    #[error("Not found: {resource_type}")]
    NotFound { resource_type: String },

    /// Conflict (e.g., concurrent modification)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Business rule violation that makes the request unprocessable (HTTP 422)
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    // ========== Repository/Persistence Errors ==========
    /// Database operation failed
    #[error("Repository error: {operation} - {details}")]
    Repository {
        operation: String,
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Transaction management error
    #[error("Transaction error: {0}")]
    Transaction(String),

    // ========== External Service Errors ==========
    /// External gateway failure (HTTP, gRPC, etc.)
    #[error("External service error: {service} - {details}")]
    ExternalService {
        service: String,
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // ========== Infrastructure Errors ==========
    /// Generic internal error (should be rare)
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Timeout error
    #[error("Operation timed out after {duration_ms}ms: {operation}")]
    Timeout { operation: String, duration_ms: u64 },
}
