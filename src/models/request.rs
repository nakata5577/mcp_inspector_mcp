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
