use serde::{Deserialize, Serialize};

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
