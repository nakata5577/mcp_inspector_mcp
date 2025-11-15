use crate::models::{InspectorError, LoggingConfig};
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

    /// Parse DSL format: name:transport:command[:arg1:arg2:...]
    ///
    /// # Format
    /// ```text
    /// name:transport:command[:arg1:arg2:...]
    /// ```
    ///
    /// # Windows Path Handling
    /// Windows paths with drive letters (e.g., `C:/path/to/file.exe`) are automatically detected
    /// and handled correctly. The drive letter and colon are treated as part of the command path.
    ///
    /// # Examples
    /// ```
    /// # use mcp_inspector_mcp::models::ServerConfig;
    /// let config = ServerConfig::from_dsl("fa:stdio:C:/path/to/fa.exe").unwrap();
    /// assert_eq!(config.name, "fa");
    /// assert_eq!(config.params.command, "C:/path/to/fa.exe");
    ///
    /// let config = ServerConfig::from_dsl("ta:stdio:/path/to/ta.exe:--verbose:--debug").unwrap();
    /// assert_eq!(config.params.args, vec!["--verbose", "--debug"]);
    /// ```
    ///
    /// # Errors
    /// Returns `InspectorError::Config` if:
    /// - The DSL string has fewer than 3 parts (name, transport, command are required)
    /// - The transport type is not "stdio"
    pub fn from_dsl(dsl: &str) -> std::result::Result<Self, InspectorError> {
        let parts: Vec<&str> = dsl.split(':').collect();

        if parts.len() < 3 {
            return Err(InspectorError::Config(format!(
                "Invalid DSL format (expected name:transport:command[:args...]): {}",
                dsl
            )));
        }

        let name = parts[0].to_string();
        let transport_str = parts[1].to_lowercase();

        // Handle Windows paths (e.g., C:/path/to/file.exe)
        // Check if parts[2] is a single letter (Windows drive letter)
        let (command, args_start_idx) = if parts.len() > 3
            && parts[2].len() == 1
            && parts[2].chars().next().unwrap().is_ascii_alphabetic()
        {
            // This is a Windows path - reconstruct command with drive letter
            let command = format!("{}:{}", parts[2], parts[3]);
            (command, 4)
        } else {
            (parts[2].to_string(), 3)
        };

        let args: Vec<String> = parts[args_start_idx..]
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Validate transport type
        let transport = match transport_str.as_str() {
            "stdio" => TransportType::Stdio,
            _ => {
                return Err(InspectorError::Config(format!(
                    "Unsupported transport type '{}'. Only 'stdio' is supported.",
                    transport_str
                )))
            }
        };

        Ok(ServerConfig {
            name,
            transport,
            params: ConnectionParams {
                command,
                args,
                env: HashMap::new(),
            },
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsl_parse_simple() {
        let result = ServerConfig::from_dsl("fa:stdio:/path/to/fa.exe");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "fa");
        assert_eq!(config.transport, TransportType::Stdio);
        assert_eq!(config.params.command, "/path/to/fa.exe");
        assert!(config.params.args.is_empty());
    }

    #[test]
    fn test_dsl_parse_with_args() {
        let result = ServerConfig::from_dsl("ta:stdio:/path/to/ta.exe:--verbose:--debug");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "ta");
        assert_eq!(config.transport, TransportType::Stdio);
        assert_eq!(config.params.command, "/path/to/ta.exe");
        assert_eq!(config.params.args, vec!["--verbose", "--debug"]);
    }

    #[test]
    fn test_dsl_parse_windows_path() {
        let result = ServerConfig::from_dsl("fa:stdio:C:/path/to/fa.exe");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.params.command, "C:/path/to/fa.exe");
        assert!(config.params.args.is_empty());
    }

    #[test]
    fn test_dsl_parse_windows_path_with_args() {
        let result = ServerConfig::from_dsl("fa:stdio:D:/tools/server.exe:--port:8080");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.params.command, "D:/tools/server.exe");
        assert_eq!(config.params.args, vec!["--port", "8080"]);
    }

    #[test]
    fn test_dsl_parse_case_insensitive_transport() {
        let result = ServerConfig::from_dsl("test:STDIO:/path/to/test.exe");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.transport, TransportType::Stdio);
    }

    #[test]
    fn test_dsl_parse_invalid_too_few_parts() {
        let result = ServerConfig::from_dsl("invalid");
        assert!(result.is_err());
        match result {
            Err(InspectorError::Config(msg)) => {
                assert!(msg.contains("Invalid DSL format"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_dsl_parse_invalid_transport() {
        let result = ServerConfig::from_dsl("test:http:/path/to/test.exe");
        assert!(result.is_err());
        match result {
            Err(InspectorError::Config(msg)) => {
                assert!(msg.contains("Unsupported transport type"));
                assert!(msg.contains("http"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_dsl_parse_with_single_arg() {
        let result = ServerConfig::from_dsl("server:stdio:/bin/server:--port");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.params.args, vec!["--port"]);
    }

    #[test]
    fn test_dsl_parse_with_many_args() {
        let result =
            ServerConfig::from_dsl("server:stdio:/bin/server:--verbose:--port:8080:--host:localhost");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.params.args,
            vec!["--verbose", "--port", "8080", "--host", "localhost"]
        );
    }
}
