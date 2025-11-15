use anyhow::{Context, Result};
use clap::Parser;
use mcp_inspector_mcp::{
    models::{InspectorError, LoggingBackend, LoggingConfig, ServerConfig},
    run_server, InspectorConfig, InspectorService,
};
use std::env;

/// MCP Inspector MCP Server
///
/// A Model Context Protocol (MCP) server that provides tools for inspecting and interacting
/// with other MCP servers. Supports configuration via CLI arguments or environment variables.
#[derive(Parser)]
#[command(name = "mcp_inspector_mcp")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server configuration in DSL format (name:transport:command[:args...])
    ///
    /// Can be specified multiple times to configure multiple servers.
    ///
    /// Examples:
    ///   --server "fa:stdio:C:/path/to/fa.exe"
    ///   --server "ta:stdio:/path/to/ta.exe:--verbose"
    #[arg(short, long = "server", value_name = "DSL")]
    servers: Vec<String>,

    /// Logging backend (memory or persistent)
    ///
    /// - memory: Store logs in memory (default)
    /// - persistent: Store logs in SQLite database
    #[arg(long, value_name = "TYPE")]
    log_backend: Option<String>,

    /// Database path for persistent logging
    ///
    /// Only used when log_backend is "persistent".
    /// Defaults to "./data/logs.db"
    #[arg(long, value_name = "PATH")]
    log_path: Option<String>,

    /// Maximum number of logs per server
    ///
    /// Defaults to 10000
    #[arg(long, value_name = "NUM")]
    log_max_logs: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("MCP Inspector Server starting...");

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration with priority: CLI args > environment variables
    let servers = load_server_configs(&cli)?;
    let logging = load_logging_config(&cli);

    let config = InspectorConfig { servers, logging };

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

/// Load server configurations with priority: CLI args > environment variables
///
/// # Priority
/// 1. CLI arguments (--server)
/// 2. Environment variable (MCP_INSPECTOR_SERVERS)
///
/// # Errors
/// Returns an error if:
/// - No configuration source is available
/// - Configuration parsing fails
/// - Configuration is invalid (empty, missing required fields, etc.)
fn load_server_configs(cli: &Cli) -> Result<Vec<ServerConfig>, InspectorError> {
    // Priority 1: CLI arguments
    if !cli.servers.is_empty() {
        tracing::info!(
            "Loading server configurations from CLI arguments ({} server(s))",
            cli.servers.len()
        );
        let configs: Result<Vec<_>, _> = cli
            .servers
            .iter()
            .map(|dsl| ServerConfig::from_dsl(dsl))
            .collect();
        return configs;
    }

    // Priority 2: Environment variables (fallback for backward compatibility)
    if let Ok(json_str) = env::var("MCP_INSPECTOR_SERVERS") {
        tracing::info!("Loading server configurations from environment variables");
        let servers: Vec<ServerConfig> = serde_json::from_str(&json_str).map_err(|e| {
            InspectorError::Config(format!(
                "Failed to parse MCP_INSPECTOR_SERVERS as JSON: {}",
                e
            ))
        })?;

        // Validate non-empty
        if servers.is_empty() {
            return Err(InspectorError::Config(
                "MCP_INSPECTOR_SERVERS must contain at least one server configuration".to_string(),
            ));
        }

        return Ok(servers);
    }

    Err(InspectorError::Config(
        "No server configuration provided. Use --server or MCP_INSPECTOR_SERVERS env var"
            .to_string(),
    ))
}

/// Load logging configuration with priority: CLI args > environment variables > defaults
///
/// # Priority
/// 1. CLI arguments (--log-backend, --log-path, --log-max-logs)
/// 2. Environment variables (MCP_LOGGING_BACKEND, MCP_LOGGING_DB_PATH, MCP_LOGGING_MAX_LOGS)
/// 3. Default values (backend: memory, max_logs: 10000, db_path: ./data/logs.db)
fn load_logging_config(cli: &Cli) -> LoggingConfig {
    // Backend: CLI > env > default
    let backend_str = cli
        .log_backend
        .clone()
        .or_else(|| env::var("MCP_LOGGING_BACKEND").ok())
        .unwrap_or_else(|| "memory".to_string());

    let backend = match backend_str.to_lowercase().as_str() {
        "persistent" => LoggingBackend::Persistent,
        _ => LoggingBackend::Memory,
    };

    // Database path: CLI > env > default
    let db_path = if backend == LoggingBackend::Persistent {
        Some(
            cli.log_path
                .clone()
                .or_else(|| env::var("MCP_LOGGING_DB_PATH").ok())
                .unwrap_or_else(|| "./data/logs.db".to_string()),
        )
    } else {
        None
    };

    // Max logs: CLI > env > default
    let max_logs = cli
        .log_max_logs
        .or_else(|| {
            env::var("MCP_LOGGING_MAX_LOGS")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(10000);

    LoggingConfig {
        backend,
        db_path,
        max_logs,
    }
}
