use crate::models::LoggingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transport type for MCP server connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    Stdio,
}

/// Connection parameters for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionParams {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub transport: TransportType,
    #[serde(flatten)]
    pub params: ConnectionParams,
}

/// Configuration file structure
#[derive(Debug, Deserialize)]
pub struct InspectorConfig {
    pub servers: Vec<ServerConfig>,

    /// Logging configuration (optional)
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl ServerConfig {
    /// Create a new ServerConfig with stdio transport
    pub fn new_stdio(
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> Self {
        Self {
            name: name.into(),
            transport: TransportType::Stdio,
            params: ConnectionParams {
                command: command.into(),
                args,
                env,
            },
        }
    }
}
