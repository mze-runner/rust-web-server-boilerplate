use crate::config::{Settings, TelemetryConfig};

/// Initialize telemetry based on configuration and enabled features
///
/// Selects the appropriate telemetry adapter implementation based on
/// the configuration provider and available feature flags.
pub fn init(settings: &Settings) -> anyhow::Result<()> {
    match &settings.telemetry {
        #[cfg(feature = "telemetry-console")]
        TelemetryConfig::Console(config) => {
            use servicez_telemetry_console::{ConsoleLogger, LogFormat};

            let format = match config.log_format {
                crate::config::LogFormat::Json => LogFormat::Json,
                crate::config::LogFormat::Pretty => LogFormat::Pretty,
            };

            let logger = ConsoleLogger::new(format);
            logger.init()?;
        }
    }

    Ok(())
}
