use std::sync::Arc;

use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use servicez_domain::{
    error::DomainError,
    ports::TaskCommentRepository,
    task_comment::TaskComment,
};

fn db_err(op: &str, e: sqlx::Error) -> DomainError {
    DomainError::Repository {
        operation: op.to_owned(),
        message: e.to_string(),
    }
}

// ===== Pool-backed repository =====

#[derive(Debug, Clone)]
pub struct PostgresTaskCommentRepository {
    pool: PgPool,
}

impl PostgresTaskCommentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaskCommentRepository for PostgresTaskCommentRepository {
    fn create(
        &self,
        comment: &TaskComment,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let pool = self.pool.clone();
        let id = *comment.id().as_uuid();
        let task_id = *comment.task_id().as_uuid();
        let author_id = *comment.author_id().as_uuid();
        let body = comment.body().to_owned();
        let created_at = comment.created_at();
        let modified_by = *comment.modified_by().as_uuid();
        let modified_at = comment.modified_at();

        async move {
            sqlx::query(
                "INSERT INTO task_comments \
                 (id, task_id, author_id, body, created_at, modified_by, modified_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(task_id)
            .bind(author_id)
            .bind(body)
            .bind(created_at)
            .bind(modified_by)
            .bind(modified_at)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| db_err("create", e))
        }
    }
}

// ===== Transaction-backed repository =====

pub struct TxTaskCommentRepository {
    tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl TxTaskCommentRepository {
    pub(crate) fn new(tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>) -> Self {
        Self { tx }
    }
}

impl TaskCommentRepository for TxTaskCommentRepository {
    fn create(
        &self,
        comment: &TaskComment,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id = *comment.id().as_uuid();
        let task_id = *comment.task_id().as_uuid();
        let author_id = *comment.author_id().as_uuid();
        let body = comment.body().to_owned();
        let created_at = comment.created_at();
        let modified_by = *comment.modified_by().as_uuid();
        let modified_at = comment.modified_at();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard.as_deref_mut().ok_or_else(|| DomainError::Repository {
                operation: "create".to_owned(),
                message: "transaction already committed or rolled back".to_owned(),
            })?;

            sqlx::query(
                "INSERT INTO task_comments \
                 (id, task_id, author_id, body, created_at, modified_by, modified_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(id)
            .bind(task_id)
            .bind(author_id)
            .bind(body)
            .bind(created_at)
            .bind(modified_by)
            .bind(modified_at)
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| db_err("create", e))
        }
    }
}

// ===== Row type =====

#[derive(sqlx::FromRow)]
pub struct TaskCommentRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_by: Uuid,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

impl From<TaskCommentRow> for TaskComment {
    fn from(row: TaskCommentRow) -> Self {
        use servicez_domain::{task::TaskId, task_comment::TaskCommentId, user::UserId};
        TaskComment::from_row(
            TaskCommentId::from_uuid(row.id),
            TaskId::from_uuid(row.task_id),
            UserId::from_uuid(row.author_id),
            row.body,
            row.created_at,
            UserId::from_uuid(row.modified_by),
            row.modified_at,
        )
    }
}
