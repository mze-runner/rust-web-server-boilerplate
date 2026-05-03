#![cfg(feature = "datetime")]

use chrono::{DateTime, FixedOffset, TimeZone};
use forma::{schema, DateTimeSchema};

const RULE:         DateTimeSchema = schema::datetime().required().utc();
const OPTIONAL_UTC: DateTimeSchema = schema::datetime().utc();
const OPTIONAL_ANY: DateTimeSchema = schema::datetime();

fn utc(y: i32, m: u32, d: u32) -> chrono::DateTime<FixedOffset> {
    FixedOffset::east_opt(0)
        .unwrap()
        .with_ymd_and_hms(y, m, d, 0, 0, 0)
        .unwrap()
}

fn non_utc() -> chrono::DateTime<FixedOffset> {
    FixedOffset::east_opt(3600)
        .unwrap()
        .with_ymd_and_hms(2025, 1, 1, 12, 0, 0)
        .unwrap()
}

#[test]
fn none_is_rejected_when_required() {
    assert!(RULE.validate(&None).is_err());
}

#[test]
fn utc_datetime_passes() {
    assert!(RULE.validate(&Some(utc(2025, 6, 1))).is_ok());
}

#[test]
fn non_utc_offset_is_rejected() {
    let err = RULE.validate(&Some(non_utc())).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "datetime.utc");
}

// ── optional (no .required()) ─────────────────────────────────────────────────

#[test]
fn optional_schema_accepts_none() {
    assert!(OPTIONAL_UTC.validate(&None).is_ok());
    assert!(OPTIONAL_ANY.validate(&None).is_ok());
}

#[test]
fn utc_constraint_still_enforced_when_present() {
    assert!(OPTIONAL_UTC.validate(&Some(utc(2025, 1, 1))).is_ok());
    assert!(OPTIONAL_UTC.validate(&Some(non_utc())).is_err());
}

#[test]
fn non_utc_passes_when_utc_not_required() {
    assert!(OPTIONAL_ANY.validate(&Some(non_utc())).is_ok());
}

// ── custom ────────────────────────────────────────────────────────────────────

fn always_reject(_: &DateTime<FixedOffset>) -> Option<forma::RuleViolation> {
    Some(forma::RuleViolation::new("datetime.custom", "always fails"))
}

const WITH_CUSTOM: DateTimeSchema = schema::datetime().custom(always_reject);

#[test]
fn custom_fn_violation_propagated() {
    let err = WITH_CUSTOM.validate(&Some(utc(2025, 1, 1))).unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "datetime.custom");
}

#[test]
fn custom_fn_not_called_for_none() {
    // Without .required(), None is accepted even with a custom fn that always rejects
    assert!(WITH_CUSTOM.validate(&None).is_ok());
}
