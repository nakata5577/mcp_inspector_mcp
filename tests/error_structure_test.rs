use mcp_inspector_mcp::error::{ToolExecutionError, ErrorResponse};

#[test]
fn test_timeout_error_creation() {
    let error = ToolExecutionError::Timeout {
        tool_name: "test_tool".to_string(),
        elapsed_ms: 35000,
        configured_timeout_ms: 30000,
        server_alive: true,
        suggestion: Some("Try increasing timeout with environment variable: MCP_TOOL_TIMEOUT_MS=60000".to_string()),
    };

    // Test user message
    let message = error.user_message();
    assert!(message.contains("test_tool"));
    assert!(message.contains("35000ms"));
    assert!(message.contains("30000ms"));
    assert!(message.contains("Suggestion"));

    // Test JSON serialization
    let json = error.to_json();
    assert_eq!(json["type"], "Timeout");
    assert_eq!(json["tool_name"], "test_tool");
    assert_eq!(json["elapsed_ms"], 35000);
    assert_eq!(json["configured_timeout_ms"], 30000);
    assert_eq!(json["server_alive"], true);
}

#[test]
fn test_server_crash_error() {
    let error = ToolExecutionError::ServerCrash {
        tool_name: "failing_tool".to_string(),
        exit_code: Some(1),
        stderr: "Connection refused".to_string(),
        last_log: Some("Last message before crash".to_string()),
    };

    let message = error.user_message();
    assert!(message.contains("failing_tool"));
    assert!(message.contains("Exit code: Some(1)"));
    assert!(message.contains("Connection refused"));

    let json = error.to_json();
    assert_eq!(json["type"], "ServerCrash");
}

#[test]
fn test_invalid_response_error() {
    let error = ToolExecutionError::InvalidResponse {
        tool_name: "bad_tool".to_string(),
        received: "{invalid json}".to_string(),
        expected_format: "Valid JSON response".to_string(),
        parse_error: "Unexpected token at position 1".to_string(),
    };

    let message = error.user_message();
    assert!(message.contains("bad_tool"));
    assert!(message.contains("Unexpected token"));

    let json = error.to_json();
    assert_eq!(json["type"], "InvalidResponse");
}

#[test]
fn test_communication_error() {
    let error = ToolExecutionError::CommunicationError {
        tool_name: "network_tool".to_string(),
        details: "Connection reset by peer".to_string(),
        suggestion: Some("Check if the server process is still running".to_string()),
    };

    let message = error.user_message();
    assert!(message.contains("network_tool"));
    assert!(message.contains("Connection reset by peer"));
    assert!(message.contains("Suggestion"));

    let json = error.to_json();
    assert_eq!(json["type"], "CommunicationError");
}

#[test]
fn test_server_error() {
    let error = ToolExecutionError::ServerError {
        tool_name: "server_tool".to_string(),
        error_message: "Internal server error".to_string(),
        error_code: Some(500),
    };

    let message = error.user_message();
    assert!(message.contains("server_tool"));
    assert!(message.contains("Internal server error"));

    let json = error.to_json();
    assert_eq!(json["type"], "ServerError");
    assert_eq!(json["error_code"], 500);
}

#[test]
fn test_error_response_creation() {
    let error = ToolExecutionError::Other {
        tool_name: "some_tool".to_string(),
        message: "Something went wrong".to_string(),
    };

    let response = ErrorResponse::new(error);
    assert!(!response.timestamp.is_empty());
    assert!(response.request_id.is_none());

    let response_with_id = ErrorResponse::new(ToolExecutionError::Other {
        tool_name: "some_tool".to_string(),
        message: "Something went wrong".to_string(),
    })
    .with_request_id("req-123".to_string());

    assert_eq!(response_with_id.request_id, Some("req-123".to_string()));
}

#[test]
fn test_timeout_without_suggestion() {
    let error = ToolExecutionError::Timeout {
        tool_name: "quick_tool".to_string(),
        elapsed_ms: 1000,
        configured_timeout_ms: 5000,
        server_alive: false,
        suggestion: None,
    };

    let message = error.user_message();
    assert!(message.contains("quick_tool"));
    assert!(message.contains("Server process has terminated unexpectedly"));
    assert!(!message.contains("Suggestion"));

    let json = error.to_json();
    // suggestion should be omitted when None
    assert!(json.get("suggestion").is_none());
}

#[test]
fn test_display_trait() {
    let error = ToolExecutionError::Other {
        tool_name: "display_tool".to_string(),
        message: "Test display".to_string(),
    };

    let display_output = format!("{}", error);
    assert!(display_output.contains("display_tool"));
    assert!(display_output.contains("Test display"));
}
