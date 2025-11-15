/// Phase 4 Sampling Integration Test
///
/// このテストは、MonitoringTransportとSamplingLoggerの統合をテストします。
/// モックサーバーを使用せず、MonitoringTransportが正しくSamplingメッセージを検出し、
/// ログに記録することを確認します。
use mcp_inspector_mcp::client::MonitoringTransport;
use mcp_inspector_mcp::services::{MemoryLogger, SamplingLogger};
use rmcp::service::{RoleClient, RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::transport::Transport;
use serde_json::json;
use std::sync::Arc;

/// Mock error type that implements std::error::Error
#[derive(Debug)]
struct MockError;

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mock transport error")
    }
}

impl std::error::Error for MockError {}

/// Mock Transport for testing
struct MockTransport {
    sent_messages: Vec<TxJsonRpcMessage<RoleClient>>,
    received_messages: Vec<RxJsonRpcMessage<RoleClient>>,
}

impl MockTransport {
    fn new() -> Self {
        Self {
            sent_messages: Vec::new(),
            received_messages: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn with_received_messages(received: Vec<RxJsonRpcMessage<RoleClient>>) -> Self {
        Self {
            sent_messages: Vec::new(),
            received_messages: received,
        }
    }
}

impl Transport<RoleClient> for MockTransport {
    type Error = MockError;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send + 'static {
        self.sent_messages.push(item);
        async { Ok(()) }
    }

    fn receive(
        &mut self,
    ) -> impl std::future::Future<Output = Option<RxJsonRpcMessage<RoleClient>>> + Send {
        let result = if self.received_messages.is_empty() {
            None
        } else {
            Some(self.received_messages.remove(0))
        };
        async move { result }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn test_monitoring_transport_detects_sampling_notification() {
    // Setup
    let backend = Arc::new(MemoryLogger::new(1000));
    let sampling_logger = Arc::new(SamplingLogger::new(backend));
    let mock_transport = MockTransport::new();
    let mut monitoring_transport = MonitoringTransport::new(
        mock_transport,
        Arc::clone(&sampling_logger),
        "test_server".to_string(),
    );

    // Create a sampling/createMessage notification
    let sampling_notification = json!({
        "jsonrpc": "2.0",
        "method": "sampling/createMessage",
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "Test message for sampling"
                    }
                }
            ],
            "modelPreferences": {
                "hints": [
                    {
                        "name": "claude-3-5-sonnet"
                    }
                ]
            },
            "systemPrompt": "You are a test assistant",
            "maxTokens": 100
        }
    });

    // Convert JSON to TxJsonRpcMessage
    let message: TxJsonRpcMessage<RoleClient> =
        serde_json::from_value(sampling_notification).expect("Failed to create message");

    // Send the message through MonitoringTransport
    monitoring_transport
        .send(message)
        .await
        .expect("Send failed");

    // Give logger time to process (synchronous operation, but adding small delay for safety)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify that the sampling logger recorded the message
    let logs = sampling_logger.get_logs("test_server", 100, "all");
    assert_eq!(logs.len(), 1, "Expected 1 log entry, found {}", logs.len());

    let log_entry = &logs[0];
    assert_eq!(log_entry.messages.len(), 1);
    assert_eq!(log_entry.messages[0].role, "user");
    assert_eq!(
        log_entry.messages[0].content.text.as_ref().unwrap(),
        "Test message for sampling"
    );
    assert!(log_entry.model_preferences.is_some());
    assert_eq!(
        log_entry.system_prompt.as_ref().unwrap(),
        "You are a test assistant"
    );
    assert_eq!(log_entry.max_tokens.unwrap(), 100);
}

#[tokio::test]
async fn test_monitoring_transport_ignores_non_sampling_messages() {
    // Setup
    let backend = Arc::new(MemoryLogger::new(1000));
    let sampling_logger = Arc::new(SamplingLogger::new(backend));
    let mock_transport = MockTransport::new();
    let mut monitoring_transport = MonitoringTransport::new(
        mock_transport,
        Arc::clone(&sampling_logger),
        "test_server".to_string(),
    );

    // Create a regular method call (not sampling)
    let regular_message = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    let message: TxJsonRpcMessage<RoleClient> =
        serde_json::from_value(regular_message).expect("Failed to create message");

    // Send the message through MonitoringTransport
    monitoring_transport
        .send(message)
        .await
        .expect("Send failed");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify that the sampling logger did NOT record the message
    let logs = sampling_logger.get_logs("test_server", 100, "all");
    assert_eq!(
        logs.len(),
        0,
        "Expected 0 log entries, found {}",
        logs.len()
    );
}

#[tokio::test]
async fn test_monitoring_transport_multiple_sampling_notifications() {
    // Setup
    let backend = Arc::new(MemoryLogger::new(1000));
    let sampling_logger = Arc::new(SamplingLogger::new(backend));
    let mock_transport = MockTransport::new();
    let mut monitoring_transport = MonitoringTransport::new(
        mock_transport,
        Arc::clone(&sampling_logger),
        "test_server".to_string(),
    );

    // Send 3 sampling notifications
    for i in 1..=3 {
        let sampling_notification = json!({
            "jsonrpc": "2.0",
            "method": "sampling/createMessage",
            "params": {
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Test message #{}", i)
                        }
                    }
                ],
                "maxTokens": 50 + i * 10
            }
        });

        let message: TxJsonRpcMessage<RoleClient> =
            serde_json::from_value(sampling_notification).expect("Failed to create message");

        monitoring_transport
            .send(message)
            .await
            .expect("Send failed");
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify all 3 were logged
    let logs = sampling_logger.get_logs("test_server", 100, "all");
    assert_eq!(
        logs.len(),
        3,
        "Expected 3 log entries, found {}",
        logs.len()
    );

    // Verify content (logs are returned in reverse chronological order, newest first)
    for (i, log_entry) in logs.iter().rev().enumerate() {
        let expected_text = format!("Test message #{}", i + 1);
        assert_eq!(
            log_entry.messages[0].content.text.as_ref().unwrap(),
            &expected_text
        );
        assert_eq!(log_entry.max_tokens.unwrap(), 50 + (i as u32 + 1) * 10);
    }
}

#[tokio::test]
async fn test_monitoring_transport_passthrough() {
    // Setup
    let backend = Arc::new(MemoryLogger::new(1000));
    let sampling_logger = Arc::new(SamplingLogger::new(backend));
    let mock_transport = MockTransport::new();
    let mut monitoring_transport = MonitoringTransport::new(
        mock_transport,
        Arc::clone(&sampling_logger),
        "test_server".to_string(),
    );

    // Send a message
    let message_json = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "test/method"
    });

    let message: TxJsonRpcMessage<RoleClient> =
        serde_json::from_value(message_json).expect("Failed to create message");

    // Send should succeed (passthrough to mock transport)
    monitoring_transport
        .send(message)
        .await
        .expect("Send should succeed");
}
