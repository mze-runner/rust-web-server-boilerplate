use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::task::TaskId;
use crate::user::UserId;

crate::id::uuid_newtype!(TaskCommentId);

#[derive(Clone, Debug)]
pub struct TaskComment {
    id: TaskCommentId,
    task_id: TaskId,
    author_id: UserId,
    body: String,
    created_at: DateTime<Utc>,
    modified_by: UserId,
    modified_at: DateTime<Utc>,
}

impl TaskComment {
    pub fn create(
        task_id: TaskId,
        author_id: UserId,
        body: String,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let body = body.trim().to_owned();
        if body.is_empty() {
            return Err(DomainError::InvariantViolation {
                message: "body must not be blank".into(),
            });
        }
        if body.chars().count() > 1000 {
            return Err(DomainError::InvariantViolation {
                message: "body must be at most 1000 characters".into(),
            });
        }
        Ok(Self {
            id: TaskCommentId::new(),
            task_id,
            author_id: author_id.clone(),
            body,
            created_at: now,
            modified_by: author_id,
            modified_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row(
        id: TaskCommentId,
        task_id: TaskId,
        author_id: UserId,
        body: String,
        created_at: DateTime<Utc>,
        modified_by: UserId,
        modified_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            task_id,
            author_id,
            body,
            created_at,
            modified_by,
            modified_at,
        }
    }

    pub fn id(&self) -> &TaskCommentId {
        &self.id
    }

    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    pub fn author_id(&self) -> &UserId {
        &self.author_id
    }

    pub fn body(&self) -> &str {
        &self.body
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
}
