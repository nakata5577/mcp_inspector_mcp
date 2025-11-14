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

/// Status of a sampling request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SamplingStatus {
    /// Request is pending
    Pending,
    /// Request completed successfully
    Success,
    /// Request failed
    Failed,
}

/// Content in a sampling message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingContent {
    /// Type of content (e.g., "text", "image")
    #[serde(rename = "type")]
    pub content_type: String,
    /// Text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Message in a sampling request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// Role of the message sender (e.g., "user", "assistant")
    pub role: String,
    /// Content of the message
    pub content: SamplingContent,
}

/// Hint for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    /// Name of the suggested model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Model preferences for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Hints for model selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
    /// Cost priority (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    /// Speed priority (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    /// Intelligence priority (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f64>,
}

/// Entry for a single sampling log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// Timestamp when the request was made (ISO 8601 format)
    pub timestamp: String,
    /// Status of the sampling request
    pub status: SamplingStatus,
    /// Messages in the sampling request
    pub messages: Vec<SamplingMessage>,
    /// Model preferences specified in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    /// System prompt used in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Maximum tokens requested
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Error message if the request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response content if the request succeeded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
}

/// Response containing sampling logs from a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingLogsResponse {
    /// Name of the target MCP server
    pub server: String,
    /// List of sampling log entries
    pub logs: Vec<SamplingLogEntry>,
    /// Total count of logs for this server
    pub total_count: usize,
}
