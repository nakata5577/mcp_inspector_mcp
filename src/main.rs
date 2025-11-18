use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mcp_inspector_mcp::{
    models::{debug_config, InspectorError, LoggingBackend, LoggingConfig, ServerConfig, TimeWindow},
    run_server, InspectorConfig, InspectorService,
};
use mcp_inspector_mcp::services::{config_manager, BottleneckDetector, DetectionConfig, MetricsCollector, ReportFormat, ReportService};
use once_cell::sync::Lazy;
use std::path::PathBuf;
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
        Some(Commands::Inspect) | None => run_inspect_mode(cli.verbose).await,
    }
}

async fn run_inspect_mode(verbose: bool) -> Result<i32> {
    tracing::info!("MCP Inspector Server starting...");
    if verbose {
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
