use crate::client::ClientManager;
use crate::models::{
    InspectorConfig, PromptGetRequest, PromptGetResponse, PromptsListRequest, PromptsListResponse,
    ResourceReadRequest, ResourceReadResponse, ResourcesListRequest, ResourcesListResponse, Result,
    SamplingLogsRequest, SamplingLogsResponse, ToolCallRequest, ToolCallResponse,
    ToolsListResponse,
};
use crate::services::{create_logger, SamplingLogger};
use anyhow::Context;
use std::sync::Arc;

/// Service for inspecting and interacting with MCP servers
#[derive(Clone)]
pub struct InspectorService {
    client_manager: Arc<ClientManager>,
    sampling_logger: Arc<SamplingLogger>,
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

        Ok(Self {
            client_manager: Arc::new(ClientManager::new(
                config.servers,
                Arc::clone(&sampling_logger),
            )),
            sampling_logger,
        })
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

    /// List all resources available on the specified server
    pub async fn list_resources(
        &self,
        request: ResourcesListRequest,
    ) -> Result<ResourcesListResponse> {
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let resources = client
            .list_resources()
            .await
            .context("Failed to list resources from server")?;

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
    pub async fn list_prompts(&self, request: PromptsListRequest) -> Result<PromptsListResponse> {
        let client = self
            .client_manager
            .get_client(&request.server)
            .await
            .context("Failed to get client")?;

        let prompts = client
            .list_prompts()
            .await
            .context("Failed to list prompts from server")?;

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
}
