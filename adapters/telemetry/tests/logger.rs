//! Integration tests for Console telemetry functionality

use servicez_telemetry_console::{ConsoleLogger, LogFormat};

#[test]
fn test_logger_creation_json() {
    let logger = ConsoleLogger::new(LogFormat::Json);
    // Verify logger can be created
    let _ = format!("{:?}", logger);
}

#[test]
fn test_logger_creation_pretty() {
    let logger = ConsoleLogger::new(LogFormat::Pretty);
    // Verify logger can be created
    let _ = format!("{:?}", logger);
}

#[test]
#[ignore] // Can only initialize tracing once per process
fn test_logger_init_json_success() {
    let logger = ConsoleLogger::new(LogFormat::Json);
    let result = logger.init();
    assert!(result.is_ok());
}

#[test]
#[ignore] // Can only initialize tracing once per process
fn test_logger_init_pretty_success() {
    let logger = ConsoleLogger::new(LogFormat::Pretty);
    let result = logger.init();
    assert!(result.is_ok());
}

/// Test that logger can be cloned (LogFormat is Clone)
#[test]
fn test_log_format_clone() {
    let format1 = LogFormat::Json;
    let format2 = format1.clone();
    let _ = format!("{:?}", format2);
}

/// Test that multiple logger instances can be created (before initialization)
#[test]
fn test_multiple_logger_instances() {
    let _logger1 = ConsoleLogger::new(LogFormat::Json);
    let _logger2 = ConsoleLogger::new(LogFormat::Pretty);
    // Verify both can be created without panicking
}

/// Test that LogFormat enum variants work correctly
#[test]
fn test_log_format_variants() {
    let json = LogFormat::Json;
    let pretty = LogFormat::Pretty;

    // Verify both variants can be created and cloned
    let _json_clone = json.clone();
    let _pretty_clone = pretty.clone();
}
