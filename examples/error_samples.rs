use mcp_inspector_mcp::error::{ToolExecutionError, ErrorResponse};

fn main() {
    println!("=== Error Message Samples ===\n");

    // 1. Timeout Error
    println!("1. Timeout Error:");
    println!("----------------------------------------");
    let timeout_error = ToolExecutionError::Timeout {
        tool_name: "long_running_query".to_string(),
        elapsed_ms: 35000,
        configured_timeout_ms: 30000,
        server_alive: true,
        suggestion: Some("Try increasing timeout with environment variable: MCP_TOOL_TIMEOUT_MS=60000".to_string()),
    };
    println!("User Message:");
    println!("{}", timeout_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&timeout_error.to_json()).unwrap());
    println!("\n");

    // 2. Server Crash Error
    println!("2. Server Crash Error:");
    println!("----------------------------------------");
    let crash_error = ToolExecutionError::ServerCrash {
        tool_name: "database_backup".to_string(),
        exit_code: Some(137),
        stderr: "Out of memory: Kill process 1234 (mcp-server)".to_string(),
        last_log: Some("Starting backup operation...".to_string()),
    };
    println!("User Message:");
    println!("{}", crash_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&crash_error.to_json()).unwrap());
    println!("\n");

    // 3. Invalid Response Error
    println!("3. Invalid Response Error:");
    println!("----------------------------------------");
    let invalid_response_error = ToolExecutionError::InvalidResponse {
        tool_name: "get_user_data".to_string(),
        received: "{name: 'John', age: 30}".to_string(),
        expected_format: "Valid JSON response".to_string(),
        parse_error: "Expected \" at position 1".to_string(),
    };
    println!("User Message:");
    println!("{}", invalid_response_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&invalid_response_error.to_json()).unwrap());
    println!("\n");

    // 4. Communication Error
    println!("4. Communication Error:");
    println!("----------------------------------------");
    let comm_error = ToolExecutionError::CommunicationError {
        tool_name: "fetch_remote_data".to_string(),
        details: "Connection reset by peer (ECONNRESET)".to_string(),
        suggestion: Some("Check if the server process is still running and responsive".to_string()),
    };
    println!("User Message:");
    println!("{}", comm_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&comm_error.to_json()).unwrap());
    println!("\n");

    // 5. Server Error
    println!("5. Server Error:");
    println!("----------------------------------------");
    let server_error = ToolExecutionError::ServerError {
        tool_name: "process_transaction".to_string(),
        error_message: "Transaction failed: Insufficient funds".to_string(),
        error_code: Some(4001),
    };
    println!("User Message:");
    println!("{}", server_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&server_error.to_json()).unwrap());
    println!("\n");

    // 6. Other Error
    println!("6. Other Error:");
    println!("----------------------------------------");
    let other_error = ToolExecutionError::Other {
        tool_name: "unknown_operation".to_string(),
        message: "An unexpected error occurred during operation".to_string(),
    };
    println!("User Message:");
    println!("{}", other_error.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&other_error.to_json()).unwrap());
    println!("\n");

    // 7. ErrorResponse with request ID
    println!("7. Complete ErrorResponse with Request ID:");
    println!("----------------------------------------");
    let error_response = ErrorResponse::new(ToolExecutionError::Timeout {
        tool_name: "api_call".to_string(),
        elapsed_ms: 45000,
        configured_timeout_ms: 30000,
        server_alive: false,
        suggestion: None,
    })
    .with_request_id("req-abc123def456".to_string());

    println!("Full ErrorResponse JSON:");
    println!("{}", serde_json::to_string_pretty(&error_response).unwrap());
    println!("\n");

    // 8. Timeout with dead server
    println!("8. Timeout with Dead Server:");
    println!("----------------------------------------");
    let dead_server_timeout = ToolExecutionError::Timeout {
        tool_name: "critical_operation".to_string(),
        elapsed_ms: 30500,
        configured_timeout_ms: 30000,
        server_alive: false,
        suggestion: Some("The server process has terminated. Check server logs for more details.".to_string()),
    };
    println!("User Message:");
    println!("{}", dead_server_timeout.user_message());
    println!("\nJSON Format:");
    println!("{}", serde_json::to_string_pretty(&dead_server_timeout.to_json()).unwrap());
}
