use crate::models::LoggingConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

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

impl InspectorConfig {
    /// Load configuration from environment variables
    ///
    /// # Environment Variables
    /// - `MCP_INSPECTOR_SERVERS`: JSON array of server configurations (required)
    /// - `MCP_LOGGING_BACKEND`: Logging backend type (optional, see `LoggingConfig::from_env`)
    /// - `MCP_LOGGING_DB_PATH`: Database path for persistent logging (conditional)
    /// - `MCP_LOGGING_MAX_LOGS`: Maximum number of logs per server (optional)
    ///
    /// # Example
    /// ```bash
    /// export MCP_INSPECTOR_SERVERS='[{"name":"test","transport":"stdio","command":"npx","args":["-y","@modelcontextprotocol/server-everything"],"env":{}}]'
    /// export MCP_LOGGING_BACKEND=memory
    /// ```
    ///
    /// # Errors
    /// Returns an error if:
    /// - `MCP_INSPECTOR_SERVERS` is not set
    /// - `MCP_INSPECTOR_SERVERS` is not valid JSON
    /// - `MCP_INSPECTOR_SERVERS` is an empty array
    /// - Any server configuration is invalid
    /// - Logging configuration is invalid
    pub fn from_env() -> Result<Self> {
        // Read and parse MCP_INSPECTOR_SERVERS
        let servers_json = env::var("MCP_INSPECTOR_SERVERS").context(
            "MCP_INSPECTOR_SERVERS environment variable not set. \
             Please set it to a JSON array of server configurations.",
        )?;

        let servers: Vec<ServerConfig> = serde_json::from_str(&servers_json).context(
            "Failed to parse MCP_INSPECTOR_SERVERS as JSON array. \
             Expected format: [{\"name\":\"...\",\"transport\":\"stdio\",\"command\":\"...\",\"args\":[...],\"env\":{...}}]"
        )?;

        // Validate non-empty
        if servers.is_empty() {
            anyhow::bail!(
                "MCP_INSPECTOR_SERVERS must contain at least one server configuration. \
                 Currently it is an empty array."
            );
        }

        // Validate each server has required fields
        for (index, server) in servers.iter().enumerate() {
            if server.name.is_empty() {
                anyhow::bail!(
                    "Server at index {} has an empty 'name' field. Each server must have a non-empty name.",
                    index
                );
            }
            if server.params.command.is_empty() {
                anyhow::bail!(
                    "Server '{}' (index {}) has an empty 'command' field. Each server must have a non-empty command.",
                    server.name,
                    index
                );
            }
        }

        // Load logging configuration
        let logging = LoggingConfig::from_env()?;

        Ok(InspectorConfig { servers, logging })
    }
}
