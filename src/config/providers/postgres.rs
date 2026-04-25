use serde::Deserialize;

/// PostgreSQL database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    /// Connection URL in format: postgresql://user:pass@host:port/database
    /// Example: postgresql://postgres:password@localhost:5432/mydb
    pub url: String,
    /// Maximum number of connections in the pool (1-1000, default: 10)
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
    /// Connection timeout in seconds (default: 30)
    #[serde(default = "default_timeout_seconds")]
    pub connection_timeout_seconds: u64,
}

impl PostgresConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.url.is_empty() {
            anyhow::bail!("Postgres URL cannot be empty");
        }

        if self.max_connections == 0 {
            anyhow::bail!("Postgres max_connections must be > 0");
        }

        if self.max_connections > 1000 {
            anyhow::bail!("Postgres max_connections too large (max 1000)");
        }

        if self.connection_timeout_seconds == 0 {
            anyhow::bail!("Postgres connection_timeout_seconds must be > 0");
        }

        Ok(())
    }
}

fn default_pool_size() -> u32 {
    10
}

fn default_timeout_seconds() -> u64 {
    30
}
