use anyhow::{Context, Result};
use clap::Parser;
use mcp_inspector_mcp::{
    models::{debug_config, InspectorError, LoggingBackend, LoggingConfig, ServerConfig},
    run_server, InspectorConfig, InspectorService,
};
use mcp_inspector_mcp::services::config_manager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// MCP Inspector Server - Monitor and debug MCP servers
#[derive(Parser, Debug)]
#[command(name = "mcp_inspector")]
#[command(about = "MCP Server Inspector for monitoring and debugging", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose debug output with detailed logging
    #[arg(short, long)]
    verbose: bool,

    /// Optional path to configuration file (default: .inspector/config.json)
    #[arg(short, long)]
    config: Option<String>,
}

/// Initialize logging with optional file output
///
/// This function sets up the tracing subscriber with:
/// - Configurable log level based on verbose mode
/// - Optional file output with rotation
/// - ANSI colors disabled for MCP compatibility
/// - Logs written to stderr to avoid interfering with JSON-RPC on stdout
///
/// # Arguments
/// * `verbose` - Enable verbose (DEBUG level) logging
///
/// # Returns
/// Returns `Ok(())` on success, or an error if logging initialization fails
fn init_logging(verbose: bool) -> Result<()> {
    use tracing_appender::rolling::RollingFileAppender;

    // Configure log level and file output based on verbose mode
    if verbose {
        debug_config::configure_verbose_logging(true);
    }

    let log_config = debug_config::get_log_config();
    let log_level = log_config.level.to_tracing_level();

    // Create stderr layer (always enabled)
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(false) // Disable ANSI colors for MCP compatibility
        .with_writer(std::io::stderr);

    if log_config.output_to_file {
        // Create file appender with rotation
        let file_appender = RollingFileAppender::new(
            log_config.log_file_rotation.to_tracing_rotation(),
            &log_config.log_file_path,
            "mcp_inspector.log",
        );

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // Create file layer
        let file_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_ansi(false)
            .with_writer(non_blocking);

        // Initialize subscriber with both stderr and file layers
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
            .with(stderr_layer)
            .with(file_layer)
            .init();

        // Keep the guard alive for the entire program lifetime
        // This is necessary to ensure the non_blocking writer is flushed properly
        std::mem::forget(_guard);
    } else {
        // Initialize subscriber with only stderr layer
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::from_level(log_level))
            .with(stderr_layer)
            .init();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging with verbose mode if requested
    init_logging(cli.verbose).context("Failed to initialize logging")?;

    tracing::info!("MCP Inspector Server starting...");
    if cli.verbose {
        tracing::debug!("Verbose mode enabled via CLI argument");
    }

    // Load configuration from .inspector/config.json
    let project_config = config_manager::load_config()
        .context("Failed to load configuration from .inspector/config.json")?;

    // Convert ProjectConfig to InspectorConfig
    let servers = convert_server_entries(&project_config.servers)?;
    let logging = convert_logging_settings(&project_config.logging);
    let mut execution_config = project_config.execution_config.clone();

    // CLI argument takes precedence over config file
    if cli.verbose {
        execution_config.verbose = true;
    }

    // Set global verbose mode based on execution config
    if execution_config.verbose {
        debug_config::enable_verbose_mode();
        tracing::info!("Verbose mode enabled from {}", if cli.verbose { "CLI argument" } else { "config file" });
    }

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
