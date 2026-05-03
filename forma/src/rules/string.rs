use crate::errors::{FieldErrors, RuleViolation};

/// Fluent, const-constructible schema for `Option<String>` fields.
///
/// Normalization (`.trim()`) runs in phase 1 via [`normalize`](StringSchema::normalize).
/// Constraint checks run in phase 2 via [`validate`](StringSchema::validate).
/// [`process`](StringSchema::process) combines both into one call.
///
/// ```rust
/// use forma::{schema, StringSchema};
///
/// const RULE: StringSchema = schema::string().trim().required().max(256);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StringSchema {
    trim: bool,
    lowercase: bool,
    uppercase: bool,
    required: bool,
    min: Option<usize>,
    max: Option<usize>,
    alphanum: bool,
    email: bool,
    custom: Option<fn(&str) -> Option<RuleViolation>>,
}

impl StringSchema {
    pub const fn new() -> Self {
        Self {
            trim: false,
            lowercase: false,
            uppercase: false,
            required: false,
            min: None,
            max: None,
            alphanum: false,
            email: false,
            custom: None,
        }
    }

    /// Strip leading and trailing whitespace before validation.
    pub const fn trim(mut self) -> Self {
        self.trim = true;
        self
    }

    /// Convert the entire string to lowercase before validation.
    pub const fn lowercase(mut self) -> Self {
        self.lowercase = true;
        self
    }

    /// Convert the entire string to uppercase before validation.
    pub const fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    /// Reject `None` and empty (or whitespace-only, when combined with `.trim()`) strings.
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Minimum character count (Unicode code points, not bytes).
    pub const fn min(mut self, n: usize) -> Self {
        self.min = Some(n);
        self
    }

    /// Maximum character count (Unicode code points, not bytes).
    pub const fn max(mut self, n: usize) -> Self {
        self.max = Some(n);
        self
    }

    /// Restrict to ASCII alphanumeric characters only (`a–z`, `A–Z`, `0–9`).
    /// Spaces, punctuation, and non-ASCII characters are rejected.
    pub const fn alphanum(mut self) -> Self {
        self.alphanum = true;
        self
    }

    /// Require a structurally valid email address (`local@domain.tld`).
    ///
    /// Checks: non-empty local part, exactly one `@`, domain contains at least
    /// one `.` with non-empty labels on both sides. Not full RFC 5321 compliance —
    /// use a confirmation flow for authoritative verification.
    pub const fn email(mut self) -> Self {
        self.email = true;
        self
    }

    /// Attach a service-specific constraint via a plain function pointer (const-compatible).
    /// Return `Some(RuleViolation)` to signal failure, `None` to pass.
    pub const fn custom(mut self, f: fn(&str) -> Option<RuleViolation>) -> Self {
        self.custom = Some(f);
        self
    }

    // ── Phase 1: normalization ────────────────────────────────────────────────

    /// Return a canonicalized copy of `raw`.
    ///
    /// - `.trim()` — strips leading/trailing whitespace.
    /// - `.email()` — implies trim, then lowercases the domain label.
    ///
    /// No-op when neither flag is set.
    pub fn normalize(&self, raw: Option<String>) -> Option<String> {
        if !self.trim && !self.lowercase && !self.uppercase {
            return raw;
        }
        raw.map(|s| {
            let s = if self.trim { s.trim().to_owned() } else { s };
            if self.lowercase {
                s.to_lowercase()
            } else if self.uppercase {
                s.to_uppercase()
            } else {
                s
            }
        })
    }

    // ── Phase 2: validation ───────────────────────────────────────────────────

    /// Assert all constraints against an already-normalized value.
    /// Length is measured in Unicode code points, not bytes.
    pub fn validate(&self, value: &Option<String>) -> Result<(), FieldErrors> {
        let mut errors = FieldErrors::default();

        let Some(s) = value.as_deref() else {
            if self.required {
                errors.push(RuleViolation::new("string.required", "must not be blank"));
            }
            return errors.finish();
        };

        if self.required && s.is_empty() {
            errors.push(RuleViolation::new("string.required", "must not be blank"));
            // early exit: length/custom checks on an absent value are meaningless
            return errors.finish();
        }

        let char_count = s.chars().count();

        if let Some(min) = self.min {
            if char_count < min {
                errors.push(RuleViolation::with_limit(
                    "string.min",
                    format!("must be at least {min} character(s)"),
                    min,
                ));
            }
        }

        if let Some(max) = self.max {
            if char_count > max {
                errors.push(RuleViolation::with_limit(
                    "string.max",
                    format!("must be at most {max} character(s)"),
                    max,
                ));
            }
        }

        if self.alphanum && !s.chars().all(|c| c.is_ascii_alphanumeric()) {
            errors.push(RuleViolation::new(
                "string.alphanum",
                "must contain only alphanumeric characters (a–z, A–Z, 0–9)",
            ));
        }

        if self.email {
            if let Some(violation) = Self::check_email(s) {
                errors.push(violation);
            }
        }

        if let Some(f) = self.custom {
            if let Some(violation) = f(s) {
                errors.push(violation);
            }
        }

        errors.finish()
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn check_email(s: &str) -> Option<RuleViolation> {
        let at_count = s.chars().filter(|&c| c == '@').count();
        match at_count {
            0 => return Some(RuleViolation::new("string.email", "must contain '@'")),
            1 => {}
            _ => {
                return Some(RuleViolation::new(
                    "string.email",
                    "must contain exactly one '@'",
                ))
            }
        }
        let (local, domain) = s.split_once('@').unwrap();
        if local.is_empty() {
            return Some(RuleViolation::new(
                "string.email",
                "local part must not be empty",
            ));
        }
        if domain.is_empty() {
            return Some(RuleViolation::new(
                "string.email",
                "domain must not be empty",
            ));
        }
        let Some(dot_pos) = domain.rfind('.') else {
            return Some(RuleViolation::new(
                "string.email",
                "domain must contain a '.'",
            ));
        };
        if dot_pos == 0 || dot_pos >= domain.len() - 1 {
            return Some(RuleViolation::new(
                "string.email",
                "domain labels must not be empty",
            ));
        }
        None
    }

    // ── Combined ──────────────────────────────────────────────────────────────

    /// Normalize then validate in one call (Joi-style).
    /// Returns the canonicalized value alongside any violations.
    pub fn process(&self, raw: Option<String>) -> (Option<String>, Result<(), FieldErrors>) {
        let normalized = self.normalize(raw);
        let result = self.validate(&normalized);
        (normalized, result)
    }
}
