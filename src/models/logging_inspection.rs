use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request for retrieving logging messages from a server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingMessagesRequest {
    /// Target server name
    pub server: String,
    /// Minimum log level filter (e.g., "debug", "info", "warning", "error")
    pub level: Option<String>,
    /// Maximum number of messages to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Return only messages after this timestamp (RFC3339 format)
    pub since: Option<String>,
}

fn default_limit() -> usize {
    100
}

/// Response containing logging messages from a server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggingMessagesResponse {
    /// Server name these messages came from
    pub server_name: String,
    /// List of log entries
    pub messages: Vec<LogEntry>,
    /// Total number of messages returned
    pub total_count: usize,
}

/// A single log entry from an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    /// When this log message was received (RFC3339 format)
    pub timestamp: String,
    /// Server that emitted this log
    pub server_name: String,
    /// Log severity level
    pub level: LogLevel,
    /// Optional logger name/component
    pub logger: Option<String>,
    /// Log message content
    pub message: String,
}

/// Log severity level matching MCP protocol LoggingLevel
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

impl From<rmcp::model::LoggingLevel> for LogLevel {
    fn from(level: rmcp::model::LoggingLevel) -> Self {
        match level {
            rmcp::model::LoggingLevel::Debug => LogLevel::Debug,
            rmcp::model::LoggingLevel::Info => LogLevel::Info,
            rmcp::model::LoggingLevel::Notice => LogLevel::Notice,
            rmcp::model::LoggingLevel::Warning => LogLevel::Warning,
            rmcp::model::LoggingLevel::Error => LogLevel::Error,
            rmcp::model::LoggingLevel::Critical => LogLevel::Critical,
            rmcp::model::LoggingLevel::Alert => LogLevel::Alert,
            rmcp::model::LoggingLevel::Emergency => LogLevel::Emergency,
        }
    }
}

impl LogLevel {
    /// Parse a log level from a string
    pub fn parse_level(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "notice" => Some(LogLevel::Notice),
            "warning" | "warn" => Some(LogLevel::Warning),
            "error" => Some(LogLevel::Error),
            "critical" => Some(LogLevel::Critical),
            "alert" => Some(LogLevel::Alert),
            "emergency" => Some(LogLevel::Emergency),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Notice => "notice",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
            LogLevel::Alert => "alert",
            LogLevel::Emergency => "emergency",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
