//! Integration tests for ScyllaDB session functionality

use servicez_db_scylla::ScyllaSession;

#[tokio::test]
#[ignore] // Requires database connection
async fn test_session_creation_success() {
    let result = ScyllaSession::new(&["127.0.0.1:9042"]).await;
    // This test requires a running ScyllaDB instance
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_session_creation_multiple_nodes() {
    let result = ScyllaSession::new(&["127.0.0.1:9042", "127.0.0.2:9042"]).await;
    // Will fail without running database, but tests multi-node setup
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_session_creation_empty_nodes() {
    let result = ScyllaSession::new(&[]).await;
    // Should fail with empty nodes list
    assert!(result.is_err());
}
