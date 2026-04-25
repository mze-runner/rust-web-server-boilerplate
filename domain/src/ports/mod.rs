use std::future::Future;

use crate::error::DomainError;
use crate::task::{Task, ListTasksQuery, TaskPage};
use crate::user::{User, UserId};

/// Read-only access to pre-seeded users.
/// Implementations live in the adapter layer (e.g. `PostgresUserRepository`).
pub trait UserRepository: Send + Sync {
    fn find_by_id(
        &self,
        id: &UserId,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;

    fn list_all(&self) -> impl Future<Output = Result<Vec<User>, DomainError>> + Send;
}

/// Write access to tasks.
pub trait TaskRepository: Send + Sync {
    fn create(&self, task: &Task) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn find_by_id(
        &self,
        id: &crate::task::TaskId,
    ) -> impl Future<Output = Result<Option<Task>, DomainError>> + Send;

    fn update(&self, task: &Task) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn list_for_user(
        &self,
        query: &ListTasksQuery,
    ) -> impl Future<Output = Result<TaskPage, DomainError>> + Send;
}

/// Transactional unit of work. Owns a set of repositories that all share the
/// same underlying database transaction. Commit or rollback ends the transaction.
pub trait UnitOfWork: Send {
    type Users: UserRepository;
    type Tasks: TaskRepository;

    fn users(&mut self) -> &mut Self::Users;
    fn tasks(&mut self) -> &mut Self::Tasks;

    fn commit(self) -> impl Future<Output = Result<(), DomainError>> + Send;
    fn rollback(self) -> impl Future<Output = Result<(), DomainError>> + Send;
}

/// Creates `UnitOfWork` instances. Implementations hold the shared pool.
/// Lives in `AppState` behind an `Arc` — must be `Send + Sync`.
pub trait UnitOfWorkFactory: Send + Sync {
    type Uow: UnitOfWork;

    fn begin(&self) -> impl Future<Output = Result<Self::Uow, DomainError>> + Send;
}
