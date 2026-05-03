use forma::{datetime_field, number_field, string_field};

// ── PasswordField ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PasswordField(pub Option<String>);
string_field!(PasswordField, string().required().min(8).max(20));

impl PasswordField {
    pub fn as_str(&self) -> &str {
        self.0
            .as_deref()
            .expect("PasswordField must be validated before access")
    }
}

// ── EmailField ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct EmailField(pub Option<String>);
string_field!(
    EmailField,
    string().trim().lowercase().required().max(64).email()
);

// ── Pagination ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PaginationLimitField(pub u32);
number_field!(PaginationLimitField, u32, number().max(100));

impl Default for PaginationLimitField {
    fn default() -> Self {
        Self(20)
    }
}

// ── CommentBodyField ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct CommentBodyField(pub Option<String>);
string_field!(CommentBodyField, string().trim().required().max(1000));

impl CommentBodyField {
    pub fn into_string(self) -> String {
        self.0
            .expect("CommentBodyField must be validated before access")
    }
}

// ── Task Fields ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TasksTitleField(pub Option<String>);
string_field!(TasksTitleField, string().trim().required().min(1).max(100));

impl TasksTitleField {
    pub fn as_str(&self) -> &str {
        self.0
            .as_deref()
            .expect("TasksTitleField must be validated before access")
    }
}

#[derive(Debug)]
pub struct TaskSubjectField(pub Option<String>);
string_field!(TaskSubjectField, string().trim().required().min(1).max(256));

impl TaskSubjectField {
    pub fn into_string(self) -> String {
        self.0
            .expect("TaskSubjectField must be validated before access")
    }
}

#[derive(Debug, Default)]
pub struct TaskDescriptionField(pub Option<String>);
string_field!(TaskDescriptionField, string().trim().max(500));

impl TaskDescriptionField {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

// ── DateField ─────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct DateField(pub Option<chrono::DateTime<chrono::FixedOffset>>);
datetime_field!(DateField, datetime().required().utc());

impl DateField {
    pub fn as_datetime(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.0.as_ref().map(|dt| dt.with_timezone(&chrono::Utc))
    }

    pub fn is_overdue(&self) -> bool {
        match &self.0 {
            Some(due_date) => *due_date < chrono::Utc::now(),
            None => false,
        }
    }
}
