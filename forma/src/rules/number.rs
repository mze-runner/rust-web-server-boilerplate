use crate::errors::{FieldErrors, RuleViolation, ViolationContext};

/// Fluent, const-constructible schema for integer fields.
///
/// Bounds are expressed as `i64`; field types cast their concrete type before calling
/// [`validate`](NumberSchema::validate). `u32::MAX` fits safely in `i64`.
///
/// ```rust
/// use forma::{schema, NumberSchema};
///
/// const RULE: NumberSchema = schema::number().max(100);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NumberSchema {
    min: Option<i64>,
    max: Option<i64>,
    custom: Option<fn(i64) -> Option<RuleViolation>>,
}

impl NumberSchema {
    pub const fn new() -> Self {
        Self {
            min: None,
            max: None,
            custom: None,
        }
    }

    /// Inclusive lower bound.
    pub const fn min(mut self, n: i64) -> Self {
        self.min = Some(n);
        self
    }

    /// Inclusive upper bound.
    pub const fn max(mut self, n: i64) -> Self {
        self.max = Some(n);
        self
    }

    /// Attach a service-specific constraint via a plain function pointer (const-compatible).
    pub const fn custom(mut self, f: fn(i64) -> Option<RuleViolation>) -> Self {
        self.custom = Some(f);
        self
    }

    /// Assert all constraints. Call with `value as i64` from field `validate` impls.
    pub fn validate(&self, value: i64) -> Result<(), FieldErrors> {
        let mut errors = FieldErrors::default();

        if let Some(min) = self.min {
            if value < min {
                errors.push(RuleViolation {
                    code: "number.min",
                    message: format!("must be at least {min}"),
                    context: ViolationContext::Limit(min as usize),
                });
            }
        }

        if let Some(max) = self.max {
            if value > max {
                errors.push(RuleViolation {
                    code: "number.max",
                    message: format!("must be at most {max}"),
                    context: ViolationContext::Limit(max as usize),
                });
            }
        }

        if let Some(f) = self.custom {
            if let Some(violation) = f(value) {
                errors.push(violation);
            }
        }

        errors.finish()
    }
}
