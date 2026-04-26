use std::sync::Arc;
use std::time::Duration;

use servicez_application::services::{task_service::TaskService, user_service::UserService};
use servicez_db_postgres::{
    build_pool, repos::task::PostgresTaskRepository, repos::user::PostgresUserRepository,
    run_migrations, PostgresUowFactory,
};

use crate::config::Settings;

/// Application state — holds only services. Settings are consumed at wiring time.
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService<PostgresUserRepository, PostgresUowFactory>>,
    pub task_service: Arc<TaskService<PostgresTaskRepository, PostgresUowFactory>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

impl AppState {
    pub async fn new(settings: Settings) -> anyhow::Result<Self> {
        use crate::config::{DatabaseProvider, ProviderConfig};

        let (url, max_connections, connect_timeout) =
            match (&settings.database.provider, &settings.database.config) {
                #[cfg(feature = "postgres")]
                (DatabaseProvider::Postgres, ProviderConfig::Postgres(cfg)) => (
                    cfg.url.as_str(),
                    cfg.max_connections,
                    Duration::from_secs(cfg.connection_timeout_seconds),
                ),
                #[allow(unreachable_patterns)]
                _ => anyhow::bail!("unsupported database provider for this build"),
            };

        tracing::info!("initializing application state");

        // One pool shared by both the read-side repo and the UoW factory.
        // `PgPool` is internally arc-ed — cloning it is free.
        let pool = build_pool(url, max_connections, connect_timeout).await?;

        run_migrations(&pool).await?;

        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let task_repo = Arc::new(PostgresTaskRepository::new(pool.clone()));
        let uow_factory = Arc::new(PostgresUowFactory::new(pool));

        let user_service = Arc::new(UserService::new(user_repo, Arc::clone(&uow_factory)));
        let task_service = Arc::new(TaskService::new(task_repo, Arc::clone(&uow_factory)));

        tracing::info!("application state initialized");

        Ok(Self {
            user_service,
            task_service,
        })
    }
}
