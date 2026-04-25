//! Authenticated caller identity — HTTP layer concern.
//!
//! `Subject` represents the validated `sub` claim from the Bearer token.
//! It is transport-agnostic: the caller may be a human user, service account,
//! or API key holder. The domain layer has no dependency on this type.

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Subject(pub Uuid);
