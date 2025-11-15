use crate::client::ClientManager;
use crate::models::{
    InspectorConfig, PromptGetRequest, PromptGetResponse, PromptsListRequest, PromptsListResponse,
    ResourceReadRequest, ResourceReadResponse, ResourcesListRequest, ResourcesListResponse, Result,
    SamplingLogsRequest, SamplingLogsResponse, ToolCallRequest, ToolCallResponse,
    ToolsListResponse,
};
use crate::services::{create_logger, ResponseCache, SamplingLogger};
use anyhow::Context;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

/// Service for inspecting and interacting with MCP servers
#[derive(Clone)]
pub struct InspectorService {
    client_manager: Arc<ClientManager>,
    sampling_logger: Arc<SamplingLogger>,
    response_cache: Arc<ResponseCache>,
}

impl InspectorService {
    /// Create a new InspectorService with the given configuration
    ///
    /// # Arguments
    /// * `config` - Inspector configuration including server configs and logging settings
    ///
    /// # Errors
    /// Returns an error if:
    /// - Logging configuration is invalid
    /// - Logger backend cannot be created
    pub fn new(config: InspectorConfig) -> anyhow::Result<Self> {
        // Create logger backend from configuration
        let logger_backend =
            create_logger(&config.logging).context("Failed to create logger backend")?;

        let sampling_logger = Arc::new(SamplingLogger::new(logger_backend));

        // Create response cache with default 5-minute TTL
        let response_cache = Arc::new(ResponseCache::new(Duration::from_secs(300)));

        Ok(Self {
            client_manager: Arc::new(ClientManager::new(
                config.servers,
                Arc::clone(&sampling_logger),
            )),
            sampling_logger,
            response_cache,
        })
    }

    /// List all tools available on the specified server
    ///
    /// This method uses caching to improve performance. If a valid cached
    /// response exists, it will be returned immediately. Otherwise, the
    /// server will be queried and the result will be cached.
    pub async fn list_tools(&self, server_name: &str) -> Result<ToolsListResponse> {
        // Try to get from cache first
        if let Some(tools) = self.response_cache.get_tools(server_name).await {
            return Ok(ToolsListResponse {
                server: server_name.to_string(),
                tools,
            });
        }

        // Cache miss - fetch from server
        let client = self
            .client_manager
            .get_client(server_name)
            .await
            .context("Failed to get client")?;

        let tools = client
            .list_tools()
            .await
            .context("Failed to list tools from server")?;

        // Cache the result
        self.response_cache
            .set_tools(
                server_name.to_string(),
                tools.clone(),
                self.response_cache.default_ttl(),
            )
            .await;

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

    /// List all resources available on the specified server
    ///
    /// This method uses caching to improve performance. If a valid cached
    /// response exists, it will be returned immediately. Otherwise, the
    /// server will be queried and the result will be cached.
    pub async fn list_resources(
        &self,
        request: ResourcesListRequest,
    ) -> Result<ResourcesListResponse> {
        // Try to get from cache first
        if let Some(resources) = self.response_cache.get_resources(&request.server).await {
            return Ok(ResourcesListResponse {
                server: request.server,
                resources,
            });
        }

        // Cache miss - fetch from server
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let resources = client
            .list_resources()
            .await
            .context("Failed to list resources from server")?;

        // Cache the result
        self.response_cache
            .set_resources(
                request.server.clone(),
                resources.clone(),
                self.response_cache.default_ttl(),
            )
            .await;

        Ok(ResourcesListResponse {
            server: request.server,
            resources,
        })
    }

    /// Read a specific resource from the specified server
    pub async fn read_resource(
        &self,
        request: ResourceReadRequest,
    ) -> Result<ResourceReadResponse> {
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let contents = client
            .read_resource(&request.uri)
            .await
            .context(format!("Failed to read resource: {}", request.uri))?;

        Ok(ResourceReadResponse {
            server: request.server,
            uri: request.uri,
            contents,
        })
    }

    /// List all prompts available on the specified server
    ///
    /// This method uses caching to improve performance. If a valid cached
    /// response exists, it will be returned immediately. Otherwise, the
    /// server will be queried and the result will be cached.
    pub async fn list_prompts(&self, request: PromptsListRequest) -> Result<PromptsListResponse> {
        // Try to get from cache first
        if let Some(prompts) = self.response_cache.get_prompts(&request.server).await {
            return Ok(PromptsListResponse {
                server: request.server,
                prompts,
            });
        }

        // Cache miss - fetch from server
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let prompts = client
            .list_prompts()
            .await
            .context("Failed to list prompts from server")?;

        // Cache the result
        self.response_cache
            .set_prompts(
                request.server.clone(),
                prompts.clone(),
                self.response_cache.default_ttl(),
            )
            .await;

        Ok(PromptsListResponse {
            server: request.server,
            prompts,
        })
    }

    /// Get a specific prompt from the specified server
    pub async fn get_prompt(&self, request: PromptGetRequest) -> Result<PromptGetResponse> {
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let messages = client
            .get_prompt(&request.name, request.arguments)
            .await
            .context(format!("Failed to get prompt: {}", request.name))?;

        Ok(PromptGetResponse {
            server: request.server,
            name: request.name,
            messages,
        })
    }

    /// Get sampling logs from the specified server
    ///
    /// This method retrieves sampling logs filtered by the provided criteria.
    /// Currently, the logs are stored in memory, but this design allows for
    /// future extensions (e.g., database persistence or file storage).
    ///
    /// # Arguments
    ///
    /// * `request` - The request containing filter criteria (server, limit, status)
    ///
    /// # Returns
    ///
    /// A response containing the filtered logs and total count
    pub async fn sampling_logs(
        &self,
        request: SamplingLogsRequest,
    ) -> Result<SamplingLogsResponse> {
        let logs = self
            .sampling_logger
            .get_logs(&request.server, request.limit, &request.status);

        let total_count = self.sampling_logger.count_logs(&request.server);

        Ok(SamplingLogsResponse {
            server: request.server,
            logs,
            total_count,
        })
    }

    /// Invalidate cache for a specific server
    ///
    /// This removes all cached data (tools, resources, prompts) for the given server.
    /// Useful when you know the server's state has changed and want to force a refresh.
    ///
    /// # Arguments
    ///
    /// * `server_name` - The server name to invalidate
    pub async fn invalidate_cache(&self, server_name: &str) {
        self.response_cache.invalidate(server_name).await;
    }

    /// Invalidate all caches
    ///
    /// This removes all cached data for all servers.
    pub async fn invalidate_all_caches(&self) {
        self.response_cache.invalidate_all().await;
    }

    /// Get cache statistics
    ///
    /// Returns a tuple of (tools_count, resources_count, prompts_count)
    pub async fn cache_stats(&self) -> (usize, usize, usize) {
        self.response_cache.stats().await
    }

    // ========== Parallel Processing Methods ==========

    /// List tools from multiple servers in parallel
    ///
    /// This method fetches tool lists from multiple servers concurrently,
    /// significantly improving performance when querying multiple servers.
    ///
    /// # Arguments
    ///
    /// * `servers` - List of server names to query
    ///
    /// # Returns
    ///
    /// A HashMap mapping server names to their tool lists.
    /// If a server fails, it will be omitted from the result (logged as warning).
    ///
    /// # Example
    ///
    /// ```
    /// let servers = vec!["server1".to_string(), "server2".to_string()];
    /// let results = service.list_tools_batch(servers).await?;
    /// ```
    pub async fn list_tools_batch(
        &self,
        servers: Vec<String>,
    ) -> anyhow::Result<HashMap<String, ToolsListResponse>> {
        let mut tasks = JoinSet::new();

        // Spawn a task for each server
        for server in servers {
            let service = self.clone();
            tasks.spawn(async move {
                let result = service.list_tools(&server).await;
                (server.clone(), result)
            });
        }

        let mut results = HashMap::new();

        // Collect results from all tasks
        while let Some(task_result) = tasks.join_next().await {
            match task_result {
                Ok((server, Ok(response))) => {
                    results.insert(server, response);
                }
                Ok((server, Err(e))) => {
                    tracing::warn!(
                        server = server.as_str(),
                        error = ?e,
                        "Failed to list tools"
                    );
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Task join error");
                }
            }
        }

        Ok(results)
    }

    /// List resources from multiple servers in parallel
    ///
    /// This method fetches resource lists from multiple servers concurrently,
    /// significantly improving performance when querying multiple servers.
    ///
    /// # Arguments
    ///
    /// * `servers` - List of server names to query
    ///
    /// # Returns
    ///
    /// A HashMap mapping server names to their resource lists.
    /// If a server fails, it will be omitted from the result (logged as warning).
    pub async fn list_resources_batch(
        &self,
        servers: Vec<String>,
    ) -> anyhow::Result<HashMap<String, ResourcesListResponse>> {
        let mut tasks = JoinSet::new();

        // Spawn a task for each server
        for server in servers {
            let service = self.clone();
            tasks.spawn(async move {
                let result = service
                    .list_resources(ResourcesListRequest {
                        server: server.clone(),
                    })
                    .await;
                (server.clone(), result)
            });
        }

        let mut results = HashMap::new();

        // Collect results from all tasks
        while let Some(task_result) = tasks.join_next().await {
            match task_result {
                Ok((server, Ok(response))) => {
                    results.insert(server, response);
                }
                Ok((server, Err(e))) => {
                    tracing::warn!(
                        server = server.as_str(),
                        error = ?e,
                        "Failed to list resources"
                    );
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Task join error");
                }
            }
        }

        Ok(results)
    }

    /// List prompts from multiple servers in parallel
    ///
    /// This method fetches prompt lists from multiple servers concurrently,
    /// significantly improving performance when querying multiple servers.
    ///
    /// # Arguments
    ///
    /// * `servers` - List of server names to query
    ///
    /// # Returns
    ///
    /// A HashMap mapping server names to their prompt lists.
    /// If a server fails, it will be omitted from the result (logged as warning).
    pub async fn list_prompts_batch(
        &self,
        servers: Vec<String>,
    ) -> anyhow::Result<HashMap<String, PromptsListResponse>> {
        let mut tasks = JoinSet::new();

        // Spawn a task for each server
        for server in servers {
            let service = self.clone();
            tasks.spawn(async move {
                let result = service
                    .list_prompts(PromptsListRequest {
                        server: server.clone(),
                    })
                    .await;
                (server.clone(), result)
            });
        }

        let mut results = HashMap::new();

        // Collect results from all tasks
        while let Some(task_result) = tasks.join_next().await {
            match task_result {
                Ok((server, Ok(response))) => {
                    results.insert(server, response);
                }
                Ok((server, Err(e))) => {
                    tracing::warn!(
                        server = server.as_str(),
                        error = ?e,
                        "Failed to list prompts"
                    );
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Task join error");
                }
            }
        }

        Ok(results)
    }
}
