pub mod tasks;

use std::future::Future;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::middleware::authenticate;
use tasks::TaskOperations;

/// Port contract for the health check handler.
/// Defined here so the HTTP adapter owns what it needs from the application state —
/// implemented by `AppState` in the composition root.
pub trait ServiceHealth: Clone + Send + Sync + 'static {
    fn health_check(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

/// Complete routing table — every URL and access level defined in one place.
///
/// Public routes:   accessible without authentication
/// Protected routes: require a valid Bearer token (authenticate middleware)
pub fn router<S>() -> Router<Arc<S>>
where
    S: ServiceHealth + TaskOperations,
{
    let public = Router::new()
        .route("/api/v1/healthz", get(healthz::<S>));

    let protected = Router::new()
        .nest("/api/v1/tasks", tasks::router::<S>())
        .layer(middleware::from_fn(authenticate));

    Router::new()
        .merge(public)
        .merge(protected)
        .fallback(handler_404)
}

/// Health check — verifies the service and its dependencies are reachable.
/// Returns 200 when healthy, 503 when a dependency is down.
async fn healthz<S: ServiceHealth>(State(state): State<Arc<S>>) -> Response {
    match state.health_check().await {
        Ok(_) => (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response(),
        Err(e) => {
            tracing::error!("health check failed: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "unavailable", "detail": "database unreachable" })),
            )
                .into_response()
        }
    }
}

async fn handler_404() -> Response {
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}
