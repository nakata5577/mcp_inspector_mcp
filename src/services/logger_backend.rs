use crate::models::SamplingLogEntry;
use anyhow::Result;
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
}
