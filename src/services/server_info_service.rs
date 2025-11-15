use crate::client::{McpClient, StdioClient};
use crate::models::{
    ConnectionStatus, PromptCapabilityInfo, ResourceCapabilityInfo, Result,
    ServerCapabilitiesInfo, ServerImplementation, ServerInspectResponse, ToolCapabilityInfo,
};
use std::sync::Arc;

/// Service for retrieving server configuration and capability information
pub struct ServerInfoService;

impl ServerInfoService {
    /// Get comprehensive server information including capabilities and connection status
    ///
    /// This method retrieves detailed information about a connected MCP server by:
    /// 1. Obtaining the InitializeResult from the server
    /// 2. Extracting server implementation details
    /// 3. Analyzing server capabilities
    /// 4. Determining connection status
    ///
    /// # Arguments
    /// * `client` - The StdioClient instance connected to the target server
    /// * `server_name` - Name of the server for response metadata
    ///
    /// # Returns
    /// A `ServerInspectResponse` containing complete server information
    ///
    /// # Errors
    /// Returns an error if:
    /// - The client is not connected
    /// - Failed to retrieve initialization result
    pub async fn get_server_info(
        client: Arc<StdioClient>,
        server_name: String,
    ) -> Result<ServerInspectResponse> {
        // Check connection status
        let is_connected = client.is_connected().await;
        if !is_connected {
            return Ok(ServerInspectResponse {
                server_name,
                implementation: ServerImplementation {
                    name: "Unknown".to_string(),
                    title: None,
                    version: "Unknown".to_string(),
                    website_url: None,
                },
                capabilities: ServerCapabilitiesInfo::default(),
                connection_status: ConnectionStatus::Disconnected,
                protocol_version: None,
                instructions: None,
            });
        }

        // Get InitializeResult from client
        let init_result = client.get_init_result().await?;

        // Extract server implementation information
        let implementation = ServerImplementation {
            name: init_result.server_info.name.clone(),
            title: init_result.server_info.title.clone(),
            version: init_result.server_info.version.clone(),
            website_url: init_result.server_info.website_url.clone(),
        };

        // Parse server capabilities
        let capabilities = Self::parse_capabilities(&init_result.capabilities);

        // Extract protocol version
        let protocol_version = Some(init_result.protocol_version.to_string());

        // Build and return response
        Ok(ServerInspectResponse {
            server_name,
            implementation,
            capabilities,
            connection_status: ConnectionStatus::Connected,
            protocol_version,
            instructions: init_result.instructions.clone(),
        })
    }

    /// Parse rmcp ServerCapabilities into our ServerCapabilitiesInfo
    ///
    /// This method converts the rmcp capability structure into our simplified
    /// representation, checking for the presence of each capability.
    fn parse_capabilities(
        caps: &rmcp::model::ServerCapabilities,
    ) -> ServerCapabilitiesInfo {
        ServerCapabilitiesInfo {
            logging: caps.logging.is_some(),
            experimental: caps.experimental.is_some(),
            completions: caps.completions.is_some(),
            prompts: PromptCapabilityInfo {
                supported: caps.prompts.is_some(),
                list_changed: caps
                    .prompts
                    .as_ref()
                    .and_then(|p| p.list_changed)
                    .unwrap_or(false),
            },
            resources: ResourceCapabilityInfo {
                supported: caps.resources.is_some(),
                subscribe: caps
                    .resources
                    .as_ref()
                    .and_then(|r| r.subscribe)
                    .unwrap_or(false),
                list_changed: caps
                    .resources
                    .as_ref()
                    .and_then(|r| r.list_changed)
                    .unwrap_or(false),
            },
            tools: ToolCapabilityInfo {
                supported: caps.tools.is_some(),
                list_changed: caps
                    .tools
                    .as_ref()
                    .and_then(|t| t.list_changed)
                    .unwrap_or(false),
            },
        }
    }
}

impl Default for ServerCapabilitiesInfo {
    fn default() -> Self {
        Self {
            logging: false,
            experimental: false,
            completions: false,
            prompts: PromptCapabilityInfo {
                supported: false,
                list_changed: false,
            },
            resources: ResourceCapabilityInfo {
                supported: false,
                subscribe: false,
                list_changed: false,
            },
            tools: ToolCapabilityInfo {
                supported: false,
                list_changed: false,
            },
        }
    }
}
