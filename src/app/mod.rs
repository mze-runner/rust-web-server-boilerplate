use crate::{
    config::{Environment, Settings},
    http,
};
use axum::{http::HeaderValue, middleware, Router};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod health;
pub mod state;
pub mod tasks;
pub mod telemetry;

pub async fn build_router(settings: &Settings) -> anyhow::Result<Router> {
    let state = Arc::new(state::AppState::new(settings.clone()).await?);
    let cors = build_cors(settings)?;

    let app = http::routes::router::<state::AppState>()
        .layer(middleware::from_fn(http::middleware::trace_id))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    Ok(app)
}

fn build_cors(settings: &Settings) -> anyhow::Result<CorsLayer> {
    #[cfg(debug_assertions)]
    if settings.env == Environment::Development {
        tracing::warn!("CORS policy is permissive — debug build + development env only");
        return Ok(CorsLayer::permissive());
    }

    let origins = settings
        .security
        .cors_allowed_origins
        .iter()
        .map(|o| {
            o.parse::<HeaderValue>()
                .map_err(|e| anyhow::anyhow!("invalid CORS origin '{o}': {e}"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(CorsLayer::new().allow_origin(origins))
}
