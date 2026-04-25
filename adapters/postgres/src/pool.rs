use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Build the shared `PgPool` from raw settings values.
///
/// Called once at startup by the composition root (`src/app/state.rs`).
/// `connect_lazy` validates the URL format synchronously — no network I/O
/// occurs here. Connectivity is verified by the health endpoint instead,
/// which lets Kubernetes readiness probes gate traffic rather than relying
/// on crash-restart loops to wait for the DB.
///
/// Runtime resilience:
/// - `test_before_acquire` validates each connection before use — stale
///   connections after a DB blip are discarded rather than causing query errors
/// - `idle_timeout` drops idle connections so the pool self-heals after outages
/// - `max_lifetime` rotates connections to avoid server-side timeouts
pub async fn build_pool(
    url: &str,
    max_connections: u32,
    acquire_timeout: Duration,
) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout)
        .test_before_acquire(true)
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_lazy(url)
        .map_err(|e| anyhow::anyhow!("postgres pool configuration error: {e}"))?;

    tracing::info!("postgres connection pool configured (max_connections={max_connections})");

    Ok(pool)
}
