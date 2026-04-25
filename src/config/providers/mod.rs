// Conditional module declarations based on features
#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "scylla")]
pub mod scylla;

// Re-export provider configs with feature guards
#[cfg(feature = "postgres")]
pub use postgres::PostgresConfig;

#[cfg(feature = "scylla")]
pub use scylla::ScyllaConfig;

pub mod telemetry; // NEW
pub use telemetry::{LogFormat, TelemetryConfig, TelemetryProvider};
