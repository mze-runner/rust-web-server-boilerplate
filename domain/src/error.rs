//! Domain layer errors - pure business logic failures
//!
//! These errors contain NO infrastructure concerns (no HTTP, no database types).
//! They represent violations of business rules and invariants.

/// Domain-level errors representing business rule violations
///
/// This type is completely infrastructure-agnostic and contains
/// only pure business logic errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DomainError {
    /// Business invariant violation
    #[error("Invariant violation: {message}")]
    InvariantViolation { message: String },

    /// Resource not found (business-level)
    #[error("{resource_type} not found: {identifier}")]
    NotFound {
        resource_type: String,
        identifier: String,
    },

    /// Business rule violation
    #[error("Business rule violation: {rule}")]
    BusinessRuleViolation { rule: String },

    /// Duplicate resource (uniqueness constraint)
    #[error("Duplicate {resource_type}: {identifier}")]
    AlreadyExists {
        resource_type: String,
        identifier: String,
    },

    /// Authorization failure (domain-level)
    #[error("Authorization failed: {reason}")]
    Forbidden { reason: String },

    /// Invalid state transition
    #[error("Invalid state transition from {from} to {to}: {reason}")]
    InvalidStateTransition {
        from: String,
        to: String,
        reason: String,
    },

    /// Resource is immutable and cannot be modified.
    #[error("resource is immutable and cannot be modified")]
    ImmutableResource,

    /// Infrastructure / repository failure — distinct from business rule violations.
    /// Carries the operation name and a sanitized message; raw driver errors
    /// must not be exposed beyond the adapter layer.
    #[error("repository error: {operation} failed: {message}")]
    Repository { operation: String, message: String },
}
