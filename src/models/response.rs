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

/// Information about a resource available on an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Response containing list of resources from a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResponse {
    pub server: String,
    pub resources: Vec<ResourceInfo>,
}

/// Content of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded binary data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Response from reading a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadResponse {
    pub server: String,
    pub uri: String,
    pub contents: Vec<ResourceContent>,
}

/// Argument definition for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
}

/// Information about a prompt available on an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

/// Response containing list of prompts from a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListResponse {
    pub server: String,
    pub prompts: Vec<PromptInfo>,
}

/// Message in a prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: serde_json::Value,
}

/// Response from getting a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGetResponse {
    pub server: String,
    pub name: String,
    pub messages: Vec<PromptMessage>,
}
