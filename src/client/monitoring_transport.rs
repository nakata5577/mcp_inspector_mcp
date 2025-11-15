use anyhow::Result;
use rmcp::service::{RoleClient, RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::transport::Transport;
use std::sync::Arc;

/// Transport wrapper that monitors messages for sampling requests and logging messages
///
/// This transport intercepts outgoing messages and logs:
/// - sampling/createMessage notifications sent from the target server to the LLM
/// - notifications/message log messages sent from the target server
pub struct MonitoringTransport<T: Transport<RoleClient>> {
    /// Inner transport handling actual communication
    inner: T,
    /// Logger for recording sampling requests
    sampling_logger: Arc<crate::services::SamplingLogger>,
    /// Inspector for recording logging messages (optional)
    logging_inspector: Option<Arc<crate::services::LoggingInspector>>,
    /// Target server name for log identification
    server_name: String,
}

impl<T: Transport<RoleClient>> MonitoringTransport<T> {
    /// Create a new MonitoringTransport wrapping the given transport
    pub fn new(
        inner: T,
        sampling_logger: Arc<crate::services::SamplingLogger>,
        server_name: String,
    ) -> Self {
        Self {
            inner,
            sampling_logger,
            logging_inspector: None,
            server_name,
        }
    }

    /// Set the logging inspector for monitoring logging messages
    pub fn with_logging_inspector(
        mut self,
        logging_inspector: Arc<crate::services::LoggingInspector>,
    ) -> Self {
        self.logging_inspector = Some(logging_inspector);
        self
    }

    /// Check if a JSON message is a sampling/createMessage notification
    fn is_sampling_notification(json_value: &serde_json::Value) -> bool {
        json_value
            .get("method")
            .and_then(|m| m.as_str())
            .map(|method| method == "sampling/createMessage")
            .unwrap_or(false)
    }

    /// Check if a JSON message is a notifications/message (logging notification)
    fn is_logging_notification(json_value: &serde_json::Value) -> bool {
        json_value
            .get("method")
            .and_then(|m| m.as_str())
            .map(|method| method == "notifications/message")
            .unwrap_or(false)
    }

    /// Extract and log a logging message notification
    fn extract_and_log_message(
        logging_inspector: &Arc<crate::services::LoggingInspector>,
        server_name: &str,
        json_value: &serde_json::Value,
    ) {
        // Extract params field
        let params = json_value
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Try to deserialize as LoggingMessageNotificationParam
        match serde_json::from_value::<rmcp::model::LoggingMessageNotificationParam>(params.clone())
        {
            Ok(notification) => {
                let entry = crate::models::LogEntry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    server_name: server_name.to_string(),
                    level: crate::models::LogLevel::from(notification.level),
                    logger: notification.logger,
                    message: notification.data.to_string(),
                };

                if let Err(e) = logging_inspector.add_log_entry(entry) {
                    tracing::warn!(
                        server = %server_name,
                        error = ?e,
                        "Failed to store log message entry"
                    );
                }
            }
            Err(e) => {
                tracing::debug!(
                    server = %server_name,
                    error = ?e,
                    params = ?params,
                    "Failed to parse logging notification params"
                );
            }
        }
    }

    /// Helper function to extract sampling params and log them
    fn extract_and_log_sampling(
        sampling_logger: &Arc<crate::services::SamplingLogger>,
        server_name: &str,
        json_value: &serde_json::Value,
    ) {
        // Extract params field
        let params = json_value
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Build log entry from JSON
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Extract fields from params
        let messages = params
            .get("messages")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|msg| {
                        Some(crate::models::SamplingMessage {
                            role: msg.get("role")?.as_str()?.to_string(),
                            content: crate::models::SamplingContent {
                                content_type: msg
                                    .get("content")?
                                    .get("type")?
                                    .as_str()?
                                    .to_string(),
                                text: msg
                                    .get("content")
                                    .and_then(|c| c.get("text"))
                                    .and_then(|t| t.as_str())
                                    .map(|s| s.to_string()),
                            },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let max_tokens = params
            .get("maxTokens")
            .and_then(|t| t.as_u64())
            .map(|t| t as u32);

        let system_prompt = params
            .get("systemPrompt")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());

        // Extract model preferences
        let model_preferences =
            params
                .get("modelPreferences")
                .map(|prefs| crate::models::ModelPreferences {
                    hints: prefs.get("hints").and_then(|h| h.as_array()).map(|hints| {
                        hints
                            .iter()
                            .map(|hint| crate::models::ModelHint {
                                name: hint
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .map(|s| s.to_string()),
                            })
                            .collect()
                    }),
                    cost_priority: prefs.get("costPriority").and_then(|p| p.as_f64()),
                    speed_priority: prefs.get("speedPriority").and_then(|p| p.as_f64()),
                    intelligence_priority: prefs
                        .get("intelligencePriority")
                        .and_then(|p| p.as_f64()),
                });

        let entry = crate::models::SamplingLogEntry {
            id: format!("{}:{}", server_name, timestamp),
            timestamp: timestamp.clone(),
            status: crate::models::SamplingStatus::Pending,
            messages,
            model_preferences,
            system_prompt,
            max_tokens,
            error: None,
            response: None,
        };

        sampling_logger.add_log(entry);
    }
}

impl<T: Transport<RoleClient> + Send> Transport<RoleClient> for MonitoringTransport<T> {
    type Error = T::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send + 'static {
        // Serialize the message to JSON for inspection
        // Note: This is a monitoring operation, so we clone and serialize
        let json_result = serde_json::to_value(&item);

        // Clone logger and server name for the async block
        let sampling_logger = self.sampling_logger.clone();
        let logging_inspector = self.logging_inspector.clone();
        let server_name = self.server_name.clone();

        // Send the message through inner transport
        let send_future = self.inner.send(item);

        async move {
            // Check if this is a sampling or logging notification
            if let Ok(json_value) = json_result {
                // Check for sampling notification
                if MonitoringTransport::<T>::is_sampling_notification(&json_value) {
                    MonitoringTransport::<T>::extract_and_log_sampling(
                        &sampling_logger,
                        &server_name,
                        &json_value,
                    );
                }

                // Check for logging notification
                if MonitoringTransport::<T>::is_logging_notification(&json_value) {
                    if let Some(ref inspector) = logging_inspector {
                        MonitoringTransport::<T>::extract_and_log_message(
                            inspector,
                            &server_name,
                            &json_value,
                        );
                    }
                }
            }

            // Execute actual send
            send_future.await
        }
    }

    fn receive(
        &mut self,
    ) -> impl std::future::Future<Output = Option<RxJsonRpcMessage<RoleClient>>> + Send {
        // Simply forward receive calls to inner transport
        // Future extension: could monitor responses here
        self.inner.receive()
    }

    fn close(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        // Forward close to inner transport
        self.inner.close()
    }
}
