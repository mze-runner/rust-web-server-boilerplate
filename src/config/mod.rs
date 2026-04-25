use serde::Deserialize;
use std::net::SocketAddr;

pub mod database;
pub mod providers;
pub mod security;
pub mod server;

pub use database::{DatabaseConfig, DatabaseProvider, ProviderConfig};
// pub use observability::{LogFormat, Observability};
pub use providers::telemetry::{LogFormat, TelemetryConfig, TelemetryProvider};
pub use security::Security;
pub use server::Server;

/// Top-level settings loaded from:
/// - Settings.toml (optional)
/// - Settings.local.toml (optional; ignored in prod builds)
/// - Environment variables with prefix APP__  (double underscore = nesting)
///
/// Example: APP__SERVER__PORT=8080  →  settings.server.port = 8080
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub env: Environment,
    pub server: Server,
    // pub observability: Observability,
    pub telemetry: TelemetryConfig, // NEW
    pub database: DatabaseConfig,
    pub security: Security,
}

/// Trait for configuration types that can be validated
pub trait Validatable {
    /// Validate configuration values
    /// Returns Err if configuration is invalid with descriptive error message
    fn validate(&self) -> anyhow::Result<()>;
}

// Environment enum
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Settings {
    /// Load layered configuration:
    /// - Settings.toml (optional)
    /// - Settings.local.toml (optional)
    /// - ENV (APP__*)
    pub fn load() -> anyhow::Result<Self> {
        use config::{Config, ConfigError, Environment, File};

        let builder = config::Config::builder()
            // Base (optional)
            .add_source(File::with_name("Settings").required(false))
            // Local overrides (optional; useful for dev)
            .add_source(File::with_name("Settings.local").required(false))
            // Environment variables: APP__SERVER__PORT=8080 etc.
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );

        let cfg: Config = builder
            .build()
            .map_err(|e: ConfigError| anyhow::anyhow!(e))?;

        let s: Self = cfg.try_deserialize()?;

        // Validate before returning
        s.validate()?;

        Ok(s)
    }

    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.server.addr, self.server.port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid socket address: {}", e))
    }

    /// Validate settings after loading
    pub fn validate(&self) -> anyhow::Result<()> {
        self.server.validate()?;
        self.database.validate()?;
        self.security.validate()?;
        self.telemetry.validate()?;

        Ok(())
    }
}
