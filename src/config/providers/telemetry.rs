use crate::config::Validatable;
use serde::Deserialize;

/// Telemetry provider selection
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryProvider {
    #[cfg(feature = "telemetry-console")]
    Console,
    // Future providers (examples):
    // #[cfg(feature = "telemetry-otlp")]
    // Otlp,
    // #[cfg(feature = "telemetry-datadog")]
    // Datadog,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum TelemetryConfig {
    #[cfg(feature = "telemetry-console")]
    Console(ConsoleConfig),
}

#[cfg(feature = "telemetry-console")]
#[derive(Debug, Clone, Deserialize)]
pub struct ConsoleConfig {
    /// Output format: "json" or "pretty"
    pub log_format: LogFormat,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Validatable for TelemetryConfig {
    fn validate(&self) -> anyhow::Result<()> {
        // Provider-specific validation happens here
        Ok(())
    }
}
