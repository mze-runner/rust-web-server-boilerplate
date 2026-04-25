use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use std::str::FromStr;

use servicez_domain::{
    error::DomainError,
    ports::TaskRepository,
    task::{ListTasksQuery, Task, TaskCursor, TaskId, TaskPage, TaskStatus},
    user::UserId,
};

fn db_err(op: &str, e: sqlx::Error) -> DomainError {
    DomainError::Repository {
        operation: op.to_owned(),
        message: e.to_string(),
    }
}

// ===== Pool-backed repository =====

#[derive(Debug, Clone)]
pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl PostgresTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl TaskRepository for PostgresTaskRepository {
    fn create(
        &self,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let pool = self.pool.clone();
        let id = *task.id().as_uuid();
        let subject = task.subject().to_owned();
        let description = task.description().map(|s| s.to_owned());
        let status = task.status().as_str().to_owned();
        let assignee_id = *task.assignee_id().as_uuid();
        let created_by = *task.created_by().as_uuid();
        let created_at = task.created_at();
        let modified_by = *task.modified_by().as_uuid();
        let modified_at = task.modified_at();

        async move {
            sqlx::query(
                "INSERT INTO tasks \
                 (id, subject, description, status, assignee_id, created_by, created_at, modified_by, modified_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(id)
            .bind(subject)
            .bind(description)
            .bind(status)
            .bind(assignee_id)
            .bind(created_by)
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
        id: &TaskId,
    ) -> impl std::future::Future<Output = Result<Option<Task>, DomainError>> + Send {
        let pool = self.pool.clone();
        let id = *id.as_uuid();

        async move {
            let row: Option<TaskRow> = sqlx::query_as(
                "SELECT id, subject, description, status, assignee_id, \
                 created_by, created_at, modified_by, modified_at \
                 FROM tasks WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| db_err("find_by_id", e))?;

            row.map(Task::try_from).transpose()
        }
    }

    fn update(
        &self,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let pool = self.pool.clone();
        let id = *task.id().as_uuid();
        let subject = task.subject().to_owned();
        let description = task.description().map(|s| s.to_owned());
        let status = task.status().as_str().to_owned();
        let assignee_id = *task.assignee_id().as_uuid();
        let modified_by = *task.modified_by().as_uuid();
        let modified_at = task.modified_at();

        async move {
            sqlx::query(
                "UPDATE tasks \
                 SET subject = $2, description = $3, status = $4, assignee_id = $5, \
                     modified_by = $6, modified_at = $7 \
                 WHERE id = $1",
            )
            .bind(id)
            .bind(subject)
            .bind(description)
            .bind(status)
            .bind(assignee_id)
            .bind(modified_by)
            .bind(modified_at)
            .execute(&pool)
            .await
            .map(|_| ())
            .map_err(|e| db_err("update", e))
        }
    }

    fn list_for_user(
        &self,
        query: &ListTasksQuery,
    ) -> impl std::future::Future<Output = Result<TaskPage, DomainError>> + Send {
        let pool = self.pool.clone();
        let uid = *query.caller_id.as_uuid();
        let statuses: Vec<String> = query.statuses.iter().map(|s| s.as_str().to_owned()).collect();
        let limit = query.limit as i64;
        let cursor = query.cursor.as_ref().map(|c| (c.created_at, *c.id.as_uuid()));

        async move {
            let mut sql = String::from(
                "SELECT id, subject, description, status, assignee_id, \
                 created_by, created_at, modified_by, modified_at \
                 FROM tasks WHERE (created_by = $1 OR assignee_id = $1)",
            );
            let mut idx: i32 = 2;

            if cursor.is_some() {
                sql.push_str(&format!(" AND (created_at, id) < (${idx}, ${})", idx + 1));
                idx += 2;
            }
            if !statuses.is_empty() {
                sql.push_str(&format!(" AND status = ANY(${idx})"));
                idx += 1;
            }
            sql.push_str(&format!(" ORDER BY created_at DESC, id DESC LIMIT ${idx}"));

            let mut q = sqlx::query_as::<_, TaskRow>(&sql).bind(uid);
            if let Some((cur_at, cur_id)) = cursor {
                q = q.bind(cur_at).bind(cur_id);
            }
            if !statuses.is_empty() {
                q = q.bind(statuses);
            }
            q = q.bind(limit + 1);

            let mut rows: Vec<TaskRow> = q
                .fetch_all(&pool)
                .await
                .map_err(|e| db_err("list_for_user", e))?;

            let has_next = rows.len() > limit as usize;
            if has_next {
                rows.truncate(limit as usize);
            }

            let items: Vec<Task> = rows
                .into_iter()
                .map(Task::try_from)
                .collect::<Result<_, _>>()?;

            let next_cursor = if has_next {
                items.last().map(|t| TaskCursor {
                    created_at: t.created_at(),
                    id: t.id().clone(),
                })
            } else {
                None
            };

            Ok(TaskPage { items, next_cursor })
        }
    }
}

// ===== Transaction-backed repository =====

pub struct TxTaskRepository {
    tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl TxTaskRepository {
    pub(crate) fn new(tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>) -> Self {
        Self { tx }
    }
}

impl TaskRepository for TxTaskRepository {
    fn create(
        &self,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id = *task.id().as_uuid();
        let subject = task.subject().to_owned();
        let description = task.description().map(|s| s.to_owned());
        let status = task.status().as_str().to_owned();
        let assignee_id = *task.assignee_id().as_uuid();
        let created_by = *task.created_by().as_uuid();
        let created_at = task.created_at();
        let modified_by = *task.modified_by().as_uuid();
        let modified_at = task.modified_at();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "create".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            sqlx::query(
                "INSERT INTO tasks \
                 (id, subject, description, status, assignee_id, created_by, created_at, modified_by, modified_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(id)
            .bind(subject)
            .bind(description)
            .bind(status)
            .bind(assignee_id)
            .bind(created_by)
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
        id: &TaskId,
    ) -> impl std::future::Future<Output = Result<Option<Task>, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id = *id.as_uuid();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "find_by_id".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            let row: Option<TaskRow> = sqlx::query_as(
                "SELECT id, subject, description, status, assignee_id, \
                 created_by, created_at, modified_by, modified_at \
                 FROM tasks WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(conn)
            .await
            .map_err(|e| db_err("find_by_id", e))?;

            row.map(Task::try_from).transpose()
        }
    }

    fn update(
        &self,
        task: &Task,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let id = *task.id().as_uuid();
        let subject = task.subject().to_owned();
        let description = task.description().map(|s| s.to_owned());
        let status = task.status().as_str().to_owned();
        let assignee_id = *task.assignee_id().as_uuid();
        let modified_by = *task.modified_by().as_uuid();
        let modified_at = task.modified_at();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "update".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            sqlx::query(
                "UPDATE tasks \
                 SET subject = $2, description = $3, status = $4, assignee_id = $5, \
                     modified_by = $6, modified_at = $7 \
                 WHERE id = $1",
            )
            .bind(id)
            .bind(subject)
            .bind(description)
            .bind(status)
            .bind(assignee_id)
            .bind(modified_by)
            .bind(modified_at)
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| db_err("update", e))
        }
    }

    fn list_for_user(
        &self,
        query: &ListTasksQuery,
    ) -> impl std::future::Future<Output = Result<TaskPage, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let uid = *query.caller_id.as_uuid();
        let statuses: Vec<String> = query.statuses.iter().map(|s| s.as_str().to_owned()).collect();
        let limit = query.limit as i64;
        let cursor = query.cursor.as_ref().map(|c| (c.created_at, *c.id.as_uuid()));

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "list_for_user".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            let mut sql = String::from(
                "SELECT id, subject, description, status, assignee_id, \
                 created_by, created_at, modified_by, modified_at \
                 FROM tasks WHERE (created_by = $1 OR assignee_id = $1)",
            );
            let mut idx: i32 = 2;

            if cursor.is_some() {
                sql.push_str(&format!(" AND (created_at, id) < (${idx}, ${})", idx + 1));
                idx += 2;
            }
            if !statuses.is_empty() {
                sql.push_str(&format!(" AND status = ANY(${idx})"));
                idx += 1;
            }
            sql.push_str(&format!(" ORDER BY created_at DESC, id DESC LIMIT ${idx}"));

            let mut q = sqlx::query_as::<_, TaskRow>(&sql).bind(uid);
            if let Some((cur_at, cur_id)) = cursor {
                q = q.bind(cur_at).bind(cur_id);
            }
            if !statuses.is_empty() {
                q = q.bind(statuses);
            }
            q = q.bind(limit + 1);

            let mut rows: Vec<TaskRow> = q
                .fetch_all(conn)
                .await
                .map_err(|e| db_err("list_for_user", e))?;

            let has_next = rows.len() > limit as usize;
            if has_next {
                rows.truncate(limit as usize);
            }

            let items: Vec<Task> = rows
                .into_iter()
                .map(Task::try_from)
                .collect::<Result<_, _>>()?;

            let next_cursor = if has_next {
                items.last().map(|t| TaskCursor {
                    created_at: t.created_at(),
                    id: t.id().clone(),
                })
            } else {
                None
            };

            Ok(TaskPage { items, next_cursor })
        }
    }
}

// ===== Row type =====

#[derive(FromRow)]
pub struct TaskRow {
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

impl TryFrom<TaskRow> for Task {
    type Error = DomainError;

    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        let status = TaskStatus::from_str(&row.status)?;
        Ok(Task::from_row(
            TaskId::from_uuid(row.id),
            row.subject,
            row.description,
            status,
            UserId::from_uuid(row.assignee_id),
            UserId::from_uuid(row.created_by),
            row.created_at,
            UserId::from_uuid(row.modified_by),
            row.modified_at,
        ))
    }
}
