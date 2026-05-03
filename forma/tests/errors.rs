use forma::{FieldErrors, RuleViolation, ViolationContext};

fn any_violation() -> RuleViolation {
    RuleViolation::new("test.code", "something went wrong")
}

// ── push / push_at ────────────────────────────────────────────────────────────

#[test]
fn push_leaves_path_empty() {
    let mut errors = FieldErrors::default();
    errors.push(any_violation());
    let inner = errors.into_inner();
    assert_eq!(inner[0].0, "");
}

#[test]
fn push_at_uses_explicit_path() {
    let mut errors = FieldErrors::default();
    errors.push_at("address.street", any_violation());
    let inner = errors.into_inner();
    assert_eq!(inner[0].0, "address.street");
}

// ── merge ─────────────────────────────────────────────────────────────────────

#[test]
fn merge_ok_adds_no_violations() {
    let mut errors = FieldErrors::default();
    errors.merge("field", Ok(()));
    assert!(errors.is_empty());
}

#[test]
fn merge_prefixes_empty_inner_path() {
    let mut parent = FieldErrors::default();
    let mut child = FieldErrors::default();
    child.push(any_violation());
    parent.merge("subject", Err(child));
    let inner = parent.into_inner();
    assert_eq!(inner[0].0, "subject");
}

#[test]
fn merge_prefixes_nested_inner_path() {
    let mut parent = FieldErrors::default();
    let mut child = FieldErrors::default();
    child.push_at("nested", any_violation());
    parent.merge("field", Err(child));
    let inner = parent.into_inner();
    assert_eq!(inner[0].0, "field.nested");
}

#[test]
fn merge_collects_all_violations() {
    let mut parent = FieldErrors::default();
    let mut child = FieldErrors::default();
    child.push(any_violation());
    child.push(any_violation());
    parent.merge("field", Err(child));
    assert_eq!(parent.into_inner().len(), 2);
}

// ── is_empty / finish ─────────────────────────────────────────────────────────

#[test]
fn is_empty_on_default() {
    assert!(FieldErrors::default().is_empty());
}

#[test]
fn is_empty_false_after_push() {
    let mut errors = FieldErrors::default();
    errors.push(any_violation());
    assert!(!errors.is_empty());
}

#[test]
fn finish_returns_ok_when_empty() {
    assert!(FieldErrors::default().finish().is_ok());
}

#[test]
fn finish_returns_err_when_violations_present() {
    let mut errors = FieldErrors::default();
    errors.push(any_violation());
    assert!(errors.finish().is_err());
}

// ── ViolationContext constructors ─────────────────────────────────────────────

#[test]
fn new_violation_has_none_context() {
    let v = RuleViolation::new("code", "msg");
    assert_eq!(v.context, ViolationContext::None);
}

#[test]
fn with_limit_carries_limit_context() {
    let v = RuleViolation::with_limit("code", "msg", 42);
    assert_eq!(v.context, ViolationContext::Limit(42));
}

#[test]
fn with_range_carries_range_context() {
    let v = RuleViolation::with_range("code", "msg", 1, 100);
    assert_eq!(v.context, ViolationContext::Range { min: 1, max: 100 });
}

// ── Display ───────────────────────────────────────────────────────────────────

#[test]
fn display_rule_violation() {
    let v = RuleViolation::new("string.required", "must not be blank");
    assert_eq!(v.to_string(), "[string.required] must not be blank");
}

#[test]
fn display_field_errors_single() {
    let mut errors = FieldErrors::default();
    errors.push_at("email", RuleViolation::new("string.required", "must not be blank"));
    assert_eq!(errors.to_string(), "email: [string.required] must not be blank");
}

#[test]
fn display_field_errors_multiple() {
    let mut errors = FieldErrors::default();
    errors.push_at("a", RuleViolation::new("code.a", "first"));
    errors.push_at("b", RuleViolation::new("code.b", "second"));
    assert_eq!(errors.to_string(), "a: [code.a] first; b: [code.b] second");
}
