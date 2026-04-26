use chrono::{DateTime, Utc};
use std::str::FromStr;

use crate::error::DomainError;
use crate::user::UserId;

crate::id::uuid_newtype!(TaskId);

impl FromStr for TaskStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Planning" => Ok(TaskStatus::Planning),
            "InProgress" => Ok(TaskStatus::InProgress),
            "Done" => Ok(TaskStatus::Done),
            "Cancelled" => Ok(TaskStatus::Cancelled),
            other => Err(DomainError::InvariantViolation {
                message: format!("unknown task status: {other}"),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Planning,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Planning => "Planning",
            TaskStatus::InProgress => "InProgress",
            TaskStatus::Done => "Done",
            TaskStatus::Cancelled => "Cancelled",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }

    pub fn can_transition_to(&self, next: &TaskStatus) -> bool {
        match self {
            TaskStatus::Planning => {
                matches!(
                    next,
                    TaskStatus::InProgress | TaskStatus::Done | TaskStatus::Cancelled
                )
            }
            TaskStatus::InProgress => {
                matches!(
                    next,
                    TaskStatus::Planning | TaskStatus::Done | TaskStatus::Cancelled
                )
            }
            TaskStatus::Done | TaskStatus::Cancelled => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskCursor {
    pub created_at: DateTime<Utc>,
    pub id: TaskId,
}

pub struct ListTasksQuery {
    pub caller_id: UserId,
    pub statuses: Vec<TaskStatus>,
    pub limit: u32,
    pub cursor: Option<TaskCursor>,
}

pub struct TaskPage {
    pub items: Vec<Task>,
    pub next_cursor: Option<TaskCursor>,
}

pub struct EditTaskCommand {
    pub subject: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<TaskStatus>,
}

#[derive(Clone, Debug)]
pub struct Task {
    id: TaskId,
    subject: String,
    description: Option<String>,
    status: TaskStatus,
    assignee_id: UserId,
    created_by: UserId,
    created_at: DateTime<Utc>,
    modified_by: UserId,
    modified_at: DateTime<Utc>,
}

impl Task {
    pub fn create(
        subject: String,
        description: Option<String>,
        assignee_id: UserId,
        created_by: UserId,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let subject = subject.trim().to_owned();
        if subject.is_empty() {
            return Err(DomainError::InvariantViolation {
                message: "subject must not be blank".into(),
            });
        }
        if subject.chars().count() > 256 {
            return Err(DomainError::InvariantViolation {
                message: "subject must be at most 256 characters".into(),
            });
        }
        if let Some(ref d) = description {
            if d.chars().count() > 500 {
                return Err(DomainError::InvariantViolation {
                    message: "description must be at most 500 characters".into(),
                });
            }
        }
        Ok(Self {
            id: TaskId::new(),
            subject,
            description,
            status: TaskStatus::Planning,
            assignee_id,
            created_by: created_by.clone(),
            created_at: now,
            modified_by: created_by,
            modified_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row(
        id: TaskId,
        subject: String,
        description: Option<String>,
        status: TaskStatus,
        assignee_id: UserId,
        created_by: UserId,
        created_at: DateTime<Utc>,
        modified_by: UserId,
        modified_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            subject,
            description,
            status,
            assignee_id,
            created_by,
            created_at,
            modified_by,
            modified_at,
        }
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn assignee_id(&self) -> &UserId {
        &self.assignee_id
    }

    pub fn created_by(&self) -> &UserId {
        &self.created_by
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn modified_by(&self) -> &UserId {
        &self.modified_by
    }

    pub fn modified_at(&self) -> DateTime<Utc> {
        self.modified_at
    }

    pub fn assign(
        &mut self,
        new_assignee_id: UserId,
        caller_id: &UserId,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if self.status.is_terminal() {
            return Err(DomainError::ImmutableResource);
        }
        if self.created_by != *caller_id {
            return Err(DomainError::Forbidden {
                reason: "only the task creator may reassign this task".into(),
            });
        }
        self.assignee_id = new_assignee_id;
        self.modified_by = caller_id.clone();
        self.modified_at = now;
        Ok(())
    }

    pub fn modify(
        &mut self,
        cmd: &EditTaskCommand,
        caller_id: &UserId,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if self.status.is_terminal() {
            return Err(DomainError::ImmutableResource);
        }

        let is_creator = self.created_by == *caller_id;
        let is_assignee = self.assignee_id == *caller_id;

        if !is_creator && !is_assignee {
            return Err(DomainError::Forbidden {
                reason: "caller is neither the task creator nor the current assignee".into(),
            });
        }

        if cmd.subject.is_some() && !is_creator {
            return Err(DomainError::Forbidden {
                reason: "only the task creator may change the subject".into(),
            });
        }

        if let Some(ref next) = cmd.status {
            if !self.status.can_transition_to(next) {
                return Err(DomainError::InvalidStateTransition {
                    from: self.status.as_str().into(),
                    to: next.as_str().into(),
                    reason: "not a permitted transition".into(),
                });
            }
        }

        if let Some(ref subject) = cmd.subject {
            let trimmed = subject.trim().to_owned();
            if trimmed.is_empty() {
                return Err(DomainError::InvariantViolation {
                    message: "subject must not be blank".into(),
                });
            }
            if trimmed.chars().count() > 256 {
                return Err(DomainError::InvariantViolation {
                    message: "subject must be at most 256 characters".into(),
                });
            }
            self.subject = trimmed;
        }

        if let Some(ref description) = cmd.description {
            if let Some(ref d) = description {
                if d.chars().count() > 500 {
                    return Err(DomainError::InvariantViolation {
                        message: "description must be at most 500 characters".into(),
                    });
                }
            }
            self.description = description.clone();
        }

        if let Some(status) = cmd.status.clone() {
            self.status = status;
        }

        self.modified_by = caller_id.clone();
        self.modified_at = now;

        Ok(())
    }
}
