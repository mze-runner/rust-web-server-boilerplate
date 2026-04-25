use std::future::Future;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use servicez_domain::{
    error::DomainError,
    ports::UserRepository,
    user::{Email, User, UserId, Username},
};

// ---------------------------------------------------------------------------
// Shared row type and helpers
// ---------------------------------------------------------------------------

/// Raw row returned by SQLx — mapped explicitly to domain types.
#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User::from_row(
            UserId::from_uuid(row.id),
            Username::new(&row.username)?,
            Email::new(&row.email)?,
            row.created_at,
            row.updated_at,
        ))
    }
}

fn db_err(op: &str, e: sqlx::Error) -> DomainError {
    DomainError::Repository {
        operation: op.to_owned(),
        message: e.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Pool-backed repository — read-only use-cases and health check
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Lightweight connectivity check — used by the health endpoint.
    pub async fn ping(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("postgres ping failed: {e}"))
    }
}

impl UserRepository for PostgresUserRepository {
    fn find_by_id(
        &self,
        id: &UserId,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send {
        let pool = self.pool.clone();
        let uuid = *id.as_uuid();

        async move {
            let row: Option<UserRow> = sqlx::query_as(
                "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
            )
            .bind(uuid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| db_err("find_by_id", e))?;

            row.map(User::try_from).transpose()
        }
    }

    fn list_all(&self) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send {
        let pool = self.pool.clone();

        async move {
            let rows: Vec<UserRow> = sqlx::query_as(
                "SELECT id, username, email, created_at, updated_at FROM users ORDER BY username",
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| db_err("list_all", e))?;

            rows.into_iter().map(User::try_from).collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Transaction-backed repository — mutating use-cases inside a UnitOfWork
// ---------------------------------------------------------------------------

/// Wraps the shared transaction handle owned by `PostgresUnitOfWork`.
/// Only constructed by the UoW factory; never instantiated directly.
///
/// `Transaction<'static, Postgres>` is valid because `PgPool::begin()` acquires
/// a connection from the pool (which is `'static`) and transfers ownership into
/// the transaction — no borrowed lifetime required.
pub struct TxUserRepository {
    tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl TxUserRepository {
    pub(crate) fn new(tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>) -> Self {
        Self { tx }
    }
}

impl UserRepository for TxUserRepository {
    fn find_by_id(
        &self,
        id: &UserId,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);
        let uuid = *id.as_uuid();

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "find_by_id".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            let row: Option<UserRow> = sqlx::query_as(
                "SELECT id, username, email, created_at, updated_at FROM users WHERE id = $1",
            )
            .bind(uuid)
            .fetch_optional(conn)
            .await
            .map_err(|e| db_err("find_by_id", e))?;

            row.map(User::try_from).transpose()
        }
    }

    fn list_all(&self) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send {
        let tx = Arc::clone(&self.tx);

        async move {
            let mut guard = tx.lock().await;
            let conn = guard
                .as_deref_mut()
                .ok_or_else(|| DomainError::Repository {
                    operation: "list_all".to_owned(),
                    message: "transaction already committed or rolled back".to_owned(),
                })?;

            let rows: Vec<UserRow> = sqlx::query_as(
                "SELECT id, username, email, created_at, updated_at FROM users ORDER BY username",
            )
            .fetch_all(conn)
            .await
            .map_err(|e| db_err("list_all", e))?;

            rows.into_iter().map(User::try_from).collect()
        }
    }
}
