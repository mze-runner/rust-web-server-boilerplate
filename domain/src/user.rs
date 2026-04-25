//! User domain model.
//!
//! `User` represents a participant in the system. Users are pre-seeded in the
//! database — create/manage flows are out of scope for this service.

use chrono::{DateTime, Utc};

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// Newtypes
// ---------------------------------------------------------------------------

crate::id::uuid_newtype!(UserId);

/// Display name — non-empty, max 30 characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    pub fn new(s: &str) -> Result<Self, DomainError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(DomainError::InvariantViolation {
                message: "username must not be empty".to_string(),
            });
        }
        if s.len() > 30 {
            return Err(DomainError::InvariantViolation {
                message: format!("username must be at most 30 characters, got {}", s.len()),
            });
        }
        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Email address — basic structural sanity (non-empty, contains exactly one `@`).
/// Full RFC 5322 validation is enforced at the HTTP boundary via `garde`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(s: &str) -> Result<Self, DomainError> {
        let s = s.trim();
        let (local, domain) = s
            .split_once('@')
            .ok_or_else(|| DomainError::InvariantViolation {
                message: "email must contain exactly one '@'".to_string(),
            })?;
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return Err(DomainError::InvariantViolation {
                message: "email must have a non-empty local part, exactly one '@', and a non-empty domain".to_string(),
            });
        }
        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ---------------------------------------------------------------------------
// Aggregate
// ---------------------------------------------------------------------------

/// A pre-seeded system participant. Immutable from the service's perspective.
#[derive(Clone, Debug)]
pub struct User {
    id: UserId,
    username: Username,
    email: Email,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    /// Reconstitute a `User` from persisted data (adapter layer only).
    pub fn from_row(
        id: UserId,
        username: Username,
        email: Email,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
