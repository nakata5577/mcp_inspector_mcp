use thiserror::Error;

#[derive(Error, Debug)]
pub enum InspectorError {
    #[error("Server '{0}' not found in configuration")]
    ServerNotFound(String),

    #[error("Failed to connect to server '{server}': {source}")]
    ConnectionFailed {
        server: String,
        source: anyhow::Error,
    },

    #[error("Tool execution failed for '{tool}' on server '{server}': {source}")]
    ToolExecutionFailed {
        server: String,
        tool: String,
        source: anyhow::Error,
    },

    #[error("Invalid arguments for tool '{tool}': {message}")]
    InvalidArguments { tool: String, message: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, InspectorError>;
