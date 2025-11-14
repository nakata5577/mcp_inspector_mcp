use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to list tools from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListRequest {
    pub server: String,
}

/// Request to call a tool on a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub server: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Request to list resources from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListRequest {
    pub server: String,
}

/// Request to read a resource from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadRequest {
    pub server: String,
    pub uri: String,
}

/// Request to list prompts from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListRequest {
    pub server: String,
}

/// Request to get a prompt from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGetRequest {
    pub server: String,
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, String>,
}

/// Default limit for sampling logs
fn default_limit() -> usize {
    100
}

/// Default status filter for sampling logs
fn default_status() -> String {
    "all".to_string()
}

/// Request to get sampling logs from a specific MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogsRequest {
    /// Name of the target MCP server
    pub server: String,
    /// Maximum number of logs to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Filter by status: "all", "success", "failed" (default: "all")
    #[serde(default = "default_status")]
    pub status: String,
}
