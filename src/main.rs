use anyhow::{Context, Result};
use mcp_inspector_mcp::{run_server, InspectorConfig, InspectorService};
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!("MCP Inspector Server starting...");

    // Load configuration
    let config_path =
        env::var("MCP_INSPECTOR_CONFIG").unwrap_or_else(|_| "config/servers.toml".to_string());

    tracing::info!("Loading configuration from: {}", config_path);

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read configuration file: {}", config_path))?;

    let config: InspectorConfig = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse configuration file: {}", config_path))?;

    tracing::info!("Loaded {} server configuration(s)", config.servers.len());

    // Log logging backend configuration
    tracing::info!(
        "Logging backend: {:?} (max_logs: {})",
        config.logging.backend,
        config.logging.max_logs
    );
    if let Some(ref db_path) = config.logging.db_path {
        tracing::info!("Database path: {}", db_path);
    }

    // Initialize inspector service
    let inspector =
        InspectorService::new(config).context("Failed to initialize InspectorService")?;

    // Log configured servers
    for server_name in inspector.list_servers() {
        tracing::info!("  - {}", server_name);
    }

    // Run MCP server
    tracing::info!("MCP Inspector Server ready");
    run_server(inspector).await?;

    Ok(())
}
