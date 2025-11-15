use serde::{Deserialize, Serialize};

/// Request parameters for server_inspect tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInspectRequest {
    /// Name of the target MCP server
    pub server: String,
}

/// Response containing server configuration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInspectResponse {
    /// Server name
    pub server_name: String,
    /// Server implementation information
    pub implementation: ServerImplementation,
    /// Server capabilities
    pub capabilities: ServerCapabilitiesInfo,
    /// Connection status
    pub connection_status: ConnectionStatus,
    /// Optional protocol version
    pub protocol_version: Option<String>,
    /// Optional instructions from server
    pub instructions: Option<String>,
}

/// Server implementation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerImplementation {
    /// Implementation name
    pub name: String,
    /// Optional title
    pub title: Option<String>,
    /// Implementation version
    pub version: String,
    /// Optional website URL
    pub website_url: Option<String>,
}

/// Server capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesInfo {
    /// Whether logging capability is enabled
    pub logging: bool,
    /// Whether experimental capabilities are enabled
    pub experimental: bool,
    /// Whether completions capability is enabled
    pub completions: bool,
    /// Prompts capability details
    pub prompts: PromptCapabilityInfo,
    /// Resources capability details
    pub resources: ResourceCapabilityInfo,
    /// Tools capability details
    pub tools: ToolCapabilityInfo,
}

/// Prompts capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilityInfo {
    /// Whether prompts are supported
    pub supported: bool,
    /// Whether list_changed notification is supported
    pub list_changed: bool,
}

/// Resources capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilityInfo {
    /// Whether resources are supported
    pub supported: bool,
    /// Whether subscribe capability is supported
    pub subscribe: bool,
    /// Whether list_changed notification is supported
    pub list_changed: bool,
}

/// Tools capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilityInfo {
    /// Whether tools are supported
    pub supported: bool,
    /// Whether list_changed notification is supported
    pub list_changed: bool,
}

/// Connection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    /// Successfully connected
    Connected,
    /// Disconnected
    Disconnected,
    /// Error state
    Error,
}
