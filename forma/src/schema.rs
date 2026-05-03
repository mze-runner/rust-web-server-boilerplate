use crate::rules::{NumberSchema, StringSchema};

#[cfg(feature = "datetime")]
use crate::rules::DateTimeSchema;

/// Begin a string field schema.
///
/// ```rust
/// use forma::{schema, StringSchema};
/// const RULE: StringSchema = schema::string().trim().required().max(256);
/// ```
pub const fn string() -> StringSchema {
    StringSchema::new()
}

/// Begin an integer field schema.
///
/// ```rust
/// use forma::{schema, NumberSchema};
/// const RULE: NumberSchema = schema::number().max(100);
/// ```
pub const fn number() -> NumberSchema {
    NumberSchema::new()
}

/// Begin a datetime field schema (requires the `datetime` feature).
///
/// ```rust
/// use forma::{schema, DateTimeSchema};
/// const RULE: DateTimeSchema = schema::datetime().required().utc();
/// ```
#[cfg(feature = "datetime")]
pub const fn datetime() -> DateTimeSchema {
    DateTimeSchema::new()
}
