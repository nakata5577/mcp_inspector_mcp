use crate::models::{LogEntry, LogLevel, SamplingLogEntry};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fmt::Debug;

/// Backend trait for logging sampling requests and responses
///
/// This trait abstracts the storage mechanism for sampling logs,
/// allowing different implementations (memory-based, persistent, etc.)
pub trait LoggerBackend: Send + Sync + Debug {
    /// Adds a new log entry
    ///
    /// # Arguments
    /// * `entry` - The sampling log entry to add
    ///
    /// # Errors
    /// Returns an error if the log cannot be added
    fn add_log(&self, entry: SamplingLogEntry) -> Result<()>;

    /// Retrieves logs filtered by server name, status, and limited by count
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to filter logs by
    /// * `limit` - Maximum number of logs to return
    /// * `status` - Status filter: "all", "success", or "failed"
    ///
    /// # Returns
    /// A vector of matching log entries, sorted newest first
    fn get_logs(
        &self,
        server_name: &str,
        limit: usize,
        status: &str,
    ) -> Result<Vec<SamplingLogEntry>>;

    /// Counts the total number of logs for a specific server
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to count logs for
    ///
    /// # Returns
    /// The total count of logs for the specified server
    fn count_logs(&self, server_name: &str) -> Result<usize>;

    /// Clears all logs for a specific server (optional, for maintenance)
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to clear logs for
    fn clear_logs(&self, server_name: &str) -> Result<()> {
        // Default implementation (no-op for backward compatibility)
        let _ = server_name;
        Ok(())
    }

    // === Logging Message Methods ===

    /// Adds a new logging message entry
    ///
    /// # Arguments
    /// * `entry` - The log message entry to add
    ///
    /// # Errors
    /// Returns an error if the log message cannot be added
    fn add_log_message(&self, entry: LogEntry) -> Result<()>;

    /// Retrieves logging messages filtered by server, level, and time
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to filter logs by
    /// * `level` - Minimum log level filter (None = all levels)
    /// * `limit` - Maximum number of messages to return
    /// * `since` - Return only messages after this timestamp (None = all time)
    ///
    /// # Returns
    /// A vector of matching log entries, sorted newest first
    fn get_log_messages(
        &self,
        server_name: &str,
        level: Option<LogLevel>,
        limit: usize,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<LogEntry>>;

    /// Clears all logging messages for a specific server
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to clear messages for
    fn clear_log_messages(&self, server_name: &str) -> Result<()> {
        // Default implementation (no-op for backward compatibility)
        let _ = server_name;
        Ok(())
    }
}
