//! HTTP extractors for request data validation and processing
//!
//! This module provides custom extractors that implement FromRequest/FromRequestParts
//! to extract and validate data from HTTP requests in a type-safe manner.

pub mod auth;
pub mod trace;
pub mod validation;

// Re-export all extractors for easy importing
pub use auth::AuthenticatedUser;
pub use trace::ExtractTraceId;
pub use validation::ValidatedJson;
