use serde::{Deserialize, Serialize};

/// Information about a tool available on an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// Response containing list of tools from a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResponse {
    pub server: String,
    pub tools: Vec<ToolInfo>,
}

/// Response from calling a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub server: String,
    pub tool_name: String,
    pub result: serde_json::Value,
}
