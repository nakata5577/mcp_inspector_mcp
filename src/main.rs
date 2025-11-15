use anyhow::{Context, Result};
use mcp_inspector_mcp::{run_server, InspectorConfig, InspectorService};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("MCP Inspector Server starting...");

    // Load configuration from environment variables
    let config = InspectorConfig::from_env()
        .context("Failed to load configuration from environment variables")?;

    tracing::info!("Loaded {} server configuration(s)", config.servers.len());
    tracing::info!(
        "Logging backend: {:?} (max_logs: {})",
        config.logging.backend,
        config.logging.max_logs
    );
    if config.logging.db_path.is_some() {
        tracing::info!("Database path: [CONFIGURED]");
    }

    // Initialize inspector service
    let inspector = InspectorService::new(config)
        .context("Failed to initialize InspectorService")?;

    // Log configured servers
    for server_name in inspector.list_servers() {
        tracing::info!("  - {}", server_name);
    }

    // Run MCP server
    tracing::info!("MCP Inspector Server ready");
    run_server(inspector).await?;

    Ok(())
}
