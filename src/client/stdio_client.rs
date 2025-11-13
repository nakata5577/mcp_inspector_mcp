use crate::client::McpClient;
use crate::models::{InspectorError, Result, ServerConfig, ToolInfo};
use anyhow::Context;
use async_trait::async_trait;
use rmcp::model::CallToolRequestParam;
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

/// MCP client using stdio transport
pub struct StdioClient {
    config: ServerConfig,
    service: Arc<Mutex<Option<RunningService<RoleClient, ()>>>>,
}

impl StdioClient {
    /// Create a new StdioClient from server configuration
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            service: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize connection to the MCP server
    async fn connect(&self) -> Result<()> {
        let mut guard = self.service.lock().await;

        if guard.is_some() {
            return Ok(());
        }

        let config = self.config.clone();

        // Build command with arguments and environment variables
        let transport = TokioChildProcess::new(Command::new(&config.params.command).configure(
            |cmd| {
                cmd.args(&config.params.args);
                for (key, value) in &config.params.env {
                    cmd.env(key, value);
                }
            },
        ))
        .map_err(|e| InspectorError::ConnectionFailed {
            server: config.name.clone(),
            source: e.into(),
        })?;

        // Create service and establish connection
        let service = ()
            .serve(transport)
            .await
            .map_err(|e| InspectorError::ConnectionFailed {
                server: config.name.clone(),
                source: e.into(),
            })?;

        *guard = Some(service);
        Ok(())
    }

    /// Get reference to the service, connecting if necessary
    async fn get_service(&self) -> Result<Arc<Mutex<Option<RunningService<RoleClient, ()>>>>> {
        self.connect().await?;
        Ok(Arc::clone(&self.service))
    }
}

#[async_trait]
impl McpClient for StdioClient {
    async fn is_connected(&self) -> bool {
        let guard = self.service.lock().await;
        guard.is_some()
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        let tools_response = service
            .list_tools(Default::default())
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context("Failed to list tools")?;

        let tools = tools_response
            .tools
            .into_iter()
            .map(|tool| ToolInfo {
                name: tool.name.to_string(),
                description: tool.description.map(|d| d.to_string()),
                input_schema: Some(serde_json::to_value(&*tool.input_schema).unwrap_or(serde_json::Value::Null)),
            })
            .collect();

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        let arguments_obj = arguments.as_object().cloned();

        let result = service
            .call_tool(CallToolRequestParam {
                name: name.to_string().into(),
                arguments: arguments_obj,
            })
            .await
            .map_err(|e| InspectorError::ToolExecutionFailed {
                server: self.config.name.clone(),
                tool: name.to_string(),
                source: e.into(),
            })?;

        Ok(serde_json::to_value(result).context("Failed to serialize tool result")?)
    }

    async fn disconnect(&mut self) -> Result<()> {
        let mut guard = self.service.lock().await;

        if let Some(service) = guard.take() {
            service
                .cancel()
                .await
                .map_err(|e| InspectorError::Internal(e.into()))
                .context("Failed to disconnect from server")?;
        }

        Ok(())
    }
}
