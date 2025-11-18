use crate::client::{McpClient, MonitoringTransport};
use crate::error::ToolExecutionError;
use crate::models::{
    debug_config, ExecutionConfig, InspectorError, PromptArgument, PromptInfo, PromptMessage,
    ResourceContent, ResourceInfo, Result, ServerConfig, ToolInfo,
};
use crate::services::{
    debug_logger::{DebugLogger, DebugLoggerConfig},
    timing_tracker::TimingTracker,
    SamplingLogger,
};
use anyhow::Context;
use async_trait::async_trait;
use chrono::Local;
use rmcp::model::{
    CallToolRequestParam, GetPromptRequestParam, ReadResourceRequestParam, ServerCapabilities,
};
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

/// MCP client using stdio transport
pub struct StdioClient {
    config: ServerConfig,
    execution_config: ExecutionConfig,
    service: Arc<Mutex<Option<RunningService<RoleClient, ()>>>>,
    sampling_logger: Arc<SamplingLogger>,
    capabilities: Arc<Mutex<Option<ServerCapabilities>>>,
    debug_logger: DebugLogger,
    timing_tracker: TimingTracker,
}

impl StdioClient {
    /// Create a new StdioClient from server configuration
    ///
    /// # Arguments
    /// * `config` - Server configuration
    /// * `execution_config` - Execution configuration (timeout, retry, etc.)
    /// * `sampling_logger` - Logger for monitoring
    pub fn new(
        config: ServerConfig,
        execution_config: ExecutionConfig,
        sampling_logger: Arc<SamplingLogger>,
    ) -> Self {
        // Create debug logger config (disable color output for MCP compatibility)
        let debug_logger_config = DebugLoggerConfig {
            color_output: false,
            max_payload_size: 4096,
            truncate_large_payloads: true,
        };

        Self {
            config,
            execution_config,
            service: Arc::new(Mutex::new(None)),
            sampling_logger,
            capabilities: Arc::new(Mutex::new(None)),
            debug_logger: DebugLogger::new(debug_logger_config),
            timing_tracker: TimingTracker::new(),
        }
    }

    /// Get the InitializeResult from the server
    ///
    /// This method returns the initialization result containing server
    /// capabilities, implementation info, and protocol version.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The client is not connected
    /// - Failed to retrieve initialization result
    pub async fn get_init_result(&self) -> Result<rmcp::model::InitializeResult> {
        self.connect().await?;
        let guard = self.service.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        // Get InitializeResult from peer_info
        let init_result = service
            .peer_info()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Peer info not available"),
            })?;

        Ok(init_result.clone())
    }

    /// Initialize connection to the MCP server if not already connected
    ///
    /// This method checks if a connection is already established and only
    /// creates a new connection if necessary. It is safe to call multiple times.
    ///
    /// # Errors
    /// Returns an error if the connection cannot be established
    pub async fn connect_if_needed(&self) -> Result<()> {
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

        // Store server capabilities for validation
        if let Some(peer_info) = guard.as_ref().and_then(|s| s.peer_info()) {
            let mut caps_guard = self.capabilities.lock().await;
            *caps_guard = Some(peer_info.capabilities.clone());
        }
        Ok(())
    }

    /// Initialize connection to the MCP server (internal helper)
    ///
    /// This is a convenience method that calls `connect_if_needed()`.
    /// It exists to maintain backward compatibility with existing code.
    async fn connect(&self) -> Result<()> {
        self.connect_if_needed().await
    }

    /// Get reference to the service, connecting if necessary
    async fn get_service(&self) -> Result<Arc<Mutex<Option<RunningService<RoleClient, ()>>>>> {
        self.connect().await?;
        Ok(Arc::clone(&self.service))
    }

    /// Ping the server to check connectivity
    ///
    /// This method sends a ping request to the server and waits for a response.
    /// It can be used to verify that the server is responsive and the connection is healthy.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The client is not connected
    /// - The ping request fails
    pub async fn ping(&self) -> Result<()> {
        self.connect().await?;
        let guard = self.service.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        // Send ping request
        service
            .send_request(rmcp::model::ClientRequest::PingRequest(
                rmcp::model::PingRequest {
                    method: rmcp::model::PingRequestMethod,
                    extensions: Default::default(),
                },
            ))
            .await
            .map_err(|e| InspectorError::Internal(e.into()))
            .context("Ping request failed")?;

        Ok(())
    }

    /// Check if the server process is still alive
    ///
    /// This method attempts to determine if the underlying server process
    /// is still running. This is useful for diagnosing timeout issues.
    ///
    /// # Implementation Notes
    /// Currently, this method performs a best-effort check by attempting
    /// to verify service availability. A more robust implementation would
    /// track the child process handle and check its status directly.
    ///
    /// # Returns
    /// - `true` if the service appears to be available
    /// - `false` if the service is definitely unavailable
    async fn is_server_process_alive(&self) -> bool {
        let guard = self.service.lock().await;

        // If service is not initialized, process is definitely not alive
        if guard.is_none() {
            tracing::debug!("Server process check: service not initialized");
            return false;
        }

        // Service is initialized, which indicates the process was alive at some point
        // A more robust check would involve:
        // 1. On Windows: Using WaitForSingleObject with WAIT_TIMEOUT to check process status
        // 2. On Unix: Using waitpid with WNOHANG to check process status
        //
        // However, rmcp's TokioChildProcess doesn't expose the underlying process handle,
        // so we rely on service availability as a proxy for process liveness.
        //
        // Future enhancement: Extend rmcp to expose process status or implement
        // process tracking separately in StdioClient.

        tracing::debug!("Server process check: service is initialized (assuming alive)");
        true
    }

    /// Get server capabilities
    ///
    /// Returns the server capabilities if available, or None if the
    /// client is not yet connected or capabilities were not provided.
    ///
    /// # Returns
    /// Optional reference to ServerCapabilities
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        let guard = self.capabilities.lock().await;
        guard.clone()
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
        // Use timeout from execution config
        let timeout_ms = self.execution_config.tool_timeout_ms;
        let timeout_duration = Duration::from_millis(timeout_ms);

        // Generate request ID for tracking
        let request_id = format!("tool-{}-{}", name, Local::now().timestamp_millis());

        // Verbose mode: Log request with DebugLogger and start timing
        if debug_config::is_verbose_mode() {
            let timestamp = Local::now();
            self.debug_logger.log_request(
                &self.config.name,
                &format!("call_tool/{}", name),
                &request_id,
                &arguments,
                timestamp,
            );
            self.timing_tracker.start_timer(&request_id);
        }

        let start_time = std::time::Instant::now();

        let service_arc = self.get_service().await?;
        let guard = service_arc.lock().await;

        let service = guard
            .as_ref()
            .ok_or_else(|| InspectorError::ConnectionFailed {
                server: self.config.name.clone(),
                source: anyhow::anyhow!("Service not initialized"),
            })?;

        // Convert arguments to Option<serde_json::Map>
        // This ensures that all JSON values (including empty objects, arrays, etc.) are properly handled
        let arguments_obj = match arguments {
            serde_json::Value::Object(map) => Some(map),
            serde_json::Value::Null => None,
            _ => {
                // For non-object values, wrap them in an object under "value" key
                // or convert to empty object if conversion is not meaningful
                let mut map = serde_json::Map::new();
                map.insert("value".to_string(), arguments);
                Some(map)
            }
        };

        // Execute tool call with timeout
        let tool_call_future = service.call_tool(CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments_obj,
        });

        let result = match timeout(timeout_duration, tool_call_future).await {
            Ok(Ok(response)) => {
                let elapsed = start_time.elapsed();
                tracing::info!(
                    "Tool '{}' completed successfully in {}ms",
                    name,
                    elapsed.as_millis()
                );

                // Verbose mode: Log response with DebugLogger
                if debug_config::is_verbose_mode() {
                    if let Some((elapsed_ms, end_time)) = self.timing_tracker.stop_timer(&request_id)
                    {
                        let response_value = serde_json::to_value(&response)
                            .unwrap_or(serde_json::Value::Null);
                        self.debug_logger.log_response(
                            &self.config.name,
                            &request_id,
                            &response_value,
                            end_time,
                            elapsed_ms,
                            true, // is_success
                        );
                    }
                }

                Ok(response)
            }
            Ok(Err(e)) => {
                let elapsed = start_time.elapsed();
                let error_string = e.to_string();

                // エラーの種類に応じて適切なToolExecutionErrorを生成
                let error = if error_string.contains("server terminated") || error_string.contains("process") {
                    ToolExecutionError::ServerCrash {
                        tool_name: name.to_string(),
                        exit_code: None,
                        stderr: error_string.clone(),
                        last_log: None,
                    }
                } else if error_string.contains("parse") || error_string.contains("invalid") || error_string.contains("deserialize") {
                    ToolExecutionError::InvalidResponse {
                        tool_name: name.to_string(),
                        received: error_string.clone(),
                        expected_format: "Valid JSON response".to_string(),
                        parse_error: error_string.clone(),
                    }
                } else if error_string.contains("connection") || error_string.contains("io") || error_string.contains("channel") {
                    ToolExecutionError::CommunicationError {
                        tool_name: name.to_string(),
                        details: error_string.clone(),
                        suggestion: Some("Check if the server process is still running and responsive".to_string()),
                    }
                } else {
                    ToolExecutionError::ServerError {
                        tool_name: name.to_string(),
                        error_message: error_string.clone(),
                        error_code: None,
                    }
                };

                tracing::error!(
                    "Tool '{}' failed after {}ms: {}",
                    name,
                    elapsed.as_millis(),
                    error.user_message()
                );
                tracing::error!("Structured error details: {}", serde_json::to_string_pretty(&error.to_json()).unwrap_or_else(|_| "Failed to serialize".to_string()));

                // Verbose mode: Log error response with DebugLogger
                if debug_config::is_verbose_mode() {
                    if let Some((elapsed_ms, end_time)) = self.timing_tracker.stop_timer(&request_id)
                    {
                        let error_response = serde_json::json!({
                            "error": error.user_message(),
                            "details": error.to_json(),
                        });
                        self.debug_logger.log_response(
                            &self.config.name,
                            &request_id,
                            &error_response,
                            end_time,
                            elapsed_ms,
                            false, // is_success
                        );
                    }
                }

                Err(InspectorError::ToolExecutionFailed {
                    server: self.config.name.clone(),
                    tool: name.to_string(),
                    source: anyhow::anyhow!("{}", error.user_message()),
                })
            }
            Err(_elapsed) => {
                // Timeout occurred
                let elapsed_ms = start_time.elapsed().as_millis() as u64;

                // Check if server process is still alive
                let server_alive = self.is_server_process_alive().await;

                let error = ToolExecutionError::Timeout {
                    tool_name: name.to_string(),
                    elapsed_ms,
                    configured_timeout_ms: timeout_ms,
                    server_alive,
                    suggestion: Some(format!(
                        "Try increasing timeout with environment variable: MCP_TOOL_TIMEOUT_MS={}",
                        timeout_ms * 2
                    )),
                };

                tracing::error!("{}", error.user_message());
                tracing::error!("Structured error details: {}", serde_json::to_string_pretty(&error.to_json()).unwrap_or_else(|_| "Failed to serialize".to_string()));

                // Verbose mode: Log timeout error with DebugLogger
                if debug_config::is_verbose_mode() {
                    if let Some((elapsed_ms_timing, end_time)) =
                        self.timing_tracker.stop_timer(&request_id)
                    {
                        let timeout_response = serde_json::json!({
                            "error": "Timeout",
                            "details": error.to_json(),
                        });
                        self.debug_logger.log_response(
                            &self.config.name,
                            &request_id,
                            &timeout_response,
                            end_time,
                            elapsed_ms_timing,
                            false, // is_success
                        );
                    }
                }

                if !server_alive {
                    Err(InspectorError::ConnectionFailed {
                        server: self.config.name.clone(),
                        source: anyhow::anyhow!("{}", error.user_message()),
                    })
                } else {
                    Err(InspectorError::ToolExecutionFailed {
                        server: self.config.name.clone(),
                        tool: name.to_string(),
                        source: anyhow::anyhow!("{}", error.user_message()),
                    })
                }
            }
        }?;

        let serialized = serde_json::to_value(result).context("Failed to serialize tool result")?;

        Ok(serialized)
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
        // Delegate to the inner implementation
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

    async fn capabilities(&self) -> Option<ServerCapabilities> {
        // Delegate to the inner implementation
        (**self).capabilities().await
    }
}
