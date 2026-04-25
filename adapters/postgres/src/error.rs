//! Postgres adapter error mappings
//!
//! Translates SQLx errors into domain-meaningful errors

use servicez_domain::error::DomainError;

/// Map SQLx errors to DomainError with semantic meaning
pub fn map_sqlx_error(err: sqlx::Error, context: &str) -> DomainError {
    match err {
        // Row not found → NotFound (domain semantic)
        sqlx::Error::RowNotFound => DomainError::NotFound {
            resource_type: "Resource".to_string(),
            identifier: context.to_string(),
        },

        // Database constraint violation → AlreadyExists
        sqlx::Error::Database(ref db_err) => {
            // Unique constraint violation (PostgreSQL error code 23505)
            if let Some(code) = db_err.code() {
                if code == "23505" {
                    return DomainError::AlreadyExists {
                        resource_type: "Resource".to_string(),
                        identifier: db_err.constraint().unwrap_or("unknown").to_string(),
                    };
                }
            }

            // Foreign key violation or other constraint
            DomainError::InvariantViolation {
                message: format!("Database constraint violation: {}", db_err),
            }
        }

        // Connection/pool errors → InvariantViolation (infrastructure issue)
        sqlx::Error::PoolTimedOut => DomainError::InvariantViolation {
            message: "Database connection pool timeout".to_string(),
        },

        sqlx::Error::PoolClosed => DomainError::InvariantViolation {
            message: "Database connection pool closed".to_string(),
        },

        // All other errors → generic infrastructure failure
        _ => DomainError::InvariantViolation {
            message: format!("Database error during {}: {}", context, err),
        },
    }
}

/// Helper: Map Result<T, sqlx::Error> to Result<T, DomainError>
pub trait SqlxResultExt<T> {
    fn map_err_domain(self, context: &str) -> Result<T, DomainError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn map_err_domain(self, context: &str) -> Result<T, DomainError> {
        self.map_err(|e| map_sqlx_error(e, context))
    }
}
