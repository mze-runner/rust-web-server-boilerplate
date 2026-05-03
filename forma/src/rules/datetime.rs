use chrono::{DateTime, FixedOffset};

use crate::errors::{FieldErrors, RuleViolation};

/// Fluent, const-constructible schema for `Option<DateTime<FixedOffset>>` fields.
///
/// Normalization is not applicable here — parsing from a string to `DateTime` is serde's
/// responsibility. This schema handles post-parse constraints only.
///
/// ```rust
/// use forma::{schema, DateTimeSchema};
///
/// const RULE: DateTimeSchema = schema::datetime().required().utc();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DateTimeSchema {
    required: bool,
    require_utc: bool,
    custom: Option<fn(&DateTime<FixedOffset>) -> Option<RuleViolation>>,
}

impl DateTimeSchema {
    pub const fn new() -> Self {
        Self {
            required: false,
            require_utc: false,
            custom: None,
        }
    }

    /// Reject `None` (field was absent in the request).
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Reject any non-UTC timezone offset (offset must be exactly `+00:00`).
    pub const fn utc(mut self) -> Self {
        self.require_utc = true;
        self
    }

    /// Attach a service-specific constraint via a plain function pointer (const-compatible).
    pub const fn custom(mut self, f: fn(&DateTime<FixedOffset>) -> Option<RuleViolation>) -> Self {
        self.custom = Some(f);
        self
    }

    /// Assert all constraints against an already-parsed value.
    pub fn validate(&self, value: &Option<DateTime<FixedOffset>>) -> Result<(), FieldErrors> {
        let mut errors = FieldErrors::default();

        let Some(dt) = value.as_ref() else {
            if self.required {
                errors.push(RuleViolation::new("datetime.required", "must be present"));
            }
            return errors.finish();
        };

        if self.require_utc && dt.offset().local_minus_utc() != 0 {
            errors.push(RuleViolation::new(
                "datetime.utc",
                "must be in UTC timezone (+00:00)",
            ));
        }

        if let Some(f) = self.custom {
            if let Some(violation) = f(dt) {
                errors.push(violation);
            }
        }

        errors.finish()
    }
}
