use std::sync::Arc;

use servicez_http::routes::ServiceHealth;

use super::state::AppState;

impl ServiceHealth for AppState {
    fn health_check(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        let svc = Arc::clone(&self.user_service);
        async move { svc.repo().ping().await }
    }
}
