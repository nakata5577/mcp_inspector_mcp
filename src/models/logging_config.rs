use serde::{Deserialize, Serialize};

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
}
