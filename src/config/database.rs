use serde::Deserialize;

// Import the Validatable trait from parent module
use super::Validatable;

// Import providers based on features
#[cfg(feature = "postgres")]
use super::providers::PostgresConfig;

#[cfg(feature = "scylla")]
use super::providers::ScyllaConfig;

// Ensure only one database adapter is enabled at compile time
#[cfg(all(feature = "postgres", feature = "scylla"))]
compile_error!("Cannot enable both 'postgres' and 'scylla' features simultaneously. Choose one.");

#[cfg(not(any(feature = "postgres", feature = "scylla")))]
compile_error!("At least one database feature must be enabled: 'postgres' or 'scylla'");

/// Database configuration wrapper
///
/// Combines provider selection with provider-specific configuration.
/// The provider field determines which database backend to use,
/// and the config field contains provider-specific settings.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// The database provider to use
    pub provider: DatabaseProvider,
    /// Provider-specific configuration (automatically matched via serde)
    #[serde(flatten)]
    pub config: ProviderConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseProvider {
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "scylla")]
    Scylla,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ProviderConfig {
    #[cfg(feature = "postgres")]
    Postgres(PostgresConfig),
    #[cfg(feature = "scylla")]
    Scylla(ScyllaConfig),
}

impl Validatable for DatabaseConfig {
    fn validate(&self) -> anyhow::Result<()> {
        // Ensure provider matches config variant
        match (&self.provider, &self.config) {
            #[cfg(feature = "postgres")]
            (DatabaseProvider::Postgres, ProviderConfig::Postgres(cfg)) => cfg.validate()?,
            #[cfg(feature = "scylla")]
            (DatabaseProvider::Scylla, ProviderConfig::Scylla(cfg)) => cfg.validate()?,
            #[allow(unreachable_patterns)]
            _ => anyhow::bail!("Database provider/config mismatch"),
        }
        Ok(())
    }
}
