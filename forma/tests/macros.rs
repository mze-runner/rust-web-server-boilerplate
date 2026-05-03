use forma::Validate;

// ── string_field! ─────────────────────────────────────────────────────────────

struct SubjectField(Option<String>);
forma::string_field!(SubjectField, string().trim().required().min(1).max(10));

#[test]
fn string_field_normalize_runs_on_deserialize() {
    let field: SubjectField = serde_json::from_str("\" hello \"").unwrap();
    assert_eq!(field.0.as_deref(), Some("hello"));
}

#[test]
fn string_field_null_deserializes_as_none() {
    let field: SubjectField = serde_json::from_str("null").unwrap();
    assert!(field.0.is_none());
}

#[test]
fn string_field_valid_passes_validate() {
    let field: SubjectField = serde_json::from_str("\"hello\"").unwrap();
    assert!(field.validate().is_ok());
}

#[test]
fn string_field_required_rejects_none() {
    let field: SubjectField = serde_json::from_str("null").unwrap();
    assert!(field.validate().is_err());
}

#[test]
fn string_field_max_violation_reported() {
    let field: SubjectField = serde_json::from_str("\"toolongstring!\"").unwrap();
    let err = field.validate().unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "string.max");
}

// ── number_field! ─────────────────────────────────────────────────────────────

struct LimitField(u32);
forma::number_field!(LimitField, u32, number().max(100));

#[test]
fn number_field_deserializes_primitive() {
    let field: LimitField = serde_json::from_str("50").unwrap();
    assert_eq!(field.0, 50);
}

#[test]
fn number_field_valid_passes_validate() {
    let field: LimitField = serde_json::from_str("50").unwrap();
    assert!(field.validate().is_ok());
}

#[test]
fn number_field_max_violation_reported() {
    let field: LimitField = serde_json::from_str("200").unwrap();
    let err = field.validate().unwrap_err();
    let (_, v) = &err.into_inner()[0];
    assert_eq!(v.code, "number.max");
}

// ── datetime_field! ───────────────────────────────────────────────────────────

#[cfg(feature = "datetime")]
mod datetime_macro {
    use forma::Validate;

    struct DueAtField(Option<chrono::DateTime<chrono::FixedOffset>>);
    forma::datetime_field!(DueAtField, datetime().required().utc());

    #[test]
    fn datetime_field_deserializes_utc_datetime() {
        let field: DueAtField = serde_json::from_str("\"2025-06-01T00:00:00+00:00\"").unwrap();
        assert!(field.0.is_some());
    }

    #[test]
    fn datetime_field_null_deserializes_as_none() {
        let field: DueAtField = serde_json::from_str("null").unwrap();
        assert!(field.0.is_none());
    }

    #[test]
    fn datetime_field_valid_utc_passes() {
        let field: DueAtField = serde_json::from_str("\"2025-06-01T00:00:00+00:00\"").unwrap();
        assert!(field.validate().is_ok());
    }

    #[test]
    fn datetime_field_non_utc_fails() {
        let field: DueAtField = serde_json::from_str("\"2025-06-01T12:00:00+01:00\"").unwrap();
        let err = field.validate().unwrap_err();
        let (_, v) = &err.into_inner()[0];
        assert_eq!(v.code, "datetime.utc");
    }

    #[test]
    fn datetime_field_required_rejects_null() {
        let field: DueAtField = serde_json::from_str("null").unwrap();
        let err = field.validate().unwrap_err();
        let (_, v) = &err.into_inner()[0];
        assert_eq!(v.code, "datetime.required");
    }
}
