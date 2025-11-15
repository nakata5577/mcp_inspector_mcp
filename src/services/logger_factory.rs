use crate::models::{LoggingBackend, LoggingConfig};
use crate::services::{LoggerBackend, MemoryLogger, PersistentLogger};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;

/// Creates a logger backend based on the configuration
///
/// # Arguments
/// * `config` - Logging configuration
///
/// # Returns
/// Arc-wrapped logger backend implementation
///
/// # Errors
/// Returns an error if:
/// - Configuration is invalid
/// - Database cannot be opened (for persistent backend)
pub fn create_logger(config: &LoggingConfig) -> Result<Arc<dyn LoggerBackend>> {
    // Validate configuration
    config.validate().context("Invalid logging configuration")?;

    match config.backend {
        LoggingBackend::Memory => {
            info!(
                "Creating memory-based logger (max_logs: {})",
                config.max_logs
            );
            Ok(Arc::new(MemoryLogger::new(config.max_logs)))
        }
        LoggingBackend::Persistent => {
            let db_path = config
                .db_path
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("db_path required for persistent backend"))?;

            info!(
                "Creating persistent logger (db_path: {}, max_logs: {})",
                db_path, config.max_logs
            );

            let logger = PersistentLogger::new(db_path, config.max_logs)
                .with_context(|| format!("Failed to create persistent logger at {}", db_path))?;

            Ok(Arc::new(logger))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_memory_logger() {
        let config = LoggingConfig {
            backend: LoggingBackend::Memory,
            db_path: None,
            max_logs: 100,
        };

        let logger = create_logger(&config).expect("Failed to create memory logger");

        // Verify it's working
        use crate::models::{SamplingLogEntry, SamplingStatus};
        let entry = SamplingLogEntry {
            id: "server1:2025-01-15T12:00:00Z".to_string(),
            timestamp: "2025-01-15T12:00:00Z".to_string(),
            status: SamplingStatus::Success,
            messages: vec![],
            model_preferences: None,
            system_prompt: None,
            max_tokens: None,
            error: None,
            response: None,
        };

        logger.add_log(entry).expect("Failed to add log");
        assert_eq!(logger.count_logs("server1").unwrap(), 1);
    }

    #[test]
    fn test_create_persistent_logger() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db");

        let config = LoggingConfig {
            backend: LoggingBackend::Persistent,
            db_path: Some(db_path.to_str().unwrap().to_string()),
            max_logs: 100,
        };

        let logger = create_logger(&config).expect("Failed to create persistent logger");

        // Verify it's working
        use crate::models::{SamplingLogEntry, SamplingStatus};
        let entry = SamplingLogEntry {
            id: "server1:2025-01-15T12:00:00Z".to_string(),
            timestamp: "2025-01-15T12:00:00Z".to_string(),
            status: SamplingStatus::Success,
            messages: vec![],
            model_preferences: None,
            system_prompt: None,
            max_tokens: None,
            error: None,
            response: None,
        };

        logger.add_log(entry).expect("Failed to add log");
        assert_eq!(logger.count_logs("server1").unwrap(), 1);
    }

    #[test]
    fn test_invalid_config() {
        // Persistent backend without db_path
        let config = LoggingConfig {
            backend: LoggingBackend::Persistent,
            db_path: None,
            max_logs: 100,
        };

        assert!(create_logger(&config).is_err());
    }
}
