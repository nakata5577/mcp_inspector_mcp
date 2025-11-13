use crate::client::ClientManager;
use crate::models::{Result, ServerConfig, ToolCallRequest, ToolCallResponse, ToolsListResponse};
use anyhow::Context;
use std::sync::Arc;

/// Service for inspecting and interacting with MCP servers
#[derive(Clone)]
pub struct InspectorService {
    client_manager: Arc<ClientManager>,
}

impl InspectorService {
    /// Create a new InspectorService with the given server configurations
    pub fn new(configs: Vec<ServerConfig>) -> Self {
        Self {
            client_manager: Arc::new(ClientManager::new(configs)),
        }
    }

    /// List all tools available on the specified server
    pub async fn list_tools(&self, server_name: &str) -> Result<ToolsListResponse> {
        let client = self
            .client_manager
            .get_client(server_name)
            .await
            .context("Failed to get client")?;

        let tools = client
            .list_tools()
            .await
            .context("Failed to list tools from server")?;

        Ok(ToolsListResponse {
            server: server_name.to_string(),
            tools,
        })
    }

    /// Call a tool on the specified server with the given arguments
    pub async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse> {
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let result = client
            .call_tool(&request.tool_name, request.arguments)
            .await
            .context("Failed to call tool on server")?;

        Ok(ToolCallResponse {
            server: request.server,
            tool_name: request.tool_name,
            result,
        })
    }

    /// List all configured server names
    pub fn list_servers(&self) -> Vec<String> {
        self.client_manager.list_servers()
    }
}
