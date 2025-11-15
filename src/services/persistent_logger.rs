use crate::models::{SamplingLogEntry, SamplingStatus};
use crate::services::LoggerBackend;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{error, warn};

/// Persistent logger implementation using sled embedded database
///
/// Stores logs on disk, surviving server restarts. Provides automatic
/// log rotation based on maximum log count per server.
#[derive(Debug, Clone)]
pub struct PersistentLogger {
    db: Arc<sled::Db>,
    max_logs: usize,
}

impl PersistentLogger {
    /// Creates a new PersistentLogger with the specified database path
    ///
    /// # Arguments
    /// * `db_path` - Path to the sled database directory
    /// * `max_logs` - Maximum number of logs per server (for rotation)
    ///
    /// # Errors
    /// Returns an error if the database cannot be opened. On database corruption,
    /// attempts automatic recovery by recreating the database.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use mcp_inspector_mcp::services::PersistentLogger;
    ///
    /// let logger = PersistentLogger::new("./data/logs.db", 1000)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(db_path: &str, max_logs: usize) -> Result<Self> {
        let db = match sled::open(db_path) {
            Ok(db) => db,
            Err(e) => {
                error!("Failed to open database at {}: {:?}", db_path, e);
                warn!("Attempting to recover by creating new database...");

                // Try to remove corrupted database
                let _ = std::fs::remove_dir_all(db_path);

                // Create new database
                sled::open(db_path)
                    .with_context(|| format!("Failed to create new database at {}", db_path))?
            }
        };

        Ok(Self {
            db: Arc::new(db),
            max_logs,
        })
    }

    /// Generates a storage key for a log entry
    ///
    /// Format: {server_name}:{timestamp}
    /// The timestamp ensures uniqueness and enables time-based sorting.
    fn make_key(entry: &SamplingLogEntry) -> String {
        // Extract server name from entry.id (format: "server:timestamp")
        let server = entry.id.split(':').next().unwrap_or("unknown");
        format!("{}:{}", server, entry.timestamp)
    }

    /// Performs log rotation for a specific server
    ///
    /// Removes oldest logs if count exceeds max_logs. This ensures
    /// disk space usage remains bounded.
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to rotate logs for
    ///
    /// # Errors
    /// Returns an error if database operations fail
    fn rotate_logs(&self, server_name: &str) -> Result<()> {
        let count = self.count_logs(server_name)?;

        if count <= self.max_logs {
            return Ok(());
        }

        let prefix = format!("{}:", server_name);

        // Collect all entries with timestamps
        let mut entries: Vec<(Vec<u8>, String)> = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item.context("Failed to read database entry")?;

            let entry: SamplingLogEntry = serde_json::from_slice(&value)
                .context("Failed to deserialize log entry during rotation")?;

            entries.push((key.to_vec(), entry.timestamp));
        }

        // Sort by timestamp (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1));

        // Delete oldest entries
        let to_delete = count - self.max_logs;
        for (key, _) in entries.iter().take(to_delete) {
            self.db
                .remove(key)
                .context("Failed to remove old log entry")?;
        }

        self.db
            .flush()
            .context("Failed to flush database after rotation")?;

        Ok(())
    }
}

impl LoggerBackend for PersistentLogger {
    fn add_log(&self, entry: SamplingLogEntry) -> Result<()> {
        let server = entry.id.split(':').next().unwrap_or("unknown");

        // Serialize entry using JSON (better compatibility with serde attributes)
        let value = serde_json::to_vec(&entry).context("Failed to serialize log entry")?;

        // Generate key
        let key = Self::make_key(&entry);

        // Store in database
        if let Err(e) = self.db.insert(key.as_bytes(), value) {
            error!("Failed to insert log into database: {:?}", e);
            return Err(e.into());
        }

        // Perform rotation (non-fatal on failure)
        if let Err(e) = self.rotate_logs(server) {
            warn!("Failed to rotate logs for server {}: {:?}", server, e);
            // Continue even if rotation fails
        }

        // Flush to disk (optional, for durability)
        if let Err(e) = self.db.flush() {
            warn!("Failed to flush database: {:?}", e);
            // Continue even if flush fails
        }

        Ok(())
    }

    fn get_logs(
        &self,
        server_name: &str,
        limit: usize,
        status: &str,
    ) -> Result<Vec<SamplingLogEntry>> {
        let prefix = format!("{}:", server_name);

        // Scan entries with the server prefix
        let mut entries: Vec<SamplingLogEntry> = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = item.context("Failed to read database entry")?;

            // Deserialize entry
            let entry: SamplingLogEntry =
                serde_json::from_slice(&value).context("Failed to deserialize log entry")?;

            // Apply status filter
            let matches_status = match status {
                "all" => true,
                "success" => entry.status == SamplingStatus::Success,
                "failed" => entry.status == SamplingStatus::Failed,
                _ => true,
            };

            if matches_status {
                entries.push(entry);
            }
        }

        // Sort by timestamp (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        Ok(entries.into_iter().take(limit).collect())
    }

    fn count_logs(&self, server_name: &str) -> Result<usize> {
        let prefix = format!("{}:", server_name);

        let count = self.db.scan_prefix(prefix.as_bytes()).count();

        Ok(count)
    }

    fn clear_logs(&self, server_name: &str) -> Result<()> {
        let prefix = format!("{}:", server_name);

        // Collect keys to delete
        let keys_to_delete: Vec<Vec<u8>> = self
            .db
            .scan_prefix(prefix.as_bytes())
            .filter_map(|item| item.ok().map(|(k, _)| k.to_vec()))
            .collect();

        // Delete all matching keys
        for key in keys_to_delete {
            if let Err(e) = self.db.remove(key) {
                error!("Failed to remove log entry: {:?}", e);
                return Err(e.into());
            }
        }

        self.db
            .flush()
            .context("Failed to flush database after clearing logs")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SamplingContent, SamplingMessage};
    use tempfile::TempDir;

    /// Helper function to create a test log entry
    fn create_test_entry(
        server: &str,
        timestamp: &str,
        status: SamplingStatus,
    ) -> SamplingLogEntry {
        SamplingLogEntry {
            id: format!("{}:{}", server, timestamp),
            timestamp: timestamp.to_string(),
            status,
            messages: vec![SamplingMessage {
                role: "user".to_string(),
                content: SamplingContent {
                    content_type: "text".to_string(),
                    text: Some("test message".to_string()),
                },
            }],
            model_preferences: None,
            system_prompt: None,
            max_tokens: Some(100),
            error: None,
            response: Some("test response".to_string()),
        }
    }

    #[test]
    fn test_persistent_logger_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

        // Add a log
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ))?;

        // Retrieve logs
        let logs = logger.get_logs("server1", 10, "all")?;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].status, SamplingStatus::Success);

        // Count logs
        assert_eq!(logger.count_logs("server1")?, 1);

        Ok(())
    }

    #[test]
    fn test_persistence_across_instances() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");

        // First instance - add logs
        {
            let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;
            logger.add_log(create_test_entry(
                "server1",
                "2025-01-15T12:00:00Z",
                SamplingStatus::Success,
            ))?;
        }

        // Second instance - verify logs are still there
        {
            let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;
            let logs = logger.get_logs("server1", 10, "all")?;
            assert_eq!(logs.len(), 1);
        }

        Ok(())
    }

    #[test]
    fn test_log_rotation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 3)?;

        // Add 5 logs (exceeds max)
        for i in 0..5 {
            logger.add_log(create_test_entry(
                "server1",
                &format!("2025-01-15T12:0{}:00Z", i),
                SamplingStatus::Success,
            ))?;
        }

        // Should only have the last 3
        let logs = logger.get_logs("server1", 10, "all")?;
        assert_eq!(logs.len(), 3);

        // Verify correct logs remain (newest 3)
        let timestamps: Vec<String> = logs.iter().map(|l| l.timestamp.clone()).collect();
        assert!(timestamps.contains(&"2025-01-15T12:02:00Z".to_string()));
        assert!(timestamps.contains(&"2025-01-15T12:03:00Z".to_string()));
        assert!(timestamps.contains(&"2025-01-15T12:04:00Z".to_string()));

        // Oldest logs should be removed
        assert!(!timestamps.contains(&"2025-01-15T12:00:00Z".to_string()));
        assert!(!timestamps.contains(&"2025-01-15T12:01:00Z".to_string()));

        Ok(())
    }

    #[test]
    fn test_status_filter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ))?;
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:01:00Z",
            SamplingStatus::Failed,
        ))?;

        // Test filters
        assert_eq!(logger.get_logs("server1", 10, "all")?.len(), 2);
        assert_eq!(logger.get_logs("server1", 10, "success")?.len(), 1);
        assert_eq!(logger.get_logs("server1", 10, "failed")?.len(), 1);

        Ok(())
    }

    #[test]
    fn test_clear_logs() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ))?;

        assert_eq!(logger.count_logs("server1")?, 1);

        logger.clear_logs("server1")?;
        assert_eq!(logger.count_logs("server1")?, 0);

        Ok(())
    }

    #[test]
    fn test_multiple_servers() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

        // Add logs for different servers
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ))?;
        logger.add_log(create_test_entry(
            "server2",
            "2025-01-15T12:01:00Z",
            SamplingStatus::Success,
        ))?;
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:02:00Z",
            SamplingStatus::Failed,
        ))?;

        // Verify counts
        assert_eq!(logger.count_logs("server1")?, 2);
        assert_eq!(logger.count_logs("server2")?, 1);

        // Clear server1 logs
        logger.clear_logs("server1")?;

        // Verify server1 logs cleared but server2 remains
        assert_eq!(logger.count_logs("server1")?, 0);
        assert_eq!(logger.count_logs("server2")?, 1);

        Ok(())
    }

    #[test]
    fn test_limit_parameter() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_db");
        let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

        // Add 5 logs
        for i in 0..5 {
            logger.add_log(create_test_entry(
                "server1",
                &format!("2025-01-15T12:0{}:00Z", i),
                SamplingStatus::Success,
            ))?;
        }

        // Request only 2 logs
        let logs = logger.get_logs("server1", 2, "all")?;
        assert_eq!(logs.len(), 2);

        // Should get the newest 2
        assert_eq!(logs[0].timestamp, "2025-01-15T12:04:00Z");
        assert_eq!(logs[1].timestamp, "2025-01-15T12:03:00Z");

        Ok(())
    }
}
