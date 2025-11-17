use crate::client::ClientManager;
use crate::models::{
    HealthCheckResponse, InspectorConfig, LoggingMessagesRequest, LoggingMessagesResponse,
    PromptGetRequest, PromptGetResponse, PromptsListRequest, PromptsListResponse,
    ResourceReadRequest, ResourceReadResponse, ResourcesListRequest, ResourcesListResponse, Result,
    SamplingLogsRequest, SamplingLogsResponse, ServerInspectResponse, ToolCallRequest,
    ToolCallResponse, ToolsListResponse,
};
use crate::services::{
    create_logger, HealthChecker, LoggingInspector, ResponseCache, SamplingLogger,
    ServerInfoService,
};
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
    logging_inspector: Arc<LoggingInspector>,
    response_cache: Arc<ResponseCache>,
    health_checker: Arc<HealthChecker>,
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

        let sampling_logger = Arc::new(SamplingLogger::new(Arc::clone(&logger_backend)));
        let logging_inspector = Arc::new(LoggingInspector::new(logger_backend));

        // Create response cache with default 5-minute TTL
        let response_cache = Arc::new(ResponseCache::new(Duration::from_secs(300)));

        let client_manager = Arc::new(ClientManager::new(
            config.servers,
            Arc::clone(&sampling_logger),
        ));

        // Create health checker
        let health_checker = Arc::new(HealthChecker::new(Arc::clone(&client_manager)));

        Ok(Self {
            client_manager,
            sampling_logger,
            logging_inspector,
            response_cache,
            health_checker,
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

    pub async fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse> {
        // DEBUG: Log the request
        tracing::info!("=== INSPECTOR CALL_TOOL DEBUG ===");
        tracing::info!("Request server: {}", request.server);
        tracing::info!("Request tool_name: {}", request.tool_name);
        tracing::info!("Request arguments: {:?}", request.arguments);

        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        tracing::info!("Client acquired for server: {}", request.server);

        let result = client
            .call_tool(&request.tool_name, request.arguments)
            .await
            .context("Failed to call tool on server")?;

        tracing::info!("Tool result from client: {:?}", result);

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

    // ========== Server Inspection Methods ==========

    /// Inspect server configuration and capabilities
    ///
    /// This method retrieves comprehensive information about a target MCP server,
    /// including its implementation details, capabilities, and connection status.
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the target server to inspect
    ///
    /// # Returns
    ///
    /// A `ServerInspectResponse` containing detailed server information
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The server is not found in the configuration
    /// - Failed to connect to the server
    /// - Failed to retrieve server information
    ///
    /// # Example
    ///
    /// ```
    /// let info = service.server_inspect("my-server").await?;
    /// println!("Server: {} v{}", info.implementation.name, info.implementation.version);
    /// println!("Tools supported: {}", info.capabilities.tools.supported);
    /// ```
    pub async fn server_inspect(&self, server_name: &str) -> Result<ServerInspectResponse> {
        // Get StdioClient for the target server
        let client = self
            .client_manager
            .get_stdio_client(server_name)
            .await
            .context("Failed to get stdio client")?;

        // Retrieve server information using ServerInfoService
        Ok(
            ServerInfoService::get_server_info(client, server_name.to_string())
                .await
                .context("Failed to get server information")?,
        )
    }

    /// Perform a health check on the specified server
    ///
    /// This method sends a ping request to the server, measures the response time,
    /// and determines the server's health status based on response time and error rate.
    ///
    /// # Arguments
    /// * `server_name` - The name of the server to check
    ///
    /// # Returns
    /// A `HealthCheckResponse` containing:
    /// - Health status (Healthy, Degraded, or Unhealthy)
    /// - Response time in milliseconds
    /// - Error count and error rate from recent checks
    /// - Timestamp of the check
    /// - Error details if the check failed
    ///
    /// # Errors
    /// Returns an error if:
    /// - The server is not found in the configuration
    /// - Failed to create or get client connection
    ///
    /// # Example
    /// ```
    /// let health = service.health_check("my-server").await?;
    /// println!("Server status: {:?}", health.status);
    /// println!("Response time: {}ms", health.response_time_ms);
    /// ```
    pub async fn health_check(&self, server_name: &str) -> Result<HealthCheckResponse> {
        Ok(self
            .health_checker
            .check_health(server_name)
            .await
            .context("Failed to perform health check")?)
    }

    // ========== Logging Inspection Methods ==========

    /// Retrieve logging messages from a server with filtering
    ///
    /// This method retrieves log messages that were sent by the target MCP server
    /// via the `notifications/message` protocol. Messages can be filtered by log level,
    /// time range, and limited by count.
    ///
    /// # Arguments
    ///
    /// * `request` - Request parameters including:
    ///   - `server`: Name of the target server
    ///   - `level`: Optional minimum log level filter (e.g., "info", "warning", "error")
    ///   - `limit`: Maximum number of messages to return
    ///   - `since`: Optional timestamp to filter messages after
    ///
    /// # Returns
    ///
    /// A `LoggingMessagesResponse` containing:
    /// - `server_name`: Name of the server these messages came from
    /// - `messages`: Vector of log entries, sorted newest first
    /// - `total_count`: Total number of messages returned
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The log level string is invalid (not a recognized level)
    /// - The timestamp string is malformed (not RFC3339 format)
    /// - Failed to retrieve messages from the backend
    ///
    /// # Example
    ///
    /// ```
    /// use crate::models::LoggingMessagesRequest;
    ///
    /// let request = LoggingMessagesRequest {
    ///     server: "my-server".to_string(),
    ///     level: Some("warning".to_string()),
    ///     limit: 100,
    ///     since: Some("2025-01-15T12:00:00Z".to_string()),
    /// };
    ///
    /// let response = service.logging_messages(request).await?;
    /// for msg in &response.messages {
    ///     println!("[{}] {}: {}", msg.level, msg.logger.as_deref().unwrap_or(""), msg.message);
    /// }
    /// ```
    pub async fn logging_messages(
        &self,
        request: LoggingMessagesRequest,
    ) -> Result<LoggingMessagesResponse> {
        Ok(self
            .logging_inspector
            .get_logging_messages(request)
            .context("Failed to retrieve logging messages")?)
    }
}
