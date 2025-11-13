pub mod manager;
pub mod stdio_client;

use crate::models::{Result, ToolInfo};
use async_trait::async_trait;

/// Trait for MCP client implementations
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Check if the client is connected to the server
    async fn is_connected(&self) -> bool;

    /// List all available tools on the server
    async fn list_tools(&self) -> Result<Vec<ToolInfo>>;

    /// Call a tool with the given name and arguments
    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value>;

    /// Disconnect from the server
    async fn disconnect(&mut self) -> Result<()>;
}

pub use manager::ClientManager;
pub use stdio_client::StdioClient;
