use axum::Router;
use servicez::app::build_router;
use servicez::app::telemetry;
use servicez::config::Settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load settings
    let settings = Settings::load()?;

    // 2. Initialize telemetry (adapter selection based on config + features)
    telemetry::init(&settings)?;

    // 3. Log intent before any blocking I/O so startup is always visible
    let addr = settings.socket_addr()?;
    tracing::info!(%addr, "starting server");

    // 4. Build router — connects to DB, wires services
    let app: Router = build_router(&settings).await?;

    // 5. Bind and serve
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}
