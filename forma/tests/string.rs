use forma::{schema, StringSchema, ViolationContext};

const SUBJECT:  StringSchema = schema::string().trim().required().max(10);
const ALPHANUM: StringSchema = schema::string().trim().required().alphanum();
const EMAIL:    StringSchema = schema::string().required().max(64).email();
const WITH_MIN: StringSchema = schema::string().required().min(3).max(10);
const LOWER:    StringSchema = schema::string().trim().lowercase().required();
const UPPER:    StringSchema = schema::string().uppercase().required();

#[test]
fn none_is_rejected_when_required() {
    assert!(SUBJECT.validate(&None).is_err());
}

#[test]
fn whitespace_only_is_rejected_after_trim() {
    let (normalized, result) = SUBJECT.process(Some("   ".to_owned()));
    assert_eq!(normalized, Some(String::new()));
    assert!(result.is_err());
}

#[test]
fn valid_value_passes() {
    let (normalized, result) = SUBJECT.process(Some("  hello  ".to_owned()));
    assert_eq!(normalized.as_deref(), Some("hello"));
    assert!(result.is_ok());
}

#[test]
fn max_counts_chars_not_bytes() {
    // "é" is 2 bytes but 1 char — 10 × "é" must pass a max(10) rule
    let (_, result) = SUBJECT.process(Some("éééééééééé".to_owned()));
    assert!(result.is_ok());

    // 11 × "é" must fail
    let (_, result) = SUBJECT.process(Some("ééééééééééé".to_owned()));
    assert!(result.is_err());
}

#[test]
fn violation_codes_are_namespaced() {
    let err = SUBJECT.validate(&None).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.required");
}

// ── alphanum ──────────────────────────────────────────────────────────────────

#[test]
fn alphanum_accepts_letters_and_digits() {
    let (_, result) = ALPHANUM.process(Some("Hello123".to_owned()));
    assert!(result.is_ok());
}

#[test]
fn alphanum_rejects_spaces() {
    let (_, result) = ALPHANUM.process(Some("hello world".to_owned()));
    assert!(result.is_err());
}

#[test]
fn alphanum_rejects_punctuation() {
    let (_, result) = ALPHANUM.process(Some("hello!".to_owned()));
    assert!(result.is_err());
}

#[test]
fn alphanum_rejects_non_ascii() {
    let (_, result) = ALPHANUM.process(Some("héllo".to_owned()));
    assert!(result.is_err());
}

#[test]
fn alphanum_violation_code() {
    let (_, result) = ALPHANUM.process(Some("bad value!".to_owned()));
    let err = result.unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.alphanum");
}

// ── min ───────────────────────────────────────────────────────────────────────

#[test]
fn at_min_boundary_passes() {
    assert!(WITH_MIN.validate(&Some("abc".to_owned())).is_ok());
}

#[test]
fn below_min_fails() {
    let err = WITH_MIN.validate(&Some("ab".to_owned())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.min");
}

#[test]
fn min_counts_unicode_chars() {
    // "é" is 2 bytes but 1 char
    assert!(WITH_MIN.validate(&Some("ééé".to_owned())).is_ok());
    assert!(WITH_MIN.validate(&Some("éé".to_owned())).is_err());
}

#[test]
fn min_violation_carries_limit_context() {
    let err = WITH_MIN.validate(&Some("ab".to_owned())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.context, ViolationContext::Limit(3));
}

#[test]
fn max_violation_carries_limit_context() {
    let err = WITH_MIN.validate(&Some("abcdefghijk".to_owned())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.context, ViolationContext::Limit(10));
}

// ── lowercase / uppercase ─────────────────────────────────────────────────────

#[test]
fn lowercase_normalizes_mixed_case() {
    let (normalized, result) = LOWER.process(Some("  Hello WORLD  ".to_owned()));
    assert_eq!(normalized.as_deref(), Some("hello world"));
    assert!(result.is_ok());
}

#[test]
fn uppercase_normalizes_mixed_case() {
    let (normalized, result) = UPPER.process(Some("hello world".to_owned()));
    assert_eq!(normalized.as_deref(), Some("HELLO WORLD"));
    assert!(result.is_ok());
}

// ── custom ────────────────────────────────────────────────────────────────────

fn reject_spaces(s: &str) -> Option<forma::RuleViolation> {
    if s.contains(' ') {
        Some(forma::RuleViolation::new("string.no_spaces", "must not contain spaces"))
    } else {
        None
    }
}

const WITH_CUSTOM: StringSchema = schema::string().required().custom(reject_spaces);

#[test]
fn custom_fn_passes_when_returning_none() {
    assert!(WITH_CUSTOM.validate(&Some("hello".to_owned())).is_ok());
}

#[test]
fn custom_fn_returns_violation_on_failure() {
    let err = WITH_CUSTOM.validate(&Some("hello world".to_owned())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.no_spaces");
}

// ── email ─────────────────────────────────────────────────────────────────────

#[test]
fn email_accepts_valid_address() {
    assert!(EMAIL.validate(&Some("user@example.com".to_owned())).is_ok());
}

#[test]
fn email_accepts_subdomain() {
    assert!(EMAIL.validate(&Some("user@mail.example.com".to_owned())).is_ok());
}

#[test]
fn email_rejects_missing_at() {
    assert!(EMAIL.validate(&Some("userexample.com".to_owned())).is_err());
}

#[test]
fn email_rejects_missing_domain() {
    assert!(EMAIL.validate(&Some("user@".to_owned())).is_err());
}

#[test]
fn email_rejects_missing_tld() {
    assert!(EMAIL.validate(&Some("user@example".to_owned())).is_err());
}

#[test]
fn email_rejects_dot_at_domain_end() {
    assert!(EMAIL.validate(&Some("user@example.".to_owned())).is_err());
}

#[test]
fn email_violation_code() {
    let err = EMAIL.validate(&Some("not-an-email".to_owned())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.email");
}
