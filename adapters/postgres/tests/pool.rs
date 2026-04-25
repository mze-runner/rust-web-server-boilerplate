//! Integration tests for PostgreSQL connectivity.

use std::time::Duration;

#[tokio::test]
async fn test_invalid_url_is_rejected() {
    // connect_lazy validates the URL format synchronously — no DB needed.
    let result = servicez_db_postgres::build_pool("not-a-url", 5, Duration::from_secs(5)).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires a running Postgres instance
async fn test_reachable_db_connects() {
    let result = servicez_db_postgres::build_pool(
        "postgresql://postgres:postgres@localhost/test",
        5,
        Duration::from_secs(5),
    )
    .await;
    assert!(
        result.is_ok(),
        "expected connection to succeed: {:?}",
        result
    );
}

#[tokio::test]
#[ignore] // Requires a running Postgres instance
async fn test_unreachable_host_fails_within_timeout() {
    let start = std::time::Instant::now();
    let result = servicez_db_postgres::build_pool(
        "postgresql://postgres:postgres@127.0.0.1:19999/test",
        5,
        Duration::from_secs(2),
    )
    .await;
    assert!(result.is_err());
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "should fail fast within acquire_timeout"
    );
}
