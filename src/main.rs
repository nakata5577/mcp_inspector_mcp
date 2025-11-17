use anyhow::{Context, Result};
use mcp_inspector_mcp::{
    models::{InspectorError, LoggingBackend, LoggingConfig, ServerConfig},
    run_server, InspectorConfig, InspectorService,
};
use mcp_inspector_mcp::services::config_manager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    // IMPORTANT: Disable ANSI color codes to prevent JSON parse errors in MCP clients
    // MCP protocol requires clean JSON on stdout, so we:
    // 1. Disable ANSI colors completely (.with_ansi(false))
    // 2. Write logs to stderr (.with_writer(std::io::stderr))
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_ansi(false)  // Disable ANSI color codes
        .with_writer(std::io::stderr)  // Write to stderr, not stdout
        .init();

    tracing::info!("MCP Inspector Server starting...");

    // Load configuration from .inspector/config.json
    let project_config = config_manager::load_config()
        .context("Failed to load configuration from .inspector/config.json")?;

    // Convert ProjectConfig to InspectorConfig
    let servers = convert_server_entries(&project_config.servers)?;
    let logging = convert_logging_settings(&project_config.logging);
    let execution_config = project_config.execution_config.clone();

    let config = InspectorConfig {
        servers,
        logging,
        execution_config,
    };

    tracing::info!("Loaded {} server configuration(s)", config.servers.len());
    tracing::info!(
        "Logging backend: {:?} (max_logs: {})",
        config.logging.backend,
        config.logging.max_logs
    );
    if config.logging.db_path.is_some() {
        tracing::info!("Database path: [CONFIGURED]");
    }
    tracing::info!(
        "Execution config: tool_timeout={}ms, connection_timeout={}ms, retry={}, auto_retry={}",
        config.execution_config.tool_timeout_ms,
        config.execution_config.connection_timeout_ms,
        config.execution_config.retry_count,
        config.execution_config.auto_retry_on_timeout
    );

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

/// Convert ServerEntry to ServerConfig
fn convert_server_entries(
    entries: &[mcp_inspector_mcp::models::ServerEntry],
) -> Result<Vec<ServerConfig>, InspectorError> {
    entries
        .iter()
        .map(|entry| {
            let transport = match entry.transport.as_str() {
                "stdio" => mcp_inspector_mcp::models::TransportType::Stdio,
                _ => {
                    return Err(InspectorError::Config(format!(
                        "Unsupported transport type: {}",
                        entry.transport
                    )))
                }
            };

            Ok(ServerConfig {
                name: entry.name.clone(),
                transport,
                params: mcp_inspector_mcp::models::ConnectionParams {
                    command: entry.command.clone(),
                    args: entry.args.clone(),
                    env: entry.env.clone(),
                },
            })
        })
        .collect()
}

/// Convert LoggingSettings to LoggingConfig
fn convert_logging_settings(
    settings: &mcp_inspector_mcp::models::LoggingSettings,
) -> LoggingConfig {
    let backend = match settings.backend.as_str() {
        "persistent" => LoggingBackend::Persistent,
        _ => LoggingBackend::Memory,
    };

    let db_path = if backend == LoggingBackend::Persistent {
        Some(settings.db_path.clone())
    } else {
        None
    };

    LoggingConfig {
        backend,
        db_path,
        max_logs: settings.max_logs,
    }
}
