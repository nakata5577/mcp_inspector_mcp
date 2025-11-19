use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mcp_inspector_mcp::{
    models::{debug_config, InspectorError, LoggingBackend, LoggingConfig, ServerConfig, TimeWindow},
    run_server, InspectorConfig, InspectorService,
};
use mcp_inspector_mcp::services::{
    BottleneckDetector, ConfigFormat, ConfigImportExport, ConfigTemplate, DetectionConfig,
    MetricsCollector, ProfileManager, ReportFormat, ReportService,
};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// MCP Inspector Server - Monitor and debug MCP servers
#[derive(Parser, Debug)]
#[command(name = "mcp_inspector")]
#[command(about = "MCP Server Inspector for monitoring and debugging", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose debug output with detailed logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Optional path to configuration file (default: .inspector/config.json)
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Profile to use (dev, staging, prod, etc.)
    #[arg(long, global = true)]
    profile: Option<String>,

    /// List available profiles
    #[arg(long)]
    list_profiles: bool,

    /// Validate a profile
    #[arg(long, value_name = "PROFILE")]
    validate_profile: Option<String>,

    /// Show performance metrics summary
    #[arg(long)]
    metrics: bool,

    /// Generate performance report (console, json, html)
    #[arg(long, value_name = "FORMAT")]
    report: Option<String>,

    /// Time window for metrics (1h, 24h, 7d)
    #[arg(long, default_value = "24h")]
    time_window: String,

    /// Detect performance bottlenecks
    #[arg(long)]
    detect_bottlenecks: bool,

    /// Output file for report (default: stdout)
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Global static variable to hold the log guard for the entire program lifetime
/// This ensures the non_blocking writer is flushed properly on program termination
static LOG_GUARD: Lazy<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(Subcommand, Debug)]
enum Commands {
    /// Inspect MCP server interactively (default mode)
    Inspect,

    /// Run batch tests from YAML/JSON file
    BatchTest {
        /// Path to test definition file (YAML or JSON)
        #[arg(short, long)]
        test_file: String,

        /// Report output format (console, junit, json)
        #[arg(long, default_value = "console")]
        report_format: String,

        /// Report output file path
        #[arg(long)]
        report_output: Option<String>,
    },

    /// Configuration management commands
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Export configuration to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Output format (json, yaml)
        #[arg(short, long)]
        format: Option<String>,

        /// Profile to export (default: current/default)
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// Import configuration from file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Input format (json, yaml)
        #[arg(short, long)]
        format: Option<String>,

        /// Dry-run mode (show changes without applying)
        #[arg(long)]
        dry_run: bool,

        /// Profile to import into
        #[arg(short, long)]
        profile: Option<String>,
    },

    /// Validate configuration file
    Validate {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Input format (json, yaml)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// List available templates
    Template {
        #[command(subcommand)]
        template_command: TemplateCommands,
    },
}

#[derive(Subcommand, Debug)]
enum TemplateCommands {
    /// List all available templates
    List,

    /// Show template content
    Show {
        /// Template name
        template: String,
    },

    /// Apply template to create/update profile
    Apply {
        /// Template name
        #[arg(short, long)]
        template: String,

        /// Output profile name
        #[arg(short, long)]
        profile: String,
    },

    /// Create custom template from existing profile
    Create {
        /// Template name
        #[arg(short, long)]
        name: String,

        /// Source profile name
        #[arg(short, long)]
        from: String,
    },

    /// Delete custom template
    Delete {
        /// Template name
        template: String,
    },
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

        // Store the guard in a static variable to keep it alive for the entire program lifetime
        // This ensures the non_blocking writer is flushed properly on program termination
        *LOG_GUARD.lock().unwrap() = Some(_guard);
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
async fn main() {
    let exit_code = run().await.unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        3 // Execution error
    });
    std::process::exit(exit_code);
}

async fn run() -> Result<i32> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging with verbose mode if requested
    init_logging(cli.verbose).context("Failed to initialize logging")?;

    // Handle profile listing
    if cli.list_profiles {
        return run_list_profiles().await;
    }

    // Handle profile validation
    if let Some(ref profile_name) = cli.validate_profile {
        return run_validate_profile(profile_name).await;
    }

    // Check if any metrics-related options are specified
    let has_metrics_options = cli.metrics || cli.report.is_some() || cli.detect_bottlenecks;

    // If metrics options are specified, run metrics mode
    if has_metrics_options {
        return run_metrics_mode(
            &cli.time_window,
            cli.metrics,
            cli.report.as_deref(),
            cli.detect_bottlenecks,
            cli.output.as_deref(),
        )
        .await;
    }

    // Dispatch command
    match cli.command {
        Some(Commands::BatchTest {
            test_file,
            report_format,
            report_output,
        }) => run_batch_test(&test_file, &report_format, report_output.as_deref()).await,
        Some(Commands::Config { config_command }) => run_config_command(config_command).await,
        Some(Commands::Inspect) | None => run_inspect_mode(cli.verbose, cli.profile.as_deref()).await,
    }
}

async fn run_inspect_mode(verbose: bool, profile: Option<&str>) -> Result<i32> {
    tracing::info!("MCP Inspector Server starting...");
    if verbose {
        tracing::debug!("Verbose mode enabled via CLI argument");
    }

    // Load configuration using ProfileManager
    let profile_manager = ProfileManager::new()
        .context("Failed to initialize ProfileManager")?;

    let (profile_config, active_profile) = profile_manager
        .load_active_profile(profile)
        .context("Failed to load profile configuration")?;

    tracing::info!("Using profile: {}", active_profile);

    // Convert ProfileConfig to ProjectConfig
    let project_config = mcp_inspector_mcp::models::ProjectConfig {
        servers: profile_config.servers.clone(),
        logging: profile_config.logging.clone(),
        execution_config: profile_config.execution_config.clone(),
    };

    // Convert ProjectConfig to InspectorConfig
    let servers = convert_server_entries(&project_config.servers)?;
    let logging = convert_logging_settings(&project_config.logging);
    let mut execution_config = project_config.execution_config.clone();

    // CLI argument takes precedence over config file
    if verbose {
        execution_config.verbose = true;
    }

    // Set global verbose mode based on execution config
    if execution_config.verbose {
        debug_config::enable_verbose_mode();
        tracing::info!("Verbose mode enabled from {}", if verbose { "CLI argument" } else { "config file" });
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

    Ok(0)
}

async fn run_batch_test(
    test_file: &str,
    report_format: &str,
    report_output: Option<&str>,
) -> Result<i32> {
    use mcp_inspector_mcp::models::TestSuite;
    use mcp_inspector_mcp::services::TestExecutor;
    use std::path::Path;

    tracing::info!("Running batch tests from: {}", test_file);

    // テスト定義ファイルの読み込み
    let suite = TestSuite::from_file(Path::new(test_file))
        .with_context(|| format!("Failed to load test file: {}", test_file))?;

    // バリデーション
    suite.validate()
        .context("Test suite validation failed")?;

    tracing::info!("Test suite loaded: {} (version: {})", suite.name, suite.version);
    tracing::info!("Test count: {}", suite.tests.len());

    // テスト実行
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&suite).await
        .context("Test execution failed")?;

    // レポート出力
    match report_format {
        "console" => {
            print_console_report(&suite.name, &results);
        }
        "junit" => {
            let xml_report = generate_junit_report(&suite.name, &results)?;
            if let Some(output_path) = report_output {
                std::fs::write(output_path, xml_report)
                    .with_context(|| format!("Failed to write JUnit report to {}", output_path))?;
                tracing::info!("JUnit report written to: {}", output_path);
            } else {
                println!("{}", xml_report);
            }
        }
        "json" => {
            let json_report = serde_json::to_string_pretty(&results)
                .context("Failed to serialize results to JSON")?;
            if let Some(output_path) = report_output {
                std::fs::write(output_path, json_report)
                    .with_context(|| format!("Failed to write JSON report to {}", output_path))?;
                tracing::info!("JSON report written to: {}", output_path);
            } else {
                println!("{}", json_report);
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown report format: {}", report_format));
        }
    }

    // 終了コードを決定
    let all_passed = results.iter().all(|r| r.passed);
    if all_passed {
        Ok(0) // 全テスト成功
    } else {
        Ok(1) // テスト失敗
    }
}

fn print_console_report(suite_name: &str, results: &[mcp_inspector_mcp::services::TestResult]) {
    println!("╔{}╗", "═".repeat(62));
    println!("║ {:^60} ║", suite_name);
    println!("╚{}╝", "═".repeat(62));
    println!();

    for result in results {
        let status = if result.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("{} {} ({}ms)", status, result.test_name, result.duration_ms);

        if !result.passed {
            for assertion in &result.assertions {
                if !assertion.passed {
                    println!("  ✗ {}: {}", assertion.assertion_type, assertion.message);
                }
            }
            if let Some(error) = &result.error {
                println!("  Error: {}", error);
            }
        }
    }

    println!();
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    println!("📊 Summary: {} total, {} passed, {} failed", total, passed, failed);
}

fn generate_junit_report(suite_name: &str, results: &[mcp_inspector_mcp::services::TestResult]) -> Result<String> {
    // 簡易的なJUnit XML生成（後で完全な実装に置き換え）
    let total_tests = results.len();
    let failures = results.iter().filter(|r| !r.passed).count();
    let total_time: f64 = results.iter().map(|r| r.duration_ms as f64 / 1000.0).sum();

    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    xml.push_str(&format!(
        r#"<testsuites tests="{}" failures="{}" time="{:.3}">"#,
        total_tests, failures, total_time
    ));
    xml.push('\n');
    xml.push_str(&format!(
        r#"  <testsuite name="{}" tests="{}" failures="{}" time="{:.3}">"#,
        suite_name, total_tests, failures, total_time
    ));
    xml.push('\n');

    for result in results {
        let time = result.duration_ms as f64 / 1000.0;
        xml.push_str(&format!(
            r#"    <testcase name="{}" time="{:.3}">"#,
            result.test_name, time
        ));
        xml.push('\n');

        if !result.passed {
            xml.push_str(r#"      <failure message="Test failed">"#);
            xml.push('\n');
            if let Some(error) = &result.error {
                xml.push_str(&format!("Error: {}\n", error));
            }
            for assertion in &result.assertions {
                if !assertion.passed {
                    xml.push_str(&format!("{}: {}\n", assertion.assertion_type, assertion.message));
                }
            }
            xml.push_str(r#"      </failure>"#);
            xml.push('\n');
        }

        xml.push_str(r#"    </testcase>"#);
        xml.push('\n');
    }

    xml.push_str(r#"  </testsuite>"#);
    xml.push('\n');
    xml.push_str(r#"</testsuites>"#);
    xml.push('\n');

    Ok(xml)
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

/// Parse time window string to TimeWindow enum
fn parse_time_window(s: &str) -> Result<TimeWindow> {
    match s.to_lowercase().as_str() {
        "1h" => Ok(TimeWindow::LastHour),
        "24h" => Ok(TimeWindow::Last24Hours),
        "7d" => Ok(TimeWindow::Last7Days),
        _ => Err(anyhow::anyhow!(
            "Invalid time window: {}. Valid values: 1h, 24h, 7d",
            s
        )),
    }
}

/// Parse report format string to ReportFormat enum
fn parse_report_format(s: &str) -> Result<ReportFormat> {
    match s.to_lowercase().as_str() {
        "console" => Ok(ReportFormat::Console),
        "json" => Ok(ReportFormat::Json),
        "html" => Ok(ReportFormat::Html),
        _ => Err(anyhow::anyhow!(
            "Invalid report format: {}. Valid values: console, json, html",
            s
        )),
    }
}

/// Run metrics mode (--metrics, --report, --detect-bottlenecks)
async fn run_metrics_mode(
    time_window_str: &str,
    show_metrics: bool,
    report_format: Option<&str>,
    detect_bottlenecks: bool,
    output_file: Option<&std::path::Path>,
) -> Result<i32> {
    tracing::info!("Running metrics mode");

    // Parse time window
    let time_window = parse_time_window(time_window_str)
        .context("Failed to parse time window")?;

    // For now, create a mock metrics collector with sample data
    // In a real implementation, this would load from persistent storage
    let metrics_collector = Arc::new(MetricsCollector::new());

    // Show metrics summary if requested
    if show_metrics {
        let server_metrics = metrics_collector.aggregate_by_server(time_window);

        if server_metrics.is_empty() {
            println!("No metrics available for the specified time window.");
            println!("Note: Metrics are only collected during runtime.");
            return Ok(0);
        }

        println!("\n=== Metrics Summary ({}) ===\n", time_window);

        for (server_name, server_agg) in &server_metrics {
            println!("Server: {}", server_name);
            println!("  Total Requests: {}", server_agg.total_requests);
            println!("  Success Rate: {:.1}%", 100.0 - server_agg.error_rate);
            println!("  Response Time (p50): {}ms", server_agg.response_time.p50);
            println!("  Response Time (p99): {}ms", server_agg.response_time.p99);
            println!("  Throughput: {:.2} req/s", server_agg.throughput);
            println!();

            // Show tool-level metrics
            let tool_metrics = metrics_collector.aggregate_by_tool(server_name, time_window);
            if !tool_metrics.is_empty() {
                println!("  Tools:");
                for (tool_name, tool_agg) in &tool_metrics {
                    println!("    - {}: {} requests, p99={}ms",
                        tool_name,
                        tool_agg.total_requests,
                        tool_agg.response_time.p99
                    );
                }
                println!();
            }
        }
    }

    // Generate report if requested
    if let Some(format_str) = report_format {
        let format = parse_report_format(format_str)
            .context("Failed to parse report format")?;

        let report_service = ReportService::new(metrics_collector.clone());
        let report = report_service.generate_report(time_window, format)
            .context("Failed to generate report")?;

        // Output report
        if let Some(path) = output_file {
            std::fs::write(path, &report)
                .with_context(|| format!("Failed to write report to {:?}", path))?;
            println!("Report written to: {}", path.display());
        } else {
            println!("{}", report);
        }
    }

    // Detect bottlenecks if requested
    if detect_bottlenecks {
        let config = DetectionConfig::default();
        let detector = BottleneckDetector::new(metrics_collector.clone(), config);
        let bottlenecks = detector.detect_bottlenecks(time_window);

        if bottlenecks.is_empty() {
            println!("\n=== No Bottlenecks Detected ===");
            println!("All metrics are within acceptable thresholds.");
        } else {
            println!("\n=== Bottlenecks Detected ({}) ===\n", bottlenecks.len());
            for bottleneck in &bottlenecks {
                println!("{}", bottleneck.format_alert());
            }
        }
    }

    Ok(0)
}

/// List available profiles
async fn run_list_profiles() -> Result<i32> {
    let profile_manager = ProfileManager::new()
        .context("Failed to initialize ProfileManager")?;

    let profiles = profile_manager.list_profiles()
        .context("Failed to list profiles")?;

    if profiles.is_empty() {
        println!("No profiles found.");
        return Ok(0);
    }

    println!("\n=== Available Profiles ===\n");

    for profile in &profiles {
        println!("Profile: {}", profile.name);
        if let Some(desc) = &profile.description {
            println!("  Description: {}", desc);
        }
        if !profile.tags.is_empty() {
            println!("  Tags: {}", profile.tags.join(", "));
        }
        println!("  Path: {}", profile.config_path);
        println!();
    }

    println!("Total: {} profile(s)", profiles.len());

    Ok(0)
}

/// Validate a profile
async fn run_validate_profile(profile_name: &str) -> Result<i32> {
    let profile_manager = ProfileManager::new()
        .context("Failed to initialize ProfileManager")?;

    match profile_manager.validate_profile(profile_name) {
        Ok(()) => {
            println!("✓ Profile '{}' is valid.", profile_name);
            Ok(0)
        }
        Err(e) => {
            eprintln!("✗ Profile '{}' validation failed: {}", profile_name, e);
            Ok(1)
        }
    }
}

/// Handle config commands
async fn run_config_command(config_command: ConfigCommands) -> Result<i32> {
    match config_command {
        ConfigCommands::Export {
            output,
            format,
            profile,
        } => run_config_export(&output, format.as_deref(), profile.as_deref()).await,
        ConfigCommands::Import {
            input,
            format,
            dry_run,
            profile,
        } => run_config_import(&input, format.as_deref(), dry_run, profile.as_deref()).await,
        ConfigCommands::Validate { input, format } => {
            run_config_validate(&input, format.as_deref()).await
        }
        ConfigCommands::Template { template_command } => {
            run_template_command(template_command).await
        }
    }
}

/// Export configuration to file
async fn run_config_export(
    output: &str,
    format: Option<&str>,
    profile: Option<&str>,
) -> Result<i32> {
    let profile_manager = ProfileManager::new()
        .context("Failed to initialize ProfileManager")?;

    // Load profile
    let (config, active_profile) = profile_manager
        .load_active_profile(profile)
        .context("Failed to load profile")?;

    // Determine format
    let format = match format {
        Some(f) => f.parse()?,
        None => ConfigFormat::from_path(Path::new(output))?,
    };

    // Export
    let size = ConfigImportExport::export_config(&config, Path::new(output), Some(format))
        .context("Failed to export configuration")?;

    println!(
        "✓ Configuration exported to {} ({} bytes, format: {:?}, profile: {})",
        output, size, format, active_profile
    );

    Ok(0)
}

/// Import configuration from file
async fn run_config_import(
    input: &str,
    format: Option<&str>,
    dry_run: bool,
    profile: Option<&str>,
) -> Result<i32> {
    let profile_manager = ProfileManager::new()
        .context("Failed to initialize ProfileManager")?;

    // Determine format
    let format = match format {
        Some(f) => Some(f.parse()?),
        None => Some(ConfigFormat::from_path(Path::new(input))?),
    };

    if dry_run {
        // Dry-run mode: show diff without applying
        let (current_config, active_profile) = profile_manager
            .load_active_profile(profile)
            .context("Failed to load current profile")?;

        let diff = ConfigImportExport::dry_run_import(&current_config, Path::new(input), format)
            .context("Failed to perform dry-run import")?;

        println!("\n=== Dry-Run: Import Preview ===\n");
        println!("Target profile: {}", profile.unwrap_or(&active_profile));
        println!("{}", diff.format());

        if diff.has_changes() {
            println!("To apply these changes, run without --dry-run flag.");
        }

        Ok(0)
    } else {
        // Import and save
        let config = ConfigImportExport::import_config(Path::new(input), format)
            .context("Failed to import configuration")?;

        let target_profile = profile.unwrap_or("");
        profile_manager.save_profile(target_profile, &config)
            .context("Failed to save imported configuration")?;

        println!(
            "✓ Configuration imported from {} to profile '{}'",
            input,
            if target_profile.is_empty() {
                "default"
            } else {
                target_profile
            }
        );

        Ok(0)
    }
}

/// Validate configuration file
async fn run_config_validate(input: &str, format: Option<&str>) -> Result<i32> {
    // Determine format
    let format = match format {
        Some(f) => Some(f.parse()?),
        None => Some(ConfigFormat::from_path(Path::new(input))?),
    };

    match ConfigImportExport::validate_config_file(Path::new(input), format) {
        Ok(warnings) => {
            println!("✓ Configuration file is valid: {}", input);

            if !warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &warnings {
                    println!("  ⚠ {}", warning);
                }
            }

            Ok(0)
        }
        Err(e) => {
            eprintln!("✗ Configuration validation failed: {}", e);
            Ok(1)
        }
    }
}

/// Handle template commands
async fn run_template_command(template_command: TemplateCommands) -> Result<i32> {
    let template_manager = ConfigTemplate::new()
        .context("Failed to initialize ConfigTemplate")?;

    match template_command {
        TemplateCommands::List => {
            let templates = template_manager.list_all_templates()
                .context("Failed to list templates")?;

            println!("\n=== Available Templates ===\n");

            // Preset templates
            let presets: Vec<_> = templates.iter().filter(|t| t.is_preset).collect();
            if !presets.is_empty() {
                println!("Preset Templates:");
                for template in presets {
                    println!("  - {}: {}", template.name, template.description);
                }
                println!();
            }

            // Custom templates
            let customs: Vec<_> = templates.iter().filter(|t| !t.is_preset).collect();
            if !customs.is_empty() {
                println!("Custom Templates:");
                for template in customs {
                    println!("  - {}: {}", template.name, template.description);
                }
                println!();
            }

            println!("Total: {} template(s)", templates.len());

            Ok(0)
        }
        TemplateCommands::Show { template } => {
            let json = template_manager.show_template(&template)
                .context("Failed to show template")?;

            println!("{}", json);

            Ok(0)
        }
        TemplateCommands::Apply { template, profile } => {
            let config = template_manager.apply_template(&template)
                .context("Failed to apply template")?;

            let profile_manager = ProfileManager::new()
                .context("Failed to initialize ProfileManager")?;

            profile_manager.save_profile(&profile, &config)
                .context("Failed to save profile")?;

            println!("✓ Template '{}' applied to profile '{}'", template, profile);

            Ok(0)
        }
        TemplateCommands::Create { name, from } => {
            let profile_manager = ProfileManager::new()
                .context("Failed to initialize ProfileManager")?;

            let config = profile_manager.load_profile(&from)
                .context("Failed to load source profile")?;

            template_manager.create_custom_template(&name, &config)
                .context("Failed to create custom template")?;

            println!("✓ Custom template '{}' created from profile '{}'", name, from);

            Ok(0)
        }
        TemplateCommands::Delete { template } => {
            template_manager.delete_custom_template(&template)
                .context("Failed to delete template")?;

            println!("✓ Custom template '{}' deleted", template);

            Ok(0)
        }
    }
}
