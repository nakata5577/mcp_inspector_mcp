use crate::models::{
    PromptGetRequest, PromptsListRequest, ResourceReadRequest, ResourcesListRequest,
    ToolCallRequest,
};
use crate::services::InspectorService;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use rmcp::{handler::server::tool::ToolRouter, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

/// MCP Server implementation for Inspector
#[derive(Clone)]
pub struct InspectorServer {
    inspector: Arc<InspectorService>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl InspectorServer {
    /// Create a new InspectorServer
    pub fn new(inspector: InspectorService) -> Self {
        Self {
            inspector: Arc::new(inspector),
            tool_router: Self::tool_router(),
        }
    }

    /// List all available tools on a specific MCP server
    #[tool(
        name = "tools_list",
        description = "List all available tools on a specific MCP server"
    )]
    async fn tools_list(
        &self,
        params: Parameters<ToolsListParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .inspector
            .list_tools(&params.0.server)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to list tools: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }

    /// Call a tool on a specific MCP server with the given arguments
    #[tool(
        name = "tools_call",
        description = "Call a tool on a specific MCP server with the given arguments"
    )]
    async fn tools_call(
        &self,
        params: Parameters<ToolsCallParams>,
    ) -> Result<CallToolResult, McpError> {
        let call_request = ToolCallRequest {
            server: params.0.server.clone(),
            tool_name: params.0.tool_name.clone(),
            arguments: params.0.arguments.clone().unwrap_or_else(|| serde_json::json!({})),
        };

        let result = self
            .inspector
            .call_tool(call_request)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to call tool: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }

    /// List all resources available on a specific MCP server
    #[tool(
        name = "resources_list",
        description = "指定されたMCPサーバーが提供するリソースの一覧を取得します"
    )]
    async fn resources_list(
        &self,
        params: Parameters<ResourcesListParams>,
    ) -> Result<CallToolResult, McpError> {
        let request = ResourcesListRequest {
            server: params.0.server.clone(),
        };

        let result = self
            .inspector
            .list_resources(request)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to list resources: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }

    /// Read a specific resource from a specific MCP server
    #[tool(
        name = "resources_read",
        description = "指定されたMCPサーバーの特定のリソースを読み込みます"
    )]
    async fn resources_read(
        &self,
        params: Parameters<ResourceReadParams>,
    ) -> Result<CallToolResult, McpError> {
        let request = ResourceReadRequest {
            server: params.0.server.clone(),
            uri: params.0.uri.clone(),
        };

        let result = self
            .inspector
            .read_resource(request)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to read resource: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }

    /// List all prompts available on a specific MCP server
    #[tool(
        name = "prompts_list",
        description = "指定されたMCPサーバーが提供するプロンプトテンプレートの一覧を取得します"
    )]
    async fn prompts_list(
        &self,
        params: Parameters<PromptsListParams>,
    ) -> Result<CallToolResult, McpError> {
        let request = PromptsListRequest {
            server: params.0.server.clone(),
        };

        let result = self
            .inspector
            .list_prompts(request)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to list prompts: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }

    /// Get a specific prompt from a specific MCP server
    #[tool(
        name = "prompts_get",
        description = "指定されたMCPサーバーの特定のプロンプトテンプレートを取得します"
    )]
    async fn prompts_get(
        &self,
        params: Parameters<PromptGetParams>,
    ) -> Result<CallToolResult, McpError> {
        let request = PromptGetRequest {
            server: params.0.server.clone(),
            name: params.0.name.clone(),
            arguments: params.0.arguments.clone().unwrap_or_default(),
        };

        let result = self
            .inspector
            .get_prompt(request)
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: format!("Failed to get prompt: {}", e).into(),
                data: None,
            })?;

        let json_result = serde_json::to_value(&result).map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: format!("JSON serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json_result)
                .unwrap_or_else(|_| json_result.to_string()),
        )]))
    }
}

impl ServerHandler for InspectorServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation {
                name: "mcp-inspector".into(),
                version: "0.1.0".into(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: None,
            ..Default::default()
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        // Find and call the appropriate tool based on the name
        match request.name.as_ref() {
            "tools_list" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<ToolsListParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.tools_list(params).await
            }
            "tools_call" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<ToolsCallParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.tools_call(params).await
            }
            "resources_list" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<ResourcesListParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.resources_list(params).await
            }
            "resources_read" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<ResourceReadParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.resources_read(params).await
            }
            "prompts_list" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<PromptsListParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.prompts_list(params).await
            }
            "prompts_get" => {
                let params_value = request.arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<PromptGetParams> = serde_json::from_value(params_value)
                    .map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.prompts_get(params).await
            }
            _ => Err(McpError {
                code: ErrorCode(-32601),
                message: format!("Unknown tool: {}", request.name).into(),
                data: None,
            }),
        }
    }
}

/// Parameters for tools_list tool
#[derive(Deserialize, JsonSchema)]
struct ToolsListParams {
    /// Name of the MCP server to list tools from
    server: String,
}

/// Parameters for tools_call tool
#[derive(Deserialize, JsonSchema)]
struct ToolsCallParams {
    /// Name of the MCP server
    server: String,
    /// Name of the tool to call
    tool_name: String,
    /// Arguments to pass to the tool
    #[serde(default)]
    arguments: Option<serde_json::Value>,
}

/// Parameters for resources_list tool
#[derive(Deserialize, JsonSchema)]
struct ResourcesListParams {
    /// Name of the MCP server to list resources from
    server: String,
}

/// Parameters for resources_read tool
#[derive(Deserialize, JsonSchema)]
struct ResourceReadParams {
    /// Name of the MCP server
    server: String,
    /// URI of the resource to read
    uri: String,
}

/// Parameters for prompts_list tool
#[derive(Deserialize, JsonSchema)]
struct PromptsListParams {
    /// Name of the MCP server to list prompts from
    server: String,
}

/// Parameters for prompts_get tool
#[derive(Deserialize, JsonSchema)]
struct PromptGetParams {
    /// Name of the MCP server
    server: String,
    /// Name of the prompt to get
    name: String,
    /// Arguments to pass to the prompt
    #[serde(default)]
    arguments: Option<HashMap<String, String>>,
}

/// Run the MCP Inspector server
pub async fn run_server(inspector: InspectorService) -> anyhow::Result<()> {
    use rmcp::service::ServiceExt;

    let server = InspectorServer::new(inspector);

    tracing::info!("Starting MCP Inspector server");

    // Use stdio transport with the server handler
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Server initialization error: {}", e))?;

    tracing::info!("MCP Inspector server initialized successfully");

    // Wait for the service to complete
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
