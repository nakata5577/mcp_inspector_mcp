use crate::client::{McpClient, MonitoringTransport};
use crate::models::{
    InspectorError, PromptArgument, PromptInfo, PromptMessage, ResourceContent, ResourceInfo,
    Result, ServerConfig, ToolInfo,
};
use crate::services::SamplingLogger;
use anyhow::Context;
use async_trait::async_trait;
use rmcp::model::{CallToolRequestParam, GetPromptRequestParam, ReadResourceRequestParam};
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

/// MCP client using stdio transport
pub struct StdioClient {
    config: ServerConfig,
    service: Arc<Mutex<Option<RunningService<RoleClient, ()>>>>,
    sampling_logger: Arc<SamplingLogger>,
}

impl StdioClient {
    /// Create a new StdioClient from server configuration
    pub fn new(config: ServerConfig, sampling_logger: Arc<SamplingLogger>) -> Self {
        Self {
            config,
            service: Arc::new(Mutex::new(None)),
            sampling_logger,
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
        let base_transport =
            TokioChildProcess::new(Command::new(&config.params.command).configure(|cmd| {
                cmd.args(&config.params.args);
                for (key, value) in &config.params.env {
                    cmd.env(key, value);
                }
            }))
            .map_err(|e| InspectorError::ConnectionFailed {
                server: config.name.clone(),
                source: e.into(),
            })?;

        // Wrap with MonitoringTransport to enable sampling monitoring
        let monitoring_transport = MonitoringTransport::new(
            base_transport,
            Arc::clone(&self.sampling_logger),
            config.name.clone(),
        );

        // Create service and establish connection
        let service =
            ().serve(monitoring_transport)
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
                input_schema: Some(
                    serde_json::to_value(&*tool.input_schema).unwrap_or(serde_json::Value::Null),
                ),
            })
            .collect();

        Ok(tools)
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
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

    async fn list_resources(&self) -> Result<Vec<ResourceInfo>> {
        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        let resources_response = service
            .list_resources(Default::default())
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context("Failed to list resources")?;

        let resources = resources_response
            .resources
            .into_iter()
            .map(|resource| ResourceInfo {
                uri: resource.uri.clone(),
                name: Some(resource.name.clone()),
                description: resource.description.clone(),
                mime_type: resource.mime_type.clone(),
            })
            .collect();

        Ok(resources)
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>> {
        if uri.is_empty() {
            return Err(InspectorError::Internal(anyhow::anyhow!(
                "URI cannot be empty"
            )));
        }

        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        let result = service
            .read_resource(ReadResourceRequestParam {
                uri: uri.to_string(),
            })
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context(format!("Failed to read resource: {}", uri))?;

        let contents = result
            .contents
            .into_iter()
            .map(|content| match content {
                rmcp::model::ResourceContents::TextResourceContents {
                    uri,
                    mime_type,
                    text,
                    meta: _,
                } => ResourceContent {
                    uri,
                    mime_type,
                    text: Some(text),
                    blob: None,
                },
                rmcp::model::ResourceContents::BlobResourceContents {
                    uri,
                    mime_type,
                    blob,
                    meta: _,
                } => ResourceContent {
                    uri,
                    mime_type,
                    text: None,
                    blob: Some(blob),
                },
            })
            .collect();

        Ok(contents)
    }

    async fn list_prompts(&self) -> Result<Vec<PromptInfo>> {
        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        let prompts_response = service
            .list_prompts(Default::default())
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context("Failed to list prompts")?;

        let prompts = prompts_response
            .prompts
            .into_iter()
            .map(|prompt| {
                let arguments = prompt
                    .arguments
                    .unwrap_or_default()
                    .into_iter()
                    .map(|arg| PromptArgument {
                        name: arg.name,
                        description: arg.description,
                        required: arg.required.unwrap_or(false),
                    })
                    .collect();

                PromptInfo {
                    name: prompt.name,
                    description: prompt.description,
                    arguments,
                }
            })
            .collect();

        Ok(prompts)
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<Vec<PromptMessage>> {
        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        // Convert HashMap<String, String> to Option<serde_json::Map<String, Value>>
        let arguments_map = if arguments.is_empty() {
            None
        } else {
            let map: serde_json::Map<String, serde_json::Value> = arguments
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            Some(map)
        };

        let result = service
            .get_prompt(GetPromptRequestParam {
                name: name.to_string(),
                arguments: arguments_map,
            })
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context(format!("Failed to get prompt: {}", name))?;

        let messages = result
            .messages
            .into_iter()
            .map(|msg| {
                // Convert rmcp::model::PromptMessage to our PromptMessage
                let role = match msg.role {
                    rmcp::model::PromptMessageRole::User => "user".to_string(),
                    rmcp::model::PromptMessageRole::Assistant => "assistant".to_string(),
                };

                // Convert content to serde_json::Value
                let content = match serde_json::to_value(&msg.content) {
                    Ok(value) => value,
                    Err(_) => serde_json::Value::Null,
                };

                PromptMessage { role, content }
            })
            .collect();

        Ok(messages)
    }
}

// Implement McpClient for Arc<StdioClient> to support connection pooling
#[async_trait]
impl McpClient for Arc<StdioClient> {
    async fn is_connected(&self) -> bool {
        (**self).is_connected().await
    }

    async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        (**self).list_tools().await
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        (**self).call_tool(name, arguments).await
    }

    async fn list_resources(&self) -> Result<Vec<ResourceInfo>> {
        (**self).list_resources().await
    }

    async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>> {
        (**self).read_resource(uri).await
    }

    async fn list_prompts(&self) -> Result<Vec<PromptInfo>> {
        (**self).list_prompts().await
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<Vec<PromptMessage>> {
        (**self).get_prompt(name, arguments).await
    }

    async fn disconnect(&mut self) -> Result<()> {
        // For Arc, we can't mutate the inner value directly
        // Connection cleanup will happen when the Arc is dropped
        Ok(())
    }
}
