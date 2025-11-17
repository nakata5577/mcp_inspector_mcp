use crate::client::{McpClient, StdioClient};
use crate::models::{InspectorError, Result, ServerConfig};
use crate::services::SamplingLogger;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages MCP client connections to multiple servers
///
/// This manager implements connection pooling to reuse client connections
/// across multiple requests, improving performance by avoiding repeated
/// connection establishment overhead.
pub struct ClientManager {
    configs: HashMap<String, ServerConfig>,
    clients: Arc<RwLock<HashMap<String, Arc<StdioClient>>>>,
    sampling_logger: Arc<SamplingLogger>,
}

impl ClientManager {
    /// Create a new ClientManager with the given server configurations
    pub fn new(configs: Vec<ServerConfig>, sampling_logger: Arc<SamplingLogger>) -> Self {
        let configs_map = configs
            .into_iter()
            .map(|config| (config.name.clone(), config))
            .collect();

        Self {
            configs: configs_map,
            clients: Arc::new(RwLock::new(HashMap::new())),
            sampling_logger,
        }
    }

    /// Get a client for the specified server, creating a new connection if necessary
    ///
    /// This method implements connection pooling:
    /// - If a connected client exists, it will be reused (returns Arc clone)
    /// - If a client exists but is disconnected, a new one will be created
    /// - If no client exists, a new one will be created
    ///
    /// Connection pooling improves performance by avoiding repeated connection
    /// establishment overhead on subsequent requests to the same server.
    ///
    /// # Arguments
    /// * `server_name` - The name of the server to connect to
    ///
    /// # Returns
    /// A boxed McpClient instance (wrapping Arc<StdioClient>)
    ///
    /// # Errors
    /// Returns an error if the server is not found in the configuration
    pub async fn get_client(&self, server_name: &str) -> Result<Box<dyn McpClient>> {
        // First, check if a connected client already exists (read lock)
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(server_name) {
                if client.is_connected().await {
                    tracing::debug!(
                        server = server_name,
                        "Reusing existing connected client from pool"
                    );
                    // Return an Arc clone wrapped in a Box
                    return Ok(Box::new(Arc::clone(client)));
                } else {
                    tracing::debug!(
                        server = server_name,
                        "Existing client is disconnected, creating new connection"
                    );
                }
            }
        }

        // Get server configuration
        let config = self
            .configs
            .get(server_name)
            .ok_or_else(|| InspectorError::ServerNotFound(server_name.to_string()))?
            .clone();

        // Create new client based on transport type
        let client = Arc::new(StdioClient::new(
            config.clone(),
            Arc::clone(&self.sampling_logger),
        ));

        // IMPORTANT: Establish connection before adding to pool
        // This ensures the client is ready to use when returned
        client.connect_if_needed().await?;

        // Store the client in the pool (write lock)
        {
            let mut clients = self.clients.write().await;
            clients.insert(server_name.to_string(), Arc::clone(&client));
        }

        tracing::debug!(
            server = server_name,
            "Created new client, established connection, and added to pool"
        );

        Ok(Box::new(client))
    }

    /// List all configured server names
    pub fn list_servers(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// Get configuration for a specific server
    pub fn get_config(&self, server_name: &str) -> Option<&ServerConfig> {
        self.configs.get(server_name)
    }

    /// Get a StdioClient for the specified server, creating if necessary
    ///
    /// This method is similar to `get_client()` but returns `Arc<StdioClient>`
    /// directly instead of a boxed trait object. This allows access to
    /// StdioClient-specific methods like `get_init_result()`.
    ///
    /// # Arguments
    /// * `server_name` - The name of the server to connect to
    ///
    /// # Returns
    /// An Arc<StdioClient> instance for the specified server
    ///
    /// # Errors
    /// Returns an error if the server is not found in the configuration
    pub async fn get_stdio_client(&self, server_name: &str) -> Result<Arc<StdioClient>> {
        // First, check if a connected client already exists (read lock)
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(server_name) {
                if client.is_connected().await {
                    tracing::debug!(
                        server = server_name,
                        "Reusing existing connected StdioClient from pool"
                    );
                    return Ok(Arc::clone(client));
                } else {
                    tracing::debug!(
                        server = server_name,
                        "Existing client is disconnected, creating new connection"
                    );
                }
            }
        }

        // Get server configuration
        let config = self
            .configs
            .get(server_name)
            .ok_or_else(|| InspectorError::ServerNotFound(server_name.to_string()))?
            .clone();

        // Create new client
        let client = Arc::new(StdioClient::new(
            config.clone(),
            Arc::clone(&self.sampling_logger),
        ));

        // IMPORTANT: Establish connection before adding to pool
        // This ensures the client is ready to use when returned
        client.connect_if_needed().await?;

        // Store the client in the pool (write lock)
        {
            let mut clients = self.clients.write().await;
            clients.insert(server_name.to_string(), Arc::clone(&client));
        }

        tracing::debug!(
            server = server_name,
            "Created new StdioClient, established connection, and added to pool"
        );

        Ok(client)
    }

    /// Clean up disconnected clients from the pool
    ///
    /// This method removes clients that are no longer connected,
    /// helping to free up resources and maintain a clean connection pool.
    ///
    /// # Example
    /// ```
    /// manager.cleanup_pool().await;
    /// ```
    pub async fn cleanup_pool(&self) {
        let mut clients = self.clients.write().await;

        // Collect disconnected server names
        let mut to_remove = Vec::new();
        for (server_name, client) in clients.iter() {
            if !client.is_connected().await {
                to_remove.push(server_name.clone());
            }
        }

        // Remove disconnected clients
        for server_name in to_remove {
            clients.remove(&server_name);
            tracing::debug!(
                server = server_name.as_str(),
                "Removed disconnected client from pool"
            );
        }
    }

    /// Get the number of clients currently in the pool
    ///
    /// This is useful for monitoring and debugging the connection pool.
    pub async fn pool_size(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
}
