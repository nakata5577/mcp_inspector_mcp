use crate::client::{McpClient, StdioClient};
use crate::models::{InspectorError, Result, ServerConfig, TransportType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages MCP client connections to multiple servers
pub struct ClientManager {
    configs: HashMap<String, ServerConfig>,
    clients: Arc<RwLock<HashMap<String, Box<dyn McpClient>>>>,
}

impl ClientManager {
    /// Create a new ClientManager with the given server configurations
    pub fn new(configs: Vec<ServerConfig>) -> Self {
        let configs_map = configs
            .into_iter()
            .map(|config| (config.name.clone(), config))
            .collect();

        Self {
            configs: configs_map,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a client for the specified server, creating a new connection if necessary
    pub async fn get_client(&self, server_name: &str) -> Result<Box<dyn McpClient>> {
        // Check if client already exists
        {
            let clients = self.clients.read().await;
            if let Some(_client) = clients.get(server_name) {
                // For MVP, we'll create a new client each time
                // In future phases, we can implement connection pooling
            }
        }

        // Get server configuration
        let config = self
            .configs
            .get(server_name)
            .ok_or_else(|| InspectorError::ServerNotFound(server_name.to_string()))?
            .clone();

        // Create new client based on transport type
        let client: Box<dyn McpClient> = match config.transport {
            TransportType::Stdio => Box::new(StdioClient::new(config)),
        };

        Ok(client)
    }

    /// List all configured server names
    pub fn list_servers(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Get configuration for a specific server
    pub fn get_config(&self, server_name: &str) -> Option<&ServerConfig> {
        self.configs.get(server_name)
    }
}
