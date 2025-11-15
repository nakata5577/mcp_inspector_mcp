use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Samplingをトリガーするツールのリクエストパラメータ
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TriggerSamplingParams {
    /// テストメッセージ（LLMに送信するユーザーメッセージ）
    pub message: String,
}

/// モックサーバーハンドラー
#[derive(Clone)]
pub struct MockSamplingServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MockSamplingServer {
    /// Create a new MockSamplingServer
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Samplingリクエストを送信するテストツール
    #[tool(
        name = "trigger_sampling",
        description = "Trigger a sampling request for testing MonitoringTransport"
    )]
    async fn trigger_sampling(
        &self,
        params: Parameters<TriggerSamplingParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let message = params.0.message.clone();

        tracing::info!("Triggering sampling request with message: {}", message);

        // Samplingリクエストのパラメータを構築
        let sampling_params = CreateMessageRequestParam {
            messages: vec![SamplingMessage {
                role: Role::User,
                content: Content::text(message.clone()),
            }],
            model_preferences: Some(ModelPreferences {
                hints: Some(vec![ModelHint {
                    name: Some("claude-3-5-sonnet".to_string()),
                }]),
                cost_priority: None,
                speed_priority: None,
                intelligence_priority: None,
            }),
            system_prompt: Some("You are a helpful assistant for testing.".to_string()),
            include_context: Some(ContextInclusion::None),
            max_tokens: 100,
            stop_sequences: Some(vec![]),
            metadata: None,
            temperature: Some(0.7),
        };

        // Samplingリクエストを送信
        // RequestContext.peerからcreate_messageを呼び出す
        match context.peer.create_message(sampling_params).await {
            Ok(result) => {
                tracing::info!("Sampling request succeeded");

                // レスポンスを整形
                let response_text = if let Some(text_content) = result.message.content.as_text() {
                    text_content.text.clone()
                } else {
                    "Non-text response received".to_string()
                };

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Sampling request sent successfully.\nOriginal message: {}\nLLM Response: {}",
                    message, response_text
                ))]))
            }
            Err(e) => {
                tracing::warn!("Sampling request failed: {:?}", e);

                // エラーの場合でも、リクエスト送信自体は成功したことを示す
                // （MonitoringTransportの検出テストが目的）
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Sampling request sent (but failed on host side).\nOriginal message: {}\nError: {:?}",
                    message, e
                ))]))
            }
        }
    }
}

impl ServerHandler for MockSamplingServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation {
                name: "mock-sampling-server".into(),
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
        tracing::info!("Mock Sampling Server initializing");
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
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Tool call requested: {}", request.name);

        match request.name.as_ref() {
            "trigger_sampling" => {
                let params_value = request
                    .arguments
                    .map(serde_json::Value::Object)
                    .unwrap_or(serde_json::json!({}));

                let params: Parameters<TriggerSamplingParams> =
                    serde_json::from_value(params_value).map_err(|e| McpError {
                        code: ErrorCode(-32602),
                        message: format!("Invalid parameters: {}", e).into(),
                        data: None,
                    })?;

                self.trigger_sampling(params, context).await
            }
            _ => Err(McpError {
                code: ErrorCode(-32601),
                message: format!("Unknown tool: {}", request.name).into(),
                data: None,
            }),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    tracing::info!("Starting Mock Sampling Server");

    // サーバー作成
    let handler = MockSamplingServer::new();

    // stdio Transportでサーバー起動
    use rmcp::service::ServiceExt;
    let service = handler
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| anyhow::anyhow!("Server initialization error: {}", e))?;

    tracing::info!("Mock Sampling Server initialized successfully");

    // サーバー実行
    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
