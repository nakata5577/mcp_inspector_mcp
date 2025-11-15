use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// Logging backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LoggingBackend {
    /// In-memory storage (data lost on restart)
    #[default]
    Memory,
    /// Disk-based storage using sled (data survives restart)
    Persistent,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Backend type: "memory" or "persistent"
    #[serde(default)]
    pub backend: LoggingBackend,

    /// Database path for persistent backend (required if backend = "persistent")
    pub db_path: Option<String>,

    /// Maximum number of logs per server (default: 10000)
    #[serde(default = "default_max_logs")]
    pub max_logs: usize,
}

fn default_max_logs() -> usize {
    10000
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            backend: LoggingBackend::Memory,
            db_path: None,
            max_logs: default_max_logs(),
        }
    }
}

impl LoggingConfig {
    /// Validates the logging configuration
    ///
    /// # Errors
    /// Returns an error if:
    /// - backend is "persistent" but db_path is not provided
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.backend == LoggingBackend::Persistent && self.db_path.is_none() {
            anyhow::bail!("db_path is required when backend = \"persistent\"");
        }
        Ok(())
    }

    /// Load logging configuration from environment variables
    ///
    /// # Environment Variables
    /// - `MCP_LOGGING_BACKEND`: Backend type ("memory" or "persistent", default: "memory")
    /// - `MCP_LOGGING_DB_PATH`: Database path (required if backend = "persistent")
    /// - `MCP_LOGGING_MAX_LOGS`: Maximum number of logs per server (default: 10000)
    ///
    /// # Errors
    /// Returns an error if:
    /// - `MCP_LOGGING_MAX_LOGS` is not a valid number
    /// - backend is "persistent" but `MCP_LOGGING_DB_PATH` is not set
    pub fn from_env() -> Result<Self> {
        // Read backend type (default: "memory")
        let backend_str = env::var("MCP_LOGGING_BACKEND").unwrap_or_else(|_| "memory".to_string());
        let backend = match backend_str.to_lowercase().as_str() {
            "memory" => LoggingBackend::Memory,
            "persistent" => LoggingBackend::Persistent,
            other => {
                anyhow::bail!(
                    "Invalid MCP_LOGGING_BACKEND value: '{}'. Must be 'memory' or 'persistent'",
                    other
                );
            }
        };

        // Read database path (required if persistent backend)
        let db_path = env::var("MCP_LOGGING_DB_PATH").ok();

        // Validate persistent backend has db_path
        if backend == LoggingBackend::Persistent && db_path.is_none() {
            anyhow::bail!(
                "MCP_LOGGING_DB_PATH environment variable is required when MCP_LOGGING_BACKEND is 'persistent'"
            );
        }

        // Read max_logs (default: 10000)
        let max_logs = if let Ok(max_logs_str) = env::var("MCP_LOGGING_MAX_LOGS") {
            max_logs_str.parse::<usize>().with_context(|| {
                format!(
                    "Failed to parse MCP_LOGGING_MAX_LOGS as a number: '{}'",
                    max_logs_str
                )
            })?
        } else {
            default_max_logs()
        };

        Ok(Self {
            backend,
            db_path,
            max_logs,
        })
    }
}
