/// Example demonstrating PersistentLogger usage
///
/// Run with: cargo run --example persistent_logger_demo
use anyhow::Result;
use mcp_inspector_mcp::models::{
    SamplingContent, SamplingLogEntry, SamplingMessage, SamplingStatus,
};
use mcp_inspector_mcp::services::{LoggerBackend, PersistentLogger};

fn main() -> Result<()> {
    println!("=== PersistentLogger Demo ===\n");

    // Create a persistent logger with database in temp directory
    let db_path = std::env::temp_dir().join("mcp_inspector_demo.db");
    println!("Database path: {:?}\n", db_path);

    let logger = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;

    // Create some test log entries
    println!("Adding 3 log entries...");

    let entry1 = SamplingLogEntry {
        id: "test_server:2025-01-15T12:00:00Z".to_string(),
        timestamp: "2025-01-15T12:00:00Z".to_string(),
        status: SamplingStatus::Success,
        messages: vec![SamplingMessage {
            role: "user".to_string(),
            content: SamplingContent {
                content_type: "text".to_string(),
                text: Some("Hello, world!".to_string()),
            },
        }],
        model_preferences: None,
        system_prompt: Some("You are a helpful assistant".to_string()),
        max_tokens: Some(100),
        error: None,
        response: Some("Hello! How can I help you?".to_string()),
    };

    let entry2 = SamplingLogEntry {
        id: "test_server:2025-01-15T12:01:00Z".to_string(),
        timestamp: "2025-01-15T12:01:00Z".to_string(),
        status: SamplingStatus::Failed,
        messages: vec![SamplingMessage {
            role: "user".to_string(),
            content: SamplingContent {
                content_type: "text".to_string(),
                text: Some("Invalid request".to_string()),
            },
        }],
        model_preferences: None,
        system_prompt: None,
        max_tokens: Some(50),
        error: Some("Connection timeout".to_string()),
        response: None,
    };

    let entry3 = SamplingLogEntry {
        id: "other_server:2025-01-15T12:02:00Z".to_string(),
        timestamp: "2025-01-15T12:02:00Z".to_string(),
        status: SamplingStatus::Success,
        messages: vec![SamplingMessage {
            role: "user".to_string(),
            content: SamplingContent {
                content_type: "text".to_string(),
                text: Some("Test from other server".to_string()),
            },
        }],
        model_preferences: None,
        system_prompt: None,
        max_tokens: Some(200),
        error: None,
        response: Some("Response from other server".to_string()),
    };

    logger.add_log(entry1)?;
    logger.add_log(entry2)?;
    logger.add_log(entry3)?;
    println!("✓ Successfully added 3 logs\n");

    // Retrieve and display logs
    println!("=== All logs for 'test_server' ===");
    let logs = logger.get_logs("test_server", 10, "all")?;
    for (i, log) in logs.iter().enumerate() {
        println!(
            "{}. [{}] {} - Status: {:?}",
            i + 1,
            log.timestamp,
            log.id,
            log.status
        );
    }
    println!();

    println!("=== Success logs for 'test_server' ===");
    let success_logs = logger.get_logs("test_server", 10, "success")?;
    println!("Found {} success log(s)", success_logs.len());
    println!();

    println!("=== Failed logs for 'test_server' ===");
    let failed_logs = logger.get_logs("test_server", 10, "failed")?;
    println!("Found {} failed log(s)", failed_logs.len());
    println!();

    // Count logs
    println!("=== Log counts ===");
    println!("test_server: {} logs", logger.count_logs("test_server")?);
    println!("other_server: {} logs", logger.count_logs("other_server")?);
    println!();

    // Drop the first logger to ensure all data is flushed
    drop(logger);

    // Test persistence by creating new logger instance
    println!("=== Testing persistence ===");
    println!("Creating new logger instance with same database...");
    let logger2 = PersistentLogger::new(db_path.to_str().unwrap(), 100)?;
    let persisted_logs = logger2.get_logs("test_server", 10, "all")?;
    println!("✓ Found {} persisted logs", persisted_logs.len());
    assert_eq!(persisted_logs.len(), 2, "Expected 2 logs to be persisted");
    println!();

    // Clean up
    println!("=== Cleanup ===");
    logger2.clear_logs("test_server")?;
    logger2.clear_logs("other_server")?;
    println!("✓ Cleared all logs");

    println!("\nDemo completed successfully!");

    Ok(())
}
