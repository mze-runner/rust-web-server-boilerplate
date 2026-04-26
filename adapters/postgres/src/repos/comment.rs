use std::sync::Arc;

use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use servicez_domain::{
    error::DomainError,
    ports::TaskCommentRepository,
    task_comment::{CommentCursor, CommentPage, ListCommentsQuery, TaskComment, TaskCommentId},
};

fn db_err(op: &str, e: sqlx::Error) -> DomainError {
    DomainError::Repository {
        operation: op.to_owned(),
        message: e.to_string(),
    }
}

fn build_list_sql(has_cursor: bool) -> String {
    let mut sql = String::from(
        "SELECT id, task_id, author_id, body, created_at, modified_by, modified_at \
         FROM task_comments WHERE task_id = $1",
    );
    if has_cursor {
        sql.push_str(" AND (created_at, id) > ($2, $3)");
    }
    sql.push_str(" ORDER BY created_at ASC, id ASC");
    let limit_idx = if has_cursor { 4 } else { 2 };
    sql.push_str(&format!(" LIMIT ${limit_idx}"));
    sql
}

fn build_page(mut rows: Vec<TaskCommentRow>, limit: u32) -> CommentPage {
    let has_next = rows.len() > limit as usize;
    if has_next {
        rows.truncate(limit as usize);
    }
    let items: Vec<TaskComment> = rows.into_iter().map(TaskComment::from).collect();
    let next_cursor = if has_next {
        items.last().map(|c| CommentCursor {
            created_at: c.created_at(),
            id: c.id().clone(),
        })
    } else {
        None
    };
    CommentPage { items, next_cursor }
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

    fn find_by_id(
        &self,
        id: &TaskCommentId,
    ) -> impl std::future::Future<Output = Result<Option<TaskComment>, DomainError>> + Send {
        let pool = self.pool.clone();
        let id_uuid = *id.as_uuid();

        async move {
            sqlx::query_as::<_, TaskCommentRow>(
                "SELECT id, task_id, author_id, body, created_at, modified_by, modified_at \
                 FROM task_comments WHERE id = $1",
            )
            .bind(id_uuid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| db_err("find_by_id", e))
            .map(|opt| opt.map(TaskComment::from))
        }
    }

    fn update(
        &self,
        comment: &TaskComment,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let pool = self.pool.clone();
        let id = *comment.id().as_uuid();
        let body = comment.body().to_owned();
        let modified_by = *comment.modified_by().as_uuid();
        let modified_at = comment.modified_at();

        async move {
            sqlx::query(
                "UPDATE task_comments SET body = $2, modified_by = $3, modified_at = $4 \
                 WHERE id = $1",
            )
            .bind(id)
            .bind(body)
            .bind(modified_by)
            .bind(modified_at)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| db_err("update", e))
        }
    }

    fn list_for_task(
        &self,
        query: &ListCommentsQuery,
    ) -> impl std::future::Future<Output = Result<CommentPage, DomainError>> + Send {
        let pool = self.pool.clone();
        let task_id_uuid = *query.task_id.as_uuid();
        let limit = query.limit;
        let cursor = query
            .cursor
            .as_ref()
            .map(|c| (c.created_at, *c.id.as_uuid()));

        async move {
            let sql = build_list_sql(cursor.is_some());
            let mut q = sqlx::query_as::<_, TaskCommentRow>(&sql).bind(task_id_uuid);
            if let Some((cur_at, cur_id)) = cursor {
                q = q.bind(cur_at).bind(cur_id);
            }
            q = q.bind(limit as i64 + 1);

            let rows = q
                .fetch_all(&pool)
                .await
                .map_err(|e| db_err("list_for_task", e))?;

            Ok(build_page(rows, limit))
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
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
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

    fn find_by_id(
        &self,
        id: &TaskCommentId,
    ) -> impl std::future::Future<Output = Result<Option<TaskComment>, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id_uuid = *id.as_uuid();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "find_by_id".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            sqlx::query_as::<_, TaskCommentRow>(
                "SELECT id, task_id, author_id, body, created_at, modified_by, modified_at \
                 FROM task_comments WHERE id = $1",
            )
            .bind(id_uuid)
            .fetch_optional(conn)
            .await
            .map_err(|e| db_err("find_by_id", e))
            .map(|opt| opt.map(TaskComment::from))
        }
    }

    fn update(
        &self,
        comment: &TaskComment,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id = *comment.id().as_uuid();
        let body = comment.body().to_owned();
        let modified_by = *comment.modified_by().as_uuid();
        let modified_at = comment.modified_at();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "update".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            sqlx::query(
                "UPDATE task_comments SET body = $2, modified_by = $3, modified_at = $4 \
                 WHERE id = $1",
            )
            .bind(id)
            .bind(body)
            .bind(modified_by)
            .bind(modified_at)
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| db_err("update", e))
        }
    }

    fn list_for_task(
        &self,
        query: &ListCommentsQuery,
    ) -> impl std::future::Future<Output = Result<CommentPage, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let task_id_uuid = *query.task_id.as_uuid();
        let limit = query.limit;
        let cursor = query
            .cursor
            .as_ref()
            .map(|c| (c.created_at, *c.id.as_uuid()));

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "list_for_task".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            let sql = build_list_sql(cursor.is_some());
            let mut q = sqlx::query_as::<_, TaskCommentRow>(&sql).bind(task_id_uuid);
            if let Some((cur_at, cur_id)) = cursor {
                q = q.bind(cur_at).bind(cur_id);
            }
            q = q.bind(limit as i64 + 1);

            let rows = q
                .fetch_all(conn)
                .await
                .map_err(|e| db_err("list_for_task", e))?;

            Ok(build_page(rows, limit))
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
        use servicez_domain::{task::TaskId, user::UserId};
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
