use uuid::Uuid;

use crate::schemas::fields::{EmailField, PasswordField, TaskDescriptionField, TaskSubjectField};
use serde::Deserialize;

/// Request body for POST /providers/self/signup
#[derive(Debug, Deserialize, garde::Validate)]
#[serde(deny_unknown_fields)]
pub struct SignupRequest {
    #[garde(dive)]
    pub email: EmailField,
    #[garde(dive)]
    pub password: PasswordField,
}

// Tasks Requests

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(deny_unknown_fields)]
pub struct EditTaskRequest {
    #[garde(dive)]
    #[serde(default)]
    pub subject: Option<TaskSubjectField>,

    #[garde(dive)]
    #[serde(default)]
    pub description: Option<TaskDescriptionField>,

    #[garde(skip)]
    #[serde(default)]
    pub status: Option<String>,
}

impl EditTaskRequest {
    pub fn is_empty(&self) -> bool {
        self.subject.is_none() && self.description.is_none() && self.status.is_none()
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(deny_unknown_fields)]
pub struct AssignTaskRequest {
    #[garde(skip)]
    pub assignee_id: Uuid,
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateTaskRequest {
    #[garde(dive)]
    pub subject: TaskSubjectField,
    #[garde(dive)]
    #[serde(default)]
    pub description: TaskDescriptionField,
    #[garde(skip)]
    pub assignee_id: Option<Uuid>,
}
