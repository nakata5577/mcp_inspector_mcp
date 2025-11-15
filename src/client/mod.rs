pub mod manager;
pub mod monitoring_transport;
pub mod stdio_client;

use crate::models::{PromptInfo, PromptMessage, ResourceContent, ResourceInfo, Result, ToolInfo};
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for MCP client implementations
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Check if the client is connected to the server
    async fn is_connected(&self) -> bool;

    /// List all available tools on the server
    async fn list_tools(&self) -> Result<Vec<ToolInfo>>;

    /// Call a tool with the given name and arguments
    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value>;

    /// List all available resources on the server
    async fn list_resources(&self) -> Result<Vec<ResourceInfo>>;

    /// Read a resource by URI
    async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>>;

    /// List all available prompts on the server
    async fn list_prompts(&self) -> Result<Vec<PromptInfo>>;

    /// Get a prompt by name with arguments
    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<Vec<PromptMessage>>;

    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<()>;
}

pub use manager::ClientManager;
pub use monitoring_transport::MonitoringTransport;
pub use stdio_client::StdioClient;
