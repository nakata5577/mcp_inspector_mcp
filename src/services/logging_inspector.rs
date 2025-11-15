use crate::models::{LogEntry, LogLevel, LoggingMessagesRequest, LoggingMessagesResponse};
use crate::services::LoggerBackend;
use anyhow::Result;
use std::sync::Arc;

/// Service for inspecting and retrieving logging messages from MCP servers
///
/// This service provides filtering and querying capabilities for log messages
/// received from MCP servers via the notifications/message protocol.
#[derive(Debug, Clone)]
pub struct LoggingInspector {
    logger_backend: Arc<dyn LoggerBackend>,
}

impl LoggingInspector {
    /// Creates a new LoggingInspector with the specified backend
    ///
    /// # Arguments
    /// * `logger_backend` - Backend for storing and retrieving log messages
    pub fn new(logger_backend: Arc<dyn LoggerBackend>) -> Self {
        Self { logger_backend }
    }

    /// Adds a log entry (called by MonitoringTransport when receiving notifications/message)
    ///
    /// # Arguments
    /// * `entry` - The log entry to store
    ///
    /// # Errors
    /// Returns an error if the backend fails to store the entry
    pub fn add_log_entry(&self, entry: LogEntry) -> Result<()> {
        self.logger_backend.add_log_message(entry)
    }

    /// Retrieves logging messages with filtering
    ///
    /// # Arguments
    /// * `request` - Request containing server name, filters, and limits
    ///
    /// # Returns
    /// A response containing filtered log messages
    ///
    /// # Errors
    /// Returns an error if:
    /// - The log level string is invalid
    /// - The timestamp string is malformed
    /// - The backend fails to retrieve messages
    pub fn get_logging_messages(
        &self,
        request: LoggingMessagesRequest,
    ) -> Result<LoggingMessagesResponse> {
        // Parse level filter
        let level_filter = if let Some(ref level_str) = request.level {
            LogLevel::parse_level(level_str)
        } else {
            None
        };

        // Parse time filter
        let since_filter = if let Some(ref since_str) = request.since {
            Some(
                chrono::DateTime::parse_from_rfc3339(since_str)
                    .map_err(|e| anyhow::anyhow!("Invalid timestamp format: {}", e))?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        // Query backend
        let messages = self.logger_backend.get_log_messages(
            &request.server,
            level_filter,
            request.limit,
            since_filter,
        )?;

        let total_count = messages.len();

        Ok(LoggingMessagesResponse {
            server_name: request.server,
            messages,
            total_count,
        })
    }

    /// Clears all logging messages for a specific server
    ///
    /// # Arguments
    /// * `server_name` - Name of the server to clear messages for
    pub fn clear_logging_messages(&self, server_name: &str) -> Result<()> {
        self.logger_backend.clear_log_messages(server_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::MemoryLogger;

    fn create_test_entry(
        server: &str,
        level: LogLevel,
        timestamp: &str,
        message: &str,
    ) -> LogEntry {
        LogEntry {
            timestamp: timestamp.to_string(),
            server_name: server.to_string(),
            level,
            logger: Some("test".to_string()),
            message: message.to_string(),
        }
    }

    #[test]
    fn test_add_and_retrieve_messages() -> Result<()> {
        let backend = Arc::new(MemoryLogger::new(100));
        let inspector = LoggingInspector::new(backend);

        // Add log messages
        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Info,
            "2025-01-15T12:00:00Z",
            "Info message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Error,
            "2025-01-15T12:01:00Z",
            "Error message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Debug,
            "2025-01-15T12:02:00Z",
            "Debug message",
        ))?;

        // Retrieve all messages
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: None,
            limit: 100,
            since: None,
        })?;

        assert_eq!(response.total_count, 3);
        assert_eq!(response.messages[0].level, LogLevel::Debug); // Newest first

        Ok(())
    }

    #[test]
    fn test_level_filter() -> Result<()> {
        let backend = Arc::new(MemoryLogger::new(100));
        let inspector = LoggingInspector::new(backend);

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Debug,
            "2025-01-15T12:00:00Z",
            "Debug message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Info,
            "2025-01-15T12:01:00Z",
            "Info message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Warning,
            "2025-01-15T12:02:00Z",
            "Warning message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Error,
            "2025-01-15T12:03:00Z",
            "Error message",
        ))?;

        // Filter for warning and above
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: Some("warning".to_string()),
            limit: 100,
            since: None,
        })?;

        assert_eq!(response.total_count, 2); // Warning and Error only
        assert!(response
            .messages
            .iter()
            .all(|m| m.level >= LogLevel::Warning));

        Ok(())
    }

    #[test]
    fn test_time_filter() -> Result<()> {
        let backend = Arc::new(MemoryLogger::new(100));
        let inspector = LoggingInspector::new(backend);

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Info,
            "2025-01-15T12:00:00Z",
            "Old message",
        ))?;

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Info,
            "2025-01-15T12:05:00Z",
            "New message",
        ))?;

        // Filter for messages after 12:02
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: None,
            limit: 100,
            since: Some("2025-01-15T12:02:00Z".to_string()),
        })?;

        assert_eq!(response.total_count, 1);
        assert_eq!(response.messages[0].message, "New message");

        Ok(())
    }

    #[test]
    fn test_limit() -> Result<()> {
        let backend = Arc::new(MemoryLogger::new(100));
        let inspector = LoggingInspector::new(backend);

        // Add 5 messages
        for i in 0..5 {
            inspector.add_log_entry(create_test_entry(
                "server1",
                LogLevel::Info,
                &format!("2025-01-15T12:0{}:00Z", i),
                &format!("Message {}", i),
            ))?;
        }

        // Limit to 2 messages
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: None,
            limit: 2,
            since: None,
        })?;

        assert_eq!(response.total_count, 2);

        Ok(())
    }

    #[test]
    fn test_clear_messages() -> Result<()> {
        let backend = Arc::new(MemoryLogger::new(100));
        let inspector = LoggingInspector::new(backend);

        inspector.add_log_entry(create_test_entry(
            "server1",
            LogLevel::Info,
            "2025-01-15T12:00:00Z",
            "Test message",
        ))?;

        // Verify message exists
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: None,
            limit: 100,
            since: None,
        })?;
        assert_eq!(response.total_count, 1);

        // Clear messages
        inspector.clear_logging_messages("server1")?;

        // Verify messages cleared
        let response = inspector.get_logging_messages(LoggingMessagesRequest {
            server: "server1".to_string(),
            level: None,
            limit: 100,
            since: None,
        })?;
        assert_eq!(response.total_count, 0);

        Ok(())
    }
}
