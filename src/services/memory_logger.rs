use crate::models::{LogEntry, LogLevel, SamplingLogEntry, SamplingStatus};
use crate::services::LoggerBackend;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Memory-based logger implementation
///
/// Stores logs in memory with a fixed maximum capacity.
/// Logs are lost on server restart.
#[derive(Debug)]
pub struct MemoryLogger {
    logs: Arc<RwLock<Vec<SamplingLogEntry>>>,
    log_messages: Arc<RwLock<HashMap<String, Vec<LogEntry>>>>,
    max_logs: usize,
}

impl Clone for MemoryLogger {
    fn clone(&self) -> Self {
        Self {
            logs: Arc::clone(&self.logs),
            log_messages: Arc::clone(&self.log_messages),
            max_logs: self.max_logs,
        }
    }
}

impl MemoryLogger {
    /// Creates a new MemoryLogger with specified maximum capacity
    ///
    /// # Arguments
    ///
    /// * `max_logs` - Maximum number of log entries to retain before rotation
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_inspector_mcp::services::MemoryLogger;
    ///
    /// let logger = MemoryLogger::new(1000);
    /// ```
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            log_messages: Arc::new(RwLock::new(HashMap::new())),
            max_logs,
        }
    }
}

impl LoggerBackend for MemoryLogger {
    fn add_log(&self, entry: SamplingLogEntry) -> Result<()> {
        let mut logs = self.logs.write().unwrap();
        logs.push(entry);

        // FIFO rotation
        if logs.len() > self.max_logs {
            logs.remove(0);
        }

        Ok(())
    }

    fn get_logs(
        &self,
        server_name: &str,
        limit: usize,
        status: &str,
    ) -> Result<Vec<SamplingLogEntry>> {
        let logs = self.logs.read().unwrap();

        let mut filtered: Vec<SamplingLogEntry> = logs
            .iter()
            .filter(|entry| {
                let entry_server = entry.id.split(':').next().unwrap_or("");
                entry_server == server_name
            })
            .filter(|entry| match status {
                "all" => true,
                "success" => entry.status == SamplingStatus::Success,
                "failed" => entry.status == SamplingStatus::Failed,
                _ => true,
            })
            .cloned()
            .collect();

        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(filtered.into_iter().take(limit).collect())
    }

    fn count_logs(&self, server_name: &str) -> Result<usize> {
        let logs = self.logs.read().unwrap();
        let count = logs
            .iter()
            .filter(|entry| {
                let entry_server = entry.id.split(':').next().unwrap_or("");
                entry_server == server_name
            })
            .count();
        Ok(count)
    }

    fn clear_logs(&self, server_name: &str) -> Result<()> {
        let mut logs = self.logs.write().unwrap();
        logs.retain(|entry| {
            let entry_server = entry.id.split(':').next().unwrap_or("");
            entry_server != server_name
        });
        Ok(())
    }

    fn add_log_message(&self, entry: LogEntry) -> Result<()> {
        let mut messages = self.log_messages.write().unwrap();

        let server_messages = messages
            .entry(entry.server_name.clone())
            .or_default();

        server_messages.push(entry);

        // FIFO rotation per server
        while server_messages.len() > self.max_logs {
            server_messages.remove(0);
        }

        Ok(())
    }

    fn get_log_messages(
        &self,
        server_name: &str,
        level: Option<LogLevel>,
        limit: usize,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<LogEntry>> {
        let messages = self.log_messages.read().unwrap();

        let server_messages = messages.get(server_name).cloned().unwrap_or_default();

        let mut filtered: Vec<LogEntry> = server_messages
            .into_iter()
            .filter(|entry| {
                // Apply level filter (minimum level)
                if let Some(min_level) = level {
                    if entry.level < min_level {
                        return false;
                    }
                }

                // Apply time filter
                if let Some(since_time) = since {
                    if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                        if entry_time.with_timezone(&Utc) < since_time {
                            return false;
                        }
                    } else {
                        // Skip entries with invalid timestamps
                        return false;
                    }
                }

                true
            })
            .collect();

        // Sort by timestamp (newest first)
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply limit
        Ok(filtered.into_iter().take(limit).collect())
    }

    fn clear_log_messages(&self, server_name: &str) -> Result<()> {
        let mut messages = self.log_messages.write().unwrap();
        messages.remove(server_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SamplingContent, SamplingMessage};

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
    fn test_add_and_get_logs() {
        let logger = MemoryLogger::new(100);

        // Add logs for different servers
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:00:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:01:00Z",
                SamplingStatus::Failed,
            ))
            .unwrap();
        logger
            .add_log(create_test_entry(
                "server2",
                "2025-01-15T12:02:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();

        // Get all logs for server1
        let server1_logs = logger.get_logs("server1", 10, "all").unwrap();
        assert_eq!(server1_logs.len(), 2);
        assert_eq!(server1_logs[0].timestamp, "2025-01-15T12:01:00Z"); // Newest first

        // Get only success logs for server1
        let success_logs = logger.get_logs("server1", 10, "success").unwrap();
        assert_eq!(success_logs.len(), 1);
        assert_eq!(success_logs[0].status, SamplingStatus::Success);

        // Get only failed logs for server1
        let failed_logs = logger.get_logs("server1", 10, "failed").unwrap();
        assert_eq!(failed_logs.len(), 1);
        assert_eq!(failed_logs[0].status, SamplingStatus::Failed);

        // Test count
        assert_eq!(logger.count_logs("server1").unwrap(), 2);
        assert_eq!(logger.count_logs("server2").unwrap(), 1);
    }

    #[test]
    fn test_log_rotation() {
        let logger = MemoryLogger::new(3); // Small limit for testing

        // Add 5 logs (exceeds limit)
        for i in 0..5 {
            logger
                .add_log(create_test_entry(
                    "server1",
                    &format!("2025-01-15T12:0{}:00Z", i),
                    SamplingStatus::Success,
                ))
                .unwrap();
        }

        // Should only have the last 3 logs
        let logs = logger.get_logs("server1", 10, "all").unwrap();
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
        let logger = MemoryLogger::new(100);

        // Add logs with different statuses
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:00:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:01:00Z",
                SamplingStatus::Failed,
            ))
            .unwrap();
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:02:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();

        // Test "all" filter
        let all_logs = logger.get_logs("server1", 10, "all").unwrap();
        assert_eq!(all_logs.len(), 3);

        // Test "success" filter
        let success_logs = logger.get_logs("server1", 10, "success").unwrap();
        assert_eq!(success_logs.len(), 2);
        assert!(success_logs
            .iter()
            .all(|l| l.status == SamplingStatus::Success));

        // Test "failed" filter
        let failed_logs = logger.get_logs("server1", 10, "failed").unwrap();
        assert_eq!(failed_logs.len(), 1);
        assert!(failed_logs
            .iter()
            .all(|l| l.status == SamplingStatus::Failed));
    }

    #[test]
    fn test_limit_parameter() {
        let logger = MemoryLogger::new(100);

        // Add 5 logs
        for i in 0..5 {
            logger
                .add_log(create_test_entry(
                    "server1",
                    &format!("2025-01-15T12:0{}:00Z", i),
                    SamplingStatus::Success,
                ))
                .unwrap();
        }

        // Request only 2 logs
        let logs = logger.get_logs("server1", 2, "all").unwrap();
        assert_eq!(logs.len(), 2);

        // Should get the newest 2
        assert_eq!(logs[0].timestamp, "2025-01-15T12:04:00Z");
        assert_eq!(logs[1].timestamp, "2025-01-15T12:03:00Z");
    }

    #[test]
    fn test_clear_logs() {
        let logger = MemoryLogger::new(100);

        // Add logs for different servers
        logger
            .add_log(create_test_entry(
                "server1",
                "2025-01-15T12:00:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();
        logger
            .add_log(create_test_entry(
                "server2",
                "2025-01-15T12:01:00Z",
                SamplingStatus::Success,
            ))
            .unwrap();

        assert_eq!(logger.count_logs("server1").unwrap(), 1);
        assert_eq!(logger.count_logs("server2").unwrap(), 1);

        // Clear server1 logs
        logger.clear_logs("server1").unwrap();

        assert_eq!(logger.count_logs("server1").unwrap(), 0);
        assert_eq!(logger.count_logs("server2").unwrap(), 1);
    }
}
