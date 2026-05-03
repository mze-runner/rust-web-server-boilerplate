# Forma *(beta)*

> **Don't Trust Your Client.**
>
> Input that reaches your service can be malformed, over-long, blank-but-not-empty, or subtly wrong in ways that only surface after it is stored. Validation alone is not enough — raw values must be normalized into a canonical form first, so that what you validate is exactly what you will use.

Schema-driven normalization and validation for Rust web services.

---

## Why Forma exists

Attribute-based validation crates — well-designed and credit due to their authors — operate on raw deserialized values. That works for simple constraint checks, but falls apart in combined scenarios:

- A string that is technically non-empty but consists entirely of whitespace passes a `required` check yet carries no meaningful content.
- Length limits applied to raw bytes rather than Unicode code points give different results for multibyte input.
- There is no standard place to express "trim before you measure" — normalization and validation are conflated or left to the caller.

The result is hand-written `validate_*` functions scattered across every text field, which is exactly the boilerplate a declarative approach is supposed to eliminate.

**Forma (beta)** was introduced to address these combined cases. It treats normalization as a first-class phase that runs before any constraint check, so that field types carry a guarantee: by the time a value is validated, it is already canonical.

---

## Two-phase pipeline

```
Raw input (JSON)
      │
      ▼
 Deserialize           ← Phase 1: normalize
 (trim, lowercase)         schema.normalize(raw)
      │
      ▼
 Validated value       ← Phase 2: validate
 (constraints check)       schema.validate(&value)
      │
      ▼
 FieldErrors → ProblemDetails (RFC 7807)
```

**Phase 1** runs inside `Deserialize` impls on field types. The stored value is always canonical.

**Phase 2** runs via the `Validate` trait, called by `ValidatedJson` after deserialization.

This means a `TaskSubjectField`, once constructed, is **guaranteed to be trimmed**. No call site can hold an un-normalized value.

---

## Usage

### Defining rules

Rules are `const`-constructible — define them once at module level, with zero runtime allocation.

```rust
use forma::{schema, StringSchema, NumberSchema};

const SUBJECT:     StringSchema = schema::string().trim().required().max(256);
const BODY:        StringSchema = schema::string().trim().required().max(1000);
const DESCRIPTION: StringSchema = schema::string().trim().max(500);
const PAGE_LIMIT:  NumberSchema = schema::number().min(1).max(100);
```

With the `datetime` feature:

```rust
use forma::{schema, DateTimeSchema};

const DUE_DATE: DateTimeSchema = schema::datetime().required().require_utc();
```

---

### Implementing a field type

```rust
use serde::{Deserialize, Deserializer};
use forma::{schema, StringSchema, Validate, FieldErrors};

const RULE: StringSchema = schema::string().trim().required().max(256);

pub struct TaskSubjectField(pub Option<String>);

// Phase 1: normalize in Deserialize — stored value is always trimmed
impl<'de> Deserialize<'de> for TaskSubjectField {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Option::<String>::deserialize(d)?;
        Ok(Self(RULE.normalize(raw)))
    }
}

// Phase 2: validate constraints on the already-normalized value
impl Validate for TaskSubjectField {
    fn validate(&self) -> Result<(), FieldErrors> {
        RULE.validate(&self.0)
    }
}
```

---

### Implementing a DTO

DTOs aggregate field-level results. Field paths use dot notation.

```rust
use forma::{Validate, FieldErrors};

impl Validate for CreateTaskRequest {
    fn validate(&self) -> Result<(), FieldErrors> {
        let mut e = FieldErrors::default();
        e.merge("subject",     self.subject.validate());
        e.merge("description", self.description.validate());
        e.finish()
    }
}
```

Nested DTOs compose naturally:

```rust
e.merge("address", self.address.validate());
// inner violation at "street" → full path becomes "address.street"
```

---

### Using `process()` for a single-pass call

When you want to normalize and validate in one step (e.g., in tests or non-HTTP contexts):

```rust
let (normalized, result) = RULE.process(Some("  hello  ".to_owned()));
// normalized = Some("hello")
// result     = Ok(())
```

---

## Schema reference

### `schema::string()` → `StringSchema`

| Method | Phase | Description |
|---|---|---|
| `.trim()` | normalize | Strip leading/trailing whitespace before validation |
| `.required()` | validate | Reject `None` and empty/blank strings |
| `.min(n)` | validate | Minimum character count (Unicode code points) |
| `.max(n)` | validate | Maximum character count (Unicode code points) |
| `.custom(fn)` | validate | Service-specific rule via `fn(&str) -> Option<RuleViolation>` |

**Length is always measured in Unicode code points, not bytes.**

---

### `schema::number()` → `NumberSchema`

Bounds are `i64`. Field types cast their concrete type before calling `validate`:

```rust
impl Validate for PaginationLimitField {
    fn validate(&self) -> Result<(), FieldErrors> {
        RULE.validate(self.0 as i64)
    }
}
```

| Method | Description |
|---|---|
| `.min(n)` | Inclusive lower bound |
| `.max(n)` | Inclusive upper bound |
| `.custom(fn)` | Service-specific rule via `fn(i64) -> Option<RuleViolation>` |

---

### `schema::datetime()` → `DateTimeSchema` *(feature: `datetime`)*

Operates on `Option<chrono::DateTime<chrono::FixedOffset>>`. Parsing from a string is serde's responsibility; this schema handles post-parse constraints only.

| Method | Description |
|---|---|
| `.required()` | Reject `None` |
| `.require_utc()` | Reject any offset other than `+00:00` |
| `.custom(fn)` | Service-specific rule via `fn(&DateTime<FixedOffset>) -> Option<RuleViolation>` |

---

## Error structure

### `RuleViolation`

```rust
pub struct RuleViolation {
    pub code:    &'static str,     // "string.required", "string.max", "datetime.utc"
    pub message: String,           // human-readable, suitable for direct display
    pub context: ViolationContext, // structured constraint metadata
}
```

### Violation codes

| Code | Trigger |
|---|---|
| `string.required` | `None` or empty/blank string on a `.required()` field |
| `string.min` | Character count below `.min(n)` |
| `string.max` | Character count above `.max(n)` |
| `number.min` | Value below `.min(n)` |
| `number.max` | Value above `.max(n)` |
| `datetime.required` | `None` on a `.required()` field |
| `datetime.utc` | Non-zero UTC offset on a `.require_utc()` field |

### `ViolationContext`

```rust
pub enum ViolationContext {
    None,
    Limit(usize),                    // single boundary — min/max
    Range { min: usize, max: usize },// closed range
}
```

### `FieldErrors`

A flat list of `(dot_path, RuleViolation)` pairs. Paths are assembled by parent DTOs via `merge`.

```rust
let errors: Vec<(String, RuleViolation)> = field_errors.into_inner();
// e.g. [("subject", RuleViolation { code: "string.required", .. })]
```

---

## Feature flags

| Feature | Default | Description |
|---|---|---|
| `datetime` | off | Enables `DateTimeSchema` and `schema::datetime()`, pulls in `chrono` |

```toml
forma = { path = "../validation", features = ["datetime"] }
```

---

## Extending with custom rules

Custom rules are plain function pointers — `const`-compatible, zero allocation.

```rust
fn no_html(s: &str) -> Option<RuleViolation> {
    if s.contains('<') || s.contains('>') {
        Some(RuleViolation::new("string.no_html", "must not contain HTML"))
    } else {
        None
    }
}

const RULE: StringSchema = schema::string().trim().required().max(500).custom(no_html);
```

---

## Running tests

```bash
# Unit and doc tests, no datetime support
cargo test -p forma

# With datetime support
cargo test -p forma --features datetime
```
