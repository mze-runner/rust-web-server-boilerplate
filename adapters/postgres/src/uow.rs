use std::sync::Arc;

use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;

use servicez_domain::{
    error::DomainError,
    ports::{UnitOfWork, UnitOfWorkFactory},
};

use crate::error::map_sqlx_error;
use crate::repos::task::TxTaskRepository;
use crate::repos::user::TxUserRepository;

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

/// Holds the shared pool. `PgPool` is internally arc-ed — cloning is free.
/// Lives in `AppState` behind an `Arc<PostgresUowFactory>`.
#[derive(Clone, Debug)]
pub struct PostgresUowFactory {
    pool: PgPool,
}

impl PostgresUowFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UnitOfWorkFactory for PostgresUowFactory {
    type Uow = PostgresUnitOfWork;

    async fn begin(&self) -> Result<Self::Uow, DomainError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| map_sqlx_error(e, "begin transaction"))?;

        let tx = Arc::new(Mutex::new(Some(tx)));
        let users = TxUserRepository::new(Arc::clone(&tx));
        let tasks = TxTaskRepository::new(Arc::clone(&tx));

        Ok(PostgresUnitOfWork { tx, users, tasks })
    }
}

// ---------------------------------------------------------------------------
// Unit of Work
// ---------------------------------------------------------------------------

pub struct PostgresUnitOfWork {
    tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
    users: TxUserRepository,
    tasks: TxTaskRepository,
}

impl UnitOfWork for PostgresUnitOfWork {
    type Users = TxUserRepository;
    type Tasks = TxTaskRepository;

    fn users(&mut self) -> &mut Self::Users {
        &mut self.users
    }

    fn tasks(&mut self) -> &mut Self::Tasks {
        &mut self.tasks
    }

    async fn commit(self) -> Result<(), DomainError> {
        let tx = {
            let mut guard = self.tx.lock().await;
            guard.take().ok_or_else(|| DomainError::Repository {
                operation: "commit".to_owned(),
                message: "transaction already committed or rolled back".to_owned(),
            })?
        };
        tx.commit().await.map_err(|e| map_sqlx_error(e, "commit"))
    }

    async fn rollback(self) -> Result<(), DomainError> {
        let tx = {
            let mut guard = self.tx.lock().await;
            guard.take().ok_or_else(|| DomainError::Repository {
                operation: "rollback".to_owned(),
                message: "transaction already committed or rolled back".to_owned(),
            })?
        };
        tx.rollback()
            .await
            .map_err(|e| map_sqlx_error(e, "rollback"))
    }
}
