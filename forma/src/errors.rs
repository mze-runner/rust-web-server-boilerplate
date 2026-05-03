/// Machine-readable metadata attached to a violation.
/// Lets API clients render their own messages (e.g. for i18n) without parsing strings.
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationContext {
    None,
    /// A single boundary value — used for `string.min`, `string.max`.
    Limit(usize),
    /// A closed range — used when both ends are meaningful.
    Range { min: usize, max: usize },
}

/// A single constraint failure on one field value.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleViolation {
    /// Dot-namespaced, machine-readable code — e.g. `"string.required"`, `"string.max"`.
    pub code: &'static str,
    /// Human-readable description suitable for direct display.
    pub message: String,
    /// Structured constraint metadata for client-side message rendering.
    pub context: ViolationContext,
}

impl RuleViolation {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: ViolationContext::None,
        }
    }

    pub fn with_limit(code: &'static str, message: impl Into<String>, limit: usize) -> Self {
        Self {
            code,
            message: message.into(),
            context: ViolationContext::Limit(limit),
        }
    }

    pub fn with_range(
        code: &'static str,
        message: impl Into<String>,
        min: usize,
        max: usize,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            context: ViolationContext::Range { min, max },
        }
    }
}

/// Collected field-level violations from a DTO or field type.
///
/// Paths use dot notation: `"subject"`, `"address.street"`.
/// Leaf validators push violations with an empty path; parent DTOs prefix them via [`merge`](FieldErrors::merge).
#[derive(Debug, Default, PartialEq)]
pub struct FieldErrors(Vec<(String, RuleViolation)>);

impl FieldErrors {
    /// Add a violation at the current level (path filled in by the parent via `merge`).
    pub fn push(&mut self, violation: RuleViolation) {
        self.0.push((String::new(), violation));
    }

    /// Add a violation with an explicit dot-separated path.
    /// Use this when converting from external validation reports that already carry full field paths.
    pub fn push_at(&mut self, path: impl Into<String>, violation: RuleViolation) {
        self.0.push((path.into(), violation));
    }

    /// Prefix all violations from `result` with `field` and collect them.
    ///
    /// ```text
    /// errors.merge("subject", self.subject.validate());
    /// // inner path ""       → "subject"
    /// // inner path "nested" → "subject.nested"
    /// ```
    pub fn merge(&mut self, field: &str, result: Result<(), FieldErrors>) {
        if let Err(inner) = result {
            for (path, violation) in inner.0 {
                let full = if path.is_empty() {
                    field.to_owned()
                } else {
                    format!("{field}.{path}")
                };
                self.0.push((full, violation));
            }
        }
    }

    /// Convert to `Err(self)` if any violations were collected; otherwise `Ok(())`.
    pub fn finish(self) -> Result<(), Self> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and return all `(path, violation)` pairs.
    pub fn into_inner(self) -> Vec<(String, RuleViolation)> {
        self.0
    }
}

impl std::fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::fmt::Display for FieldErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (path, violation) in &self.0 {
            if !first {
                write!(f, "; ")?;
            }
            write!(f, "{path}: {violation}")?;
            first = false;
        }
        Ok(())
    }
}

impl std::error::Error for FieldErrors {}
