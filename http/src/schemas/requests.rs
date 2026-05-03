use uuid::Uuid;

use crate::schemas::fields::{
    CommentBodyField, EmailField, PaginationLimitField, PasswordField, TaskDescriptionField,
    TaskSubjectField,
};
use serde::Deserialize;

// ── Auth ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignupRequest {
    pub email: EmailField,
    pub password: PasswordField,
}

impl forma::Validate for SignupRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("email", self.email.validate());
        e.merge("password", self.password.validate());
        e.finish()
    }
}

// ── Task Requests ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTaskRequest {
    pub subject: TaskSubjectField,
    #[serde(default)]
    pub description: TaskDescriptionField,
    pub assignee_id: Option<Uuid>,
}

impl forma::Validate for CreateTaskRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("subject", self.subject.validate());
        e.merge("description", self.description.validate());
        e.finish()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditTaskRequest {
    #[serde(default)]
    pub subject: Option<TaskSubjectField>,
    #[serde(default)]
    pub description: Option<TaskDescriptionField>,
    #[serde(default)]
    pub status: Option<String>,
}

impl EditTaskRequest {
    pub fn is_empty(&self) -> bool {
        self.subject.is_none() && self.description.is_none() && self.status.is_none()
    }
}

impl forma::Validate for EditTaskRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        if let Some(subject) = &self.subject {
            e.merge("subject", subject.validate());
        }
        if let Some(description) = &self.description {
            e.merge("description", description.validate());
        }
        e.finish()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignTaskRequest {
    pub assignee_id: Uuid,
}

impl forma::Validate for AssignTaskRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        Ok(())
    }
}

// ── Comment Requests ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddCommentRequest {
    pub body: CommentBodyField,
}

impl forma::Validate for AddCommentRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("body", self.body.validate());
        e.finish()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditCommentRequest {
    pub body: CommentBodyField,
}

impl forma::Validate for EditCommentRequest {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("body", self.body.validate());
        e.finish()
    }
}

// ── Query Params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListTasksParams {
    #[serde(default, rename = "status")]
    pub statuses: Vec<String>,
    #[serde(default)]
    pub limit: PaginationLimitField,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListCommentsParams {
    #[serde(default)]
    pub limit: PaginationLimitField,
    pub cursor: Option<String>,
}

impl forma::Validate for ListTasksParams {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("limit", self.limit.validate());
        e.finish()
    }
}

impl forma::Validate for ListCommentsParams {
    fn validate(&self) -> Result<(), forma::FieldErrors> {
        let mut e = forma::FieldErrors::default();
        e.merge("limit", self.limit.validate());
        e.finish()
    }
}
