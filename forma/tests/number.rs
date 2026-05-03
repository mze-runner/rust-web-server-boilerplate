use forma::{schema, NumberSchema, ViolationContext};

const LIMIT:    NumberSchema = schema::number().min(1).max(100);
const UNBOUNDED: NumberSchema = schema::number();
const NEGATIVE:  NumberSchema = schema::number().min(-100).max(-10);
// inverted range: any value between 5 and 10 violates both bounds simultaneously
const INVERTED:  NumberSchema = schema::number().min(10).max(5);

#[test]
fn value_within_range_passes() {
    assert!(LIMIT.validate(50).is_ok());
}

#[test]
fn value_below_min_fails() {
    let err = LIMIT.validate(0).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "number.min");
}

#[test]
fn value_above_max_fails() {
    let err = LIMIT.validate(101).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "number.max");
}

#[test]
fn boundaries_are_inclusive() {
    assert!(LIMIT.validate(1).is_ok());
    assert!(LIMIT.validate(100).is_ok());
}

// ── violation context ─────────────────────────────────────────────────────────

#[test]
fn min_violation_carries_limit_context() {
    let err = LIMIT.validate(0).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.context, ViolationContext::Limit(1));
}

#[test]
fn max_violation_carries_limit_context() {
    let err = LIMIT.validate(101).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.context, ViolationContext::Limit(100));
}

// ── unbounded schema ──────────────────────────────────────────────────────────

#[test]
fn unbounded_schema_always_passes() {
    assert!(UNBOUNDED.validate(i64::MIN).is_ok());
    assert!(UNBOUNDED.validate(0).is_ok());
    assert!(UNBOUNDED.validate(i64::MAX).is_ok());
}

// ── negative bounds ───────────────────────────────────────────────────────────

#[test]
fn negative_bounds_work() {
    assert!(NEGATIVE.validate(-50).is_ok());
    assert!(NEGATIVE.validate(-10).is_ok());
    assert!(NEGATIVE.validate(-100).is_ok());
    assert!(NEGATIVE.validate(-5).is_err());
    assert!(NEGATIVE.validate(-101).is_err());
}

// ── multiple violations ───────────────────────────────────────────────────────

#[test]
fn both_violations_reported_when_range_is_inverted() {
    // 7 < min(10) and 7 > max(5): both violations must be collected
    let err = INVERTED.validate(7).unwrap_err();
    let inner = err.into_inner();
    assert_eq!(inner.len(), 2);
    assert_eq!(inner[0].1.code, "number.min");
    assert_eq!(inner[1].1.code, "number.max");
}

// ── custom ────────────────────────────────────────────────────────────────────

fn must_be_even(n: i64) -> Option<forma::RuleViolation> {
    if n % 2 != 0 {
        Some(forma::RuleViolation::new("number.even", "must be even"))
    } else {
        None
    }
}

const WITH_CUSTOM: NumberSchema = schema::number().custom(must_be_even);

#[test]
fn custom_fn_passes_when_returning_none() {
    assert!(WITH_CUSTOM.validate(4).is_ok());
}

#[test]
fn custom_fn_returns_violation_on_failure() {
    let err = WITH_CUSTOM.validate(3).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "number.even");
}
