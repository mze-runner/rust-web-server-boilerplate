use std::sync::Arc;

use servicez_domain::{
    ports::{UnitOfWorkFactory, UserRepository},
    user::{User, UserId},
};

use crate::error::AppError;

pub struct UserService<R, F>
where
    R: UserRepository,
    F: UnitOfWorkFactory,
{
    repo: Arc<R>,
    uow_factory: Arc<F>,
}

impl<R, F> UserService<R, F>
where
    R: UserRepository,
    F: UnitOfWorkFactory,
{
    pub fn new(repo: Arc<R>, uow_factory: Arc<F>) -> Self {
        Self { repo, uow_factory }
    }

    pub fn repo(&self) -> &R {
        &self.repo
    }

    pub fn uow_factory(&self) -> &F {
        &self.uow_factory
    }

    /// Find a user by id. Returns `AppError::NotFound` when absent.
    pub async fn find_by_id(&self, id: &UserId) -> Result<User, AppError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(AppError::Domain)?
            .ok_or_else(|| AppError::NotFound {
                resource_type: format!("User({})", id),
            })
    }

    /// List all users — used for task assignment selection.
    pub async fn list_users(&self) -> Result<Vec<User>, AppError> {
        self.repo.list_all().await.map_err(AppError::Domain)
    }
}
