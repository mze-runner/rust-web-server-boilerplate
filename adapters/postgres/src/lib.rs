//! PostgreSQL adapter implementation using SQLx.
pub mod error;
pub mod pool;
pub mod repos;
pub mod uow;

pub use pool::build_pool;
pub use repos::task::PostgresTaskRepository;
pub use uow::{PostgresUnitOfWork, PostgresUowFactory};

pub async fn run_migrations(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::migrate!()
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))
}
