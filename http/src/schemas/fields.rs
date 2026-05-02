use serde::{Deserialize, Deserializer};

// password has built-in deserializer
#[derive(Debug, Deserialize, garde::Validate)]
#[garde(transparent)]
pub struct PasswordField(#[garde(required, length(min = 8, max = 20))] pub Option<String>);

impl PasswordField {
    pub fn as_str(&self) -> &str {
        self.0
            .as_deref()
            .expect("Password field should be validated before access")
    }
}

// email has custome deserializer to normalize domain part to lowercase
#[derive(Debug, garde::Validate)]
#[garde(transparent)]
pub struct EmailField(#[garde(required, email, length(max = 64))] Option<String>);
impl EmailField {
    pub fn new(email: Option<String>) -> Self {
        let normalized = email.map(|e| {
            // Trim whitespace first
            let trimmed = e.trim();
            if let Some((local, domain)) = trimmed.split_once('@') {
                // Normalize: trim local part, lowercase domain
                format!("{}@{}", local.trim(), domain.to_ascii_lowercase())
            } else {
                trimmed.to_string()
            }
        });
        EmailField(normalized)
    }

    pub fn as_str(&self) -> &str {
        self.0
            .as_deref()
            .expect("Email field should be validated before access")
    }

    pub fn domain(&self) -> &str {
        self.as_str()
            .split_once('@')
            .map(|(_, domain)| domain)
            .expect("Email should be valid after garde validation")
    }

    pub fn local_part(&self) -> &str {
        self.as_str()
            .split_once('@')
            .map(|(local, _)| local)
            .expect("Email should be valid after garde validation")
    }
}

// Custom deserializer
impl<'de> Deserialize<'de> for EmailField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(EmailField::new(opt))
    }
}

// Pagination Fields

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(transparent)]
pub struct PaginationLimitField(#[garde(range(max = 100))] pub u32);

impl Default for PaginationLimitField {
    fn default() -> Self {
        Self(20)
    }
}

// Comment Fields

#[derive(Debug, garde::Validate)]
#[garde(transparent)]
pub struct CommentBodyField(#[garde(required, custom(validate_comment_body))] pub Option<String>);

impl CommentBodyField {
    pub fn into_string(self) -> String {
        self.0
            .expect("CommentBodyField must be validated before access")
    }
}

fn validate_comment_body(value: &Option<String>, _ctx: &()) -> garde::Result {
    let Some(s) = value else { return Ok(()) };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(garde::Error::new("body must not be blank"));
    }
    if trimmed.chars().count() > 1000 {
        return Err(garde::Error::new("body must be at most 1000 characters"));
    }
    Ok(())
}

impl<'de> serde::Deserialize<'de> for CommentBodyField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(CommentBodyField(opt))
    }
}

// Tasks Fields

// Task Title field
// TODO: dopes not handlt empty string well - likely crate limitations
#[derive(Debug, Deserialize, garde::Validate)]
#[garde(transparent)]
pub struct TasksTitleField(#[garde(required, length(min = 1, max = 100))] pub Option<String>);

// Task Subject field (1–256 chars, whitespace-only is invalid)
#[derive(Debug, garde::Validate)]
#[garde(transparent)]
pub struct TaskSubjectField(
    #[garde(required, custom(validate_non_blank_subject))] pub Option<String>,
);

impl TaskSubjectField {
    pub fn into_string(self) -> String {
        self.0
            .expect("TaskSubjectField must be validated before access")
    }
}

fn validate_non_blank_subject(value: &Option<String>, _ctx: &()) -> garde::Result {
    let Some(s) = value else { return Ok(()) };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(garde::Error::new("subject must not be blank"));
    }
    if trimmed.chars().count() > 256 {
        return Err(garde::Error::new("subject must be at most 256 characters"));
    }
    Ok(())
}

impl<'de> Deserialize<'de> for TaskSubjectField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        Ok(TaskSubjectField(opt))
    }
}

impl TasksTitleField {
    pub fn as_str(&self) -> &str {
        self.0
            .as_deref()
            .expect("Title field should be validated before access")
    }
}

// Task Desciption field
#[derive(Debug, Default, Deserialize, garde::Validate)]
#[garde(transparent)]
// pub struct TaskDescriptionField(#[garde(length(min = 1, max = 500))] pub Option<String>);
pub struct TaskDescriptionField(#[garde(length(max = 500))] pub Option<String>);

impl TaskDescriptionField {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

// // Task due date field
// #[derive(Debug, Deserialize, garde::Validate)]
// #[garde(transparent)]
// pub struct TaskDueDateField(#[garde(required)] pub Option<chrono::DateTime<chrono::Utc>>);

// impl TaskDueDateField {
//     pub fn as_datetime(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
//         self.0.as_ref()
//     }

//     pub fn is_overdue(&self) -> bool {
//         match &self.0 {
//             Some(due_date) => *due_date < chrono::Utc::now(),
//             None => false, // No due date means not overdue
//         }
//     }
// }

// General date field
// Task due date field - enforces UTC timezone
#[derive(Debug, Default, Deserialize, garde::Validate)]
#[garde(transparent)]
pub struct DateField(
    #[garde(required, custom(validate_utc_datetime))]
    pub  Option<chrono::DateTime<chrono::FixedOffset>>,
);

impl DateField {
    pub fn as_datetime(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        // self.0.as_ref().map(|dt| dt.with_timezone(&chrono::Utc))
        self.0.as_ref().map(|dt| dt.with_timezone(&chrono::Utc)) // Returns owned DateTime
    }

    pub fn is_overdue(&self) -> bool {
        match &self.0 {
            Some(due_date) => *due_date < chrono::Utc::now(),
            None => false, // No due date means not overdue
        }
    }
}

/// Custom validation function for UTC datetime fields
fn validate_utc_datetime(
    datetime: &Option<chrono::DateTime<chrono::FixedOffset>>,
    _context: &(),
) -> garde::Result {
    match datetime {
        Some(dt) => {
            // Check if timezone is UTC (offset must be +00:00)
            if dt.offset().local_minus_utc() != 0 {
                return Err(garde::Error::new(format!(
                    "Date must be in UTC timezone. Got timezone offset: {:+05}h",
                    dt.offset().local_minus_utc() / 3600
                )));
            }
            Ok(())
        }
        None => Ok(()), // None is valid for optional fields
    }
}

// /// Custom validation function for UTC datetime fields
// fn validate_utc_datetime(
//     datetime: &Option<chrono::DateTime<chrono::Utc>>,
//     _context: &()
// ) -> garde::Result {
//     // Optional field - None is valid
//     if datetime.is_none() {
//         return Ok(());
//     }

//     // If we reach here, datetime parsing succeeded and it's in UTC
//     // chrono::DateTime<Utc> guarantees UTC timezone after successful parsing
//     Ok(())
// }

// /// Custom deserializer to enforce UTC timezone during parsing
// impl<'de> Deserialize<'de> for DateField {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let opt = Option::<String>::deserialize(deserializer)?;

//         match opt {
//             Some(date_str) => {
//                 // Parse the datetime string and enforce UTC
//                 match parse_utc_datetime(&date_str) {
//                     Ok(datetime) => Ok(DateField(Some(datetime))),
//                     Err(e) => Err(serde::de::Error::custom(format!("Invalid UTC datetime: {}", e))),
//                 }
//             },
//             None => Ok(DateField(None)),
//         }
//     }
// }

// /// Parse datetime string and enforce UTC timezone
// fn parse_utc_datetime(date_str: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
//     // First try parsing as RFC3339 (ISO 8601) with timezone
//     if let Ok(datetime_with_tz) = chrono::DateTime::parse_from_rfc3339(date_str) {
//         // Check if the timezone is UTC (offset must be +00:00)
//         if datetime_with_tz.offset().local_minus_utc() != 0 {
//             return Err(format!("Date must be in UTC timezone. Got timezone offset: {:+05}",
//                 datetime_with_tz.offset().local_minus_utc() / 3600));
//         }
//         // Convert to UTC (safe since we verified it's already UTC)
//         return Ok(datetime_with_tz.with_timezone(&chrono::Utc));
//     }

//     // Try parsing as naive datetime and assume UTC (only if it ends with 'Z' or has no timezone)
//     if date_str.ends_with('Z') {
//         if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&date_str[..date_str.len()-1], "%Y-%m-%dT%H:%M:%S") {
//             return Ok(chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc));
//         }
//         // Also try with microseconds
//         if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&date_str[..date_str.len()-1], "%Y-%m-%dT%H:%M:%S%.f") {
//             return Ok(chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc));
//         }
//     }

//     Err(format!("Invalid datetime format. Expected UTC datetime in ISO 8601 format (e.g., '2025-11-15T10:00:00Z' or '2025-11-15T10:00:00+00:00')"))
// }
