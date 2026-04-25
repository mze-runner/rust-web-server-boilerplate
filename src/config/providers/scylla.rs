use serde::Deserialize;

/// ScyllaDB database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ScyllaConfig {
    /// List of Scylla node addresses in format: ["host:port"]
    /// Example: ["127.0.0.1:9042", "127.0.0.2:9042"]
    pub nodes: Vec<String>,
    /// Keyspace name to use
    pub keyspace: String,
    /// Maximum number of connections (1-1000, default: 10)
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
}

impl ScyllaConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.nodes.is_empty() {
            anyhow::bail!("ScyllaDB nodes list cannot be empty");
        }

        for node in &self.nodes {
            if node.is_empty() {
                anyhow::bail!("ScyllaDB node address cannot be empty");
            }
        }

        if self.keyspace.is_empty() {
            anyhow::bail!("ScyllaDB keyspace cannot be empty");
        }

        if self.max_connections == 0 {
            anyhow::bail!("ScyllaDB max_connections must be > 0");
        }

        if self.max_connections > 1000 {
            anyhow::bail!("ScyllaDB max_connections too large (max 1000)");
        }

        Ok(())
    }
}

fn default_pool_size() -> u32 {
    10
}
