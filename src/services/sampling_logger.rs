use crate::models::SamplingLogEntry;
use crate::services::LoggerBackend;
use std::sync::Arc;

/// Thread-safe logger wrapper for sampling requests and responses
///
/// This is a facade that delegates to a LoggerBackend implementation.
#[derive(Debug, Clone)]
pub struct SamplingLogger {
    backend: Arc<dyn LoggerBackend>,
}

impl SamplingLogger {
    /// Creates a new SamplingLogger with the specified backend
    ///
    /// # Arguments
    /// * `backend` - The logger backend implementation to use
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_inspector_mcp::services::{SamplingLogger, MemoryLogger};
    /// use std::sync::Arc;
    ///
    /// let backend = Arc::new(MemoryLogger::new(1000));
    /// let logger = SamplingLogger::new(backend);
    /// ```
    pub fn new(backend: Arc<dyn LoggerBackend>) -> Self {
        Self { backend }
    }

    /// Adds a new log entry to the logger
    ///
    /// # Arguments
    ///
    /// * `entry` - The sampling log entry to add
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_inspector_mcp::services::{SamplingLogger, MemoryLogger};
    /// use mcp_inspector_mcp::models::{SamplingLogEntry, SamplingStatus};
    /// use std::sync::Arc;
    ///
    /// let backend = Arc::new(MemoryLogger::new(100));
    /// let logger = SamplingLogger::new(backend);
    /// let entry = SamplingLogEntry {
    ///     id: "test-id".to_string(),
    ///     timestamp: "2025-01-15T12:00:00Z".to_string(),
    ///     status: SamplingStatus::Success,
    ///     messages: vec![],
    ///     model_preferences: None,
    ///     system_prompt: None,
    ///     max_tokens: None,
    ///     error: None,
    ///     response: None,
    /// };
    /// logger.add_log(entry);
    /// ```
    pub fn add_log(&self, entry: SamplingLogEntry) {
        // For backward compatibility, ignore errors
        let _ = self.backend.add_log(entry);
    }

    /// Retrieves logs filtered by server name, status, and limited by count
    ///
    /// Results are returned in reverse chronological order (newest first).
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the server to filter logs by
    /// * `limit` - Maximum number of logs to return
    /// * `status` - Status filter: "all", "success", or "failed"
    ///
    /// # Returns
    ///
    /// A vector of matching log entries, sorted newest first
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_inspector_mcp::services::{SamplingLogger, MemoryLogger};
    /// use std::sync::Arc;
    ///
    /// let backend = Arc::new(MemoryLogger::new(100));
    /// let logger = SamplingLogger::new(backend);
    /// let logs = logger.get_logs("my-server", 10, "all");
    /// ```
    pub fn get_logs(&self, server_name: &str, limit: usize, status: &str) -> Vec<SamplingLogEntry> {
        // For backward compatibility, return empty vec on error
        self.backend
            .get_logs(server_name, limit, status)
            .unwrap_or_default()
    }

    /// Counts the total number of logs for a specific server
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the server to count logs for
    ///
    /// # Returns
    ///
    /// The total count of logs for the specified server
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_inspector_mcp::services::{SamplingLogger, MemoryLogger};
    /// use std::sync::Arc;
    ///
    /// let backend = Arc::new(MemoryLogger::new(100));
    /// let logger = SamplingLogger::new(backend);
    /// let count = logger.count_logs("my-server");
    /// ```
    pub fn count_logs(&self, server_name: &str) -> usize {
        // For backward compatibility, return 0 on error
        self.backend.count_logs(server_name).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SamplingContent, SamplingMessage, SamplingStatus};
    use crate::services::MemoryLogger;

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
    fn test_sampling_logger_wrapper() {
        let backend = Arc::new(MemoryLogger::new(100));
        let logger = SamplingLogger::new(backend);

        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ));

        let logs = logger.get_logs("server1", 10, "all");
        assert_eq!(logs.len(), 1);
        assert_eq!(logger.count_logs("server1"), 1);
    }

    #[test]
    fn test_add_and_get_logs() {
        let backend = Arc::new(MemoryLogger::new(100));
        let logger = SamplingLogger::new(backend);

        // Add logs for different servers
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ));
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:01:00Z",
            SamplingStatus::Failed,
        ));
        logger.add_log(create_test_entry(
            "server2",
            "2025-01-15T12:02:00Z",
            SamplingStatus::Success,
        ));

        // Get all logs for server1
        let server1_logs = logger.get_logs("server1", 10, "all");
        assert_eq!(server1_logs.len(), 2);
        assert_eq!(server1_logs[0].timestamp, "2025-01-15T12:01:00Z"); // Newest first

        // Get only success logs for server1
        let success_logs = logger.get_logs("server1", 10, "success");
        assert_eq!(success_logs.len(), 1);
        assert_eq!(success_logs[0].status, SamplingStatus::Success);

        // Get only failed logs for server1
        let failed_logs = logger.get_logs("server1", 10, "failed");
        assert_eq!(failed_logs.len(), 1);
        assert_eq!(failed_logs[0].status, SamplingStatus::Failed);

        // Test count
        assert_eq!(logger.count_logs("server1"), 2);
        assert_eq!(logger.count_logs("server2"), 1);
    }

    #[test]
    fn test_log_rotation() {
        let backend = Arc::new(MemoryLogger::new(3));
        let logger = SamplingLogger::new(backend);

        // Add 5 logs (exceeds limit)
        for i in 0..5 {
            logger.add_log(create_test_entry(
                "server1",
                &format!("2025-01-15T12:0{}:00Z", i),
                SamplingStatus::Success,
            ));
        }

        // Should only have the last 3 logs
        let logs = logger.get_logs("server1", 10, "all");
        assert_eq!(logs.len(), 3);

        // Verify oldest logs were removed (should have timestamps 2, 3, 4)
        let timestamps: Vec<String> = logs.iter().map(|l| l.timestamp.clone()).collect();
        assert!(timestamps.contains(&"2025-01-15T12:02:00Z".to_string()));
        assert!(timestamps.contains(&"2025-01-15T12:03:00Z".to_string()));
        assert!(timestamps.contains(&"2025-01-15T12:04:00Z".to_string()));

        // The first two should not exist
        assert!(!timestamps.contains(&"2025-01-15T12:00:00Z".to_string()));
        assert!(!timestamps.contains(&"2025-01-15T12:01:00Z".to_string()));
    }

    #[test]
    fn test_status_filter() {
        let backend = Arc::new(MemoryLogger::new(100));
        let logger = SamplingLogger::new(backend);

        // Add logs with different statuses
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:00:00Z",
            SamplingStatus::Success,
        ));
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:01:00Z",
            SamplingStatus::Failed,
        ));
        logger.add_log(create_test_entry(
            "server1",
            "2025-01-15T12:02:00Z",
            SamplingStatus::Success,
        ));

        // Test "all" filter
        let all_logs = logger.get_logs("server1", 10, "all");
        assert_eq!(all_logs.len(), 3);

        // Test "success" filter
        let success_logs = logger.get_logs("server1", 10, "success");
        assert_eq!(success_logs.len(), 2);
        assert!(success_logs
            .iter()
            .all(|l| l.status == SamplingStatus::Success));

        // Test "failed" filter
        let failed_logs = logger.get_logs("server1", 10, "failed");
        assert_eq!(failed_logs.len(), 1);
        assert!(failed_logs
            .iter()
            .all(|l| l.status == SamplingStatus::Failed));
    }

    #[test]
    fn test_limit_parameter() {
        let backend = Arc::new(MemoryLogger::new(100));
        let logger = SamplingLogger::new(backend);

        // Add 5 logs
        for i in 0..5 {
            logger.add_log(create_test_entry(
                "server1",
                &format!("2025-01-15T12:0{}:00Z", i),
                SamplingStatus::Success,
            ));
        }

        // Request only 2 logs
        let logs = logger.get_logs("server1", 2, "all");
        assert_eq!(logs.len(), 2);

        // Should get the newest 2
        assert_eq!(logs[0].timestamp, "2025-01-15T12:04:00Z");
        assert_eq!(logs[1].timestamp, "2025-01-15T12:03:00Z");
    }
}
