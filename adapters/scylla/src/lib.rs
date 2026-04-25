//! ScyllaDB adapter implementation
//!
//! This adapter implements domain repository ports using ScyllaDB
//! as the underlying database.

use anyhow::{Context, Result};
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;

/// ScyllaDB session wrapper
pub struct ScyllaSession {
    session: Session,
}

impl ScyllaSession {
    /// Create a new ScyllaDB session from a list of known nodes
    ///
    /// # Arguments
    /// * `nodes` - Slice of node addresses in "host:port" format (e.g., "127.0.0.1:9042")
    ///
    /// # Errors
    /// Returns error if:
    /// - Node list is empty
    /// - Connection to cluster fails
    pub async fn new(nodes: &[&str]) -> Result<Self> {
        if nodes.is_empty() {
            anyhow::bail!("ScyllaDB node list cannot be empty");
        }

        let mut builder = SessionBuilder::new();
        for node in nodes {
            builder = builder.known_node(node);
        }

        let session = builder
            .build()
            .await
            .context("Failed to build ScyllaDB session")?;

        Ok(Self { session })
    }

    /// Get a reference to the underlying session
    pub fn inner(&self) -> &Session {
        &self.session
    }
}
