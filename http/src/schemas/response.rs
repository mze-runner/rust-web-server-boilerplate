use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use servicez_domain::task::Task;
use servicez_domain::task_comment::TaskComment;

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub subject: String,
    pub description: Option<String>,
    pub status: String,
    pub assignee_id: Uuid,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_by: Uuid,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub modified_by: Uuid,
    pub modified_at: DateTime<Utc>,
}

impl From<TaskComment> for CommentResponse {
    fn from(c: TaskComment) -> Self {
        Self {
            id: *c.id().as_uuid(),
            task_id: *c.task_id().as_uuid(),
            author_id: *c.author_id().as_uuid(),
            body: c.body().to_owned(),
            created_at: c.created_at(),
            modified_by: *c.modified_by().as_uuid(),
            modified_at: c.modified_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub items: Vec<CommentResponse>,
}

#[derive(Debug, Serialize)]
pub struct TaskPageResponse {
    pub items: Vec<TaskResponse>,
    pub next_cursor: Option<String>,
}

impl From<Task> for TaskResponse {
    fn from(t: Task) -> Self {
        Self {
            id: *t.id().as_uuid(),
            subject: t.subject().to_owned(),
            description: t.description().map(|s| s.to_owned()),
            status: t.status().as_str().to_owned(),
            assignee_id: *t.assignee_id().as_uuid(),
            created_by: *t.created_by().as_uuid(),
            created_at: t.created_at(),
            modified_by: *t.modified_by().as_uuid(),
            modified_at: t.modified_at(),
        }
    }
}
