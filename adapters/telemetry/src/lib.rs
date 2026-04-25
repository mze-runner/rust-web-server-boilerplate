use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Console-based telemetry initializer
///
/// Outputs structured logs to stdout in JSON or pretty-printed format
#[derive(Debug)]
pub struct ConsoleLogger {
    format: LogFormat,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Pretty,
}

impl ConsoleLogger {
    pub fn new(format: LogFormat) -> Self {
        Self { format }
    }

    /// Initialize tracing subscriber with console output
    pub fn init(&self) -> Result<(), anyhow::Error> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        match self.format {
            LogFormat::Json => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .json()
                            .with_target(true)
                            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                            .with_current_span(true)
                            .flatten_event(true),
                    )
                    .init();
            }
            LogFormat::Pretty => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .pretty()
                            .compact()
                            .with_target(false)
                            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
                    )
                    .init();
            }
        }

        tracing::info!(format = ?self.format, "Console telemetry initialized");
        Ok(())
    }
}
