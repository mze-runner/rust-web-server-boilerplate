//! # forma
//!
//! Schema-driven validation with optional normalization, inspired by [Joi](https://joi.dev/).
//!
//! ## Two-phase pipeline
//!
//! 1. **Normalize** — called in `Deserialize` impls; transforms raw input into canonical form
//!    (trim whitespace, lowercase, etc.)
//! 2. **Validate** — called via the [`Validate`] trait; asserts constraints on normalized data
//!
//! ## Entry points
//!
//! ```rust
//! use forma::{schema, StringSchema, NumberSchema};
//!
//! const SUBJECT: StringSchema = schema::string().trim().required().max(256);
//! const LIMIT:   NumberSchema = schema::number().max(100);
//! // const DUE: DateTimeSchema = schema::datetime().required().require_utc(); // feature = "datetime"
//! ```
//!
//! ## Error codes
//!
//! Violations use dot-namespaced codes — `"string.required"`, `"string.max"`, `"number.min"`,
//! `"datetime.utc"` — so API clients can key on them without parsing messages.

pub mod errors;
pub mod rules;
pub mod schema;

#[cfg(feature = "datetime")]
#[doc(hidden)]
pub mod __private {
    pub use chrono;
}

/// Generate `Deserialize` (with normalization) and `Validate` impls for a string newtype.
///
/// The struct itself is defined by the caller — add `#[derive(Debug)]`, extra methods,
/// or `Default` as needed. The macro contributes the two boilerplate impls only.
///
/// ```rust
/// use forma::string_field;
///
/// #[derive(Debug)]
/// pub struct SubjectField(pub Option<String>);
/// string_field!(SubjectField, string().trim().required().min(1).max(256));
/// ```
#[macro_export]
macro_rules! string_field {
    ($type:ty, $($rule:tt)*) => {
        const _: () = {
            const __RULE: $crate::StringSchema = {
                use $crate::schema::*;
                $($rule)*
            };

            impl<'de> ::serde::Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    let opt = <::core::option::Option<::std::string::String>
                        as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                    ::core::result::Result::Ok(Self(__RULE.normalize(opt)))
                }
            }

            impl $crate::Validate for $type {
                fn validate(&self) -> ::core::result::Result<(), $crate::FieldErrors> {
                    __RULE.validate(&self.0)
                }
            }
        };
    };
}

pub use errors::{FieldErrors, RuleViolation, ViolationContext};
pub use rules::{NumberSchema, StringSchema};
pub use schema::{number, string};

#[cfg(feature = "datetime")]
pub use rules::DateTimeSchema;
#[cfg(feature = "datetime")]
pub use schema::datetime;

/// Generate `Deserialize` and `Validate` impls for an integer newtype.
///
/// The struct itself is defined by the caller. Specify the primitive inner type so the
/// `Deserialize` impl can delegate to it directly. The `Validate` impl widens the value
/// to `i64` before passing it to the schema — safe for all integer types up to `i64::MAX`.
///
/// ```rust
/// use forma::number_field;
///
/// #[derive(Debug)]
/// pub struct LimitField(pub u32);
/// number_field!(LimitField, u32, number().max(100));
/// ```
#[macro_export]
macro_rules! number_field {
    ($type:ty, $primitive:ty, $($rule:tt)*) => {
        const _: () = {
            const __RULE: $crate::NumberSchema = {
                use $crate::schema::*;
                $($rule)*
            };

            impl<'de> ::serde::Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    let val =
                        <$primitive as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                    ::core::result::Result::Ok(Self(val))
                }
            }

            impl $crate::Validate for $type {
                fn validate(&self) -> ::core::result::Result<(), $crate::FieldErrors> {
                    __RULE.validate(self.0 as i64)
                }
            }
        };
    };
}

/// Generate `Deserialize` and `Validate` impls for an `Option<DateTime<FixedOffset>>` newtype.
///
/// Requires the `datetime` feature.
///
/// ```rust
/// use forma::datetime_field;
///
/// #[derive(Debug, Default)]
/// pub struct DueAtField(pub Option<chrono::DateTime<chrono::FixedOffset>>);
/// datetime_field!(DueAtField, datetime().required().utc());
/// ```
#[cfg(feature = "datetime")]
#[macro_export]
macro_rules! datetime_field {
    ($type:ty, $($rule:tt)*) => {
        const _: () = {
            const __RULE: $crate::DateTimeSchema = {
                use $crate::schema::*;
                $($rule)*
            };

            impl<'de> ::serde::Deserialize<'de> for $type {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    let val = <::core::option::Option<
                        $crate::__private::chrono::DateTime<
                            $crate::__private::chrono::FixedOffset,
                        >,
                    > as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
                    ::core::result::Result::Ok(Self(val))
                }
            }

            impl $crate::Validate for $type {
                fn validate(&self) -> ::core::result::Result<(), $crate::FieldErrors> {
                    __RULE.validate(&self.0)
                }
            }
        };
    };
}

/// Implemented by field types and DTOs. Called by `ValidatedJson` after deserialization.
///
/// Field types validate their own constraints; DTOs aggregate field results via
/// [`FieldErrors::merge`].
///
/// ```rust,no_run
/// use forma::{Validate, FieldErrors};
///
/// # struct CreateTaskRequest { subject: SubjectField, description: DescField }
/// # struct SubjectField; impl Validate for SubjectField { fn validate(&self) -> Result<(), FieldErrors> { Ok(()) } }
/// # struct DescField; impl Validate for DescField { fn validate(&self) -> Result<(), FieldErrors> { Ok(()) } }
/// impl Validate for CreateTaskRequest {
///     fn validate(&self) -> Result<(), FieldErrors> {
///         let mut e = FieldErrors::default();
///         e.merge("subject",     self.subject.validate());
///         e.merge("description", self.description.validate());
///         e.finish()
///     }
/// }
/// ```
pub trait Validate {
    fn validate(&self) -> Result<(), FieldErrors>;
}
