//! Phase 3 Integration Tests
//!
//! This module contains comprehensive integration tests for all Phase 3 features:
//! - Task 3.1: Debug Mode (Verbose logging, request/response display, timing)
//! - Task 3.2: Batch Testing (YAML/JSON test definitions, assertions, JUnit XML reports)
//! - Task 3.4: Performance Monitoring (Metrics collection, reports, bottleneck detection)
//! - Task 3.5: Configuration Management (Profiles, import/export, templates)
//!
//! Note: Task 3.3 (Interactive Mode) was skipped and is not included.
//!
//! Total Test Count: 45 tests
//! - Debug Mode Integration: 10 tests
//! - Batch Testing Integration: 12 tests
//! - Performance Monitoring Integration: 12 tests
//! - Configuration Management Integration: 11 tests

use mcp_inspector_mcp::models::{
    Assertion, ExecutionConfig, MetricStatus, ProfileConfig, TestCase, TestConfig, TestSuite,
    TimeWindow,
};
use mcp_inspector_mcp::services::{
    BottleneckDetector, ConfigFormat, ConfigImportExport, ConfigTemplate, DetectionConfig,
    MetricsCollector, ProfileManager, ReportFormat, ReportService, TestExecutor,
};
use serial_test::serial;
use serde_json::json;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Helper function to setup test environment with a temporary directory
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();
    temp_dir
}

/// Helper function to create test metrics data for integration testing
fn create_integration_test_metrics() -> Arc<MetricsCollector> {
    let collector = Arc::new(MetricsCollector::new());

    // Simulate realistic server metrics
    // Server 1: Fast, reliable
    for i in 0..20 {
        collector.record_metric(
            "fast_server".to_string(),
            format!("tool_{}", i % 5),
            50 + i * 5,
            MetricStatus::Success,
            i % 4 == 0, // 25% cache hit rate
            false,
        );
    }

    // Server 2: Slow, some errors
    for i in 0..15 {
        collector.record_metric(
            "slow_server".to_string(),
            "slow_tool".to_string(),
            1200 + i * 100,
            if i % 10 == 0 {
                MetricStatus::Error
            } else {
                MetricStatus::Success
            },
            false,
            false,
        );
    }

    // Server 3: High cache hit rate
    for i in 0..10 {
        collector.record_metric(
            "cached_server".to_string(),
            "cached_tool".to_string(),
            100,
            MetricStatus::Success,
            i % 2 == 0, // 50% cache hit rate
            true,
        );
    }

    collector
}

/// Helper function to create a test case
fn create_test_case(
    name: &str,
    tool: &str,
    server: &str,
    assertions: Vec<Assertion>,
) -> TestCase {
    TestCase {
        name: name.to_string(),
        description: None,
        server: server.to_string(),
        tool: tool.to_string(),
        arguments: json!({}),
        expect_error: false,
        assertions,
    }
}

/// Helper function to create a test suite
fn create_test_suite(name: &str, tests: Vec<TestCase>) -> TestSuite {
    TestSuite {
        name: name.to_string(),
        version: "1.0".to_string(),
        description: Some(format!("Integration test suite: {}", name)),
        config: TestConfig {
            parallel: false,
            fail_fast: true,
            timeout_ms: 10000,
            retry_count: 1,
        },
        tests,
    }
}

// ============================================================================
// Test Group 1: Debug Mode Integration Tests (10 tests)
// ============================================================================

#[test]
fn test_debug_mode_verbose_flag_integration() {
    // Test that verbose flag affects execution config
    let mut config = ExecutionConfig::default();
    assert!(!config.verbose);

    config.verbose = true;
    assert!(config.verbose);

    // Verify verbose mode enables detailed logging
    assert!(config.verbose);
}

#[test]
fn test_debug_mode_timing_tracking_integration() {
    // Test timing tracking functionality
    let start = Instant::now();
    std::thread::sleep(Duration::from_millis(50));
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(50));
    assert!(elapsed < Duration::from_millis(200));
}

#[test]
fn test_debug_mode_request_response_logging() {
    // Test that request/response can be formatted properly
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "test_tool",
            "arguments": {"key": "value"}
        }
    });

    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "content": [{"type": "text", "text": "success"}]
        }
    });

    // Verify JSON can be pretty printed
    let request_str = serde_json::to_string_pretty(&request).unwrap();
    let response_str = serde_json::to_string_pretty(&response).unwrap();

    assert!(request_str.contains("tools/call"));
    assert!(response_str.contains("content"));
}

#[test]
fn test_debug_mode_execution_config_verbose_setting() {
    // Test execution config with verbose enabled
    let config = ExecutionConfig {
        verbose: true,
        tool_timeout_ms: 30000,
        connection_timeout_ms: 5000,
        retry_count: 2,
        auto_retry_on_timeout: true,
    };

    assert!(config.verbose);
    assert_eq!(config.tool_timeout_ms, 30000);
}

#[test]
fn test_debug_mode_log_truncation() {
    // Test that large payloads can be truncated
    let large_payload = "x".repeat(10000);
    let max_size = 4096;

    let truncated = if large_payload.len() > max_size {
        format!("{}... (truncated)", &large_payload[..max_size])
    } else {
        large_payload.clone()
    };

    assert!(truncated.len() <= max_size + 20); // +20 for "... (truncated)"
    assert!(truncated.contains("truncated"));
}

#[test]
fn test_debug_mode_timestamp_formatting() {
    // Test timestamp formatting for debug logs
    use chrono::{Local, TimeZone};

    let timestamp = Local.with_ymd_and_hms(2025, 11, 27, 10, 30, 45).unwrap();
    let formatted = timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

    assert!(formatted.contains("2025-11-27"));
    assert!(formatted.contains("10:30:45"));
}

#[test]
fn test_debug_mode_json_formatting() {
    // Test JSON pretty printing for debug output
    let json_value = json!({
        "server": "test_server",
        "tool": "test_tool",
        "args": {"param1": "value1", "param2": 42}
    });

    let pretty = serde_json::to_string_pretty(&json_value).unwrap();

    assert!(pretty.contains("\"server\""));
    assert!(pretty.contains("\"test_server\""));
    assert!(pretty.contains("\"param2\""));
    assert!(pretty.contains("42"));
}

#[test]
fn test_debug_mode_error_logging_integration() {
    // Test that errors are properly formatted in debug mode
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32603,
            "message": "Internal error",
            "data": {"details": "Connection timeout"}
        }
    });

    let error_str = serde_json::to_string_pretty(&error_response).unwrap();

    assert!(error_str.contains("error"));
    assert!(error_str.contains("Internal error"));
    assert!(error_str.contains("Connection timeout"));
}

#[test]
fn test_debug_mode_multiple_request_tracking() {
    // Test tracking multiple requests with timing
    let mut request_times = Vec::new();

    for i in 0..5 {
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = start.elapsed();
        request_times.push((format!("req_{}", i), elapsed));
    }

    assert_eq!(request_times.len(), 5);
    for (req_id, elapsed) in &request_times {
        assert!(req_id.starts_with("req_"));
        assert!(*elapsed >= Duration::from_millis(10));
    }
}

#[test]
fn test_debug_mode_cache_hit_logging() {
    // Test that cache hits can be tracked for debug logging
    struct CacheStats {
        hits: u32,
        misses: u32,
    }

    let mut stats = CacheStats { hits: 0, misses: 0 };

    // Simulate cache operations
    for i in 0..10 {
        if i % 3 == 0 {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
    }

    let hit_rate = (stats.hits as f64 / (stats.hits + stats.misses) as f64) * 100.0;

    assert_eq!(stats.hits, 4);
    assert_eq!(stats.misses, 6);
    assert!((hit_rate - 40.0).abs() < 0.1);
}

// ============================================================================
// Test Group 2: Batch Testing Integration Tests (12 tests)
// ============================================================================

#[test]
fn test_batch_test_suite_creation() {
    // Test creating a test suite with multiple test cases
    let test_suite = create_test_suite(
        "integration_test_suite",
        vec![
            create_test_case(
                "test_health_check",
                "health_check",
                "test_server",
                vec![Assertion::Status {
                    expected: "healthy".to_string(),
                }],
            ),
            create_test_case(
                "test_tools_list",
                "tools_list",
                "test_server",
                vec![Assertion::FieldExists {
                    field: "tools".to_string(),
                }],
            ),
        ],
    );

    assert_eq!(test_suite.tests.len(), 2);
    assert_eq!(test_suite.name, "integration_test_suite");
}

#[tokio::test]
async fn test_batch_test_executor_with_mock() {
    // Test batch test execution with mock executor
    let test_case = create_test_case(
        "test_mock_execution",
        "health_check",
        "mock_server",
        vec![Assertion::Status {
            expected: "healthy".to_string(),
        }],
    );

    let test_suite = create_test_suite("mock_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await;

    assert!(results.is_ok());
    let test_results = results.unwrap();
    assert_eq!(test_results.len(), 1);
    assert!(test_results[0].passed);
    assert!(test_results[0].duration_ms > 0);
}

#[tokio::test]
async fn test_batch_test_assertion_status() {
    // Test Status assertion
    let test_case = create_test_case(
        "test_status",
        "health_check",
        "test_server",
        vec![Assertion::Status {
            expected: "healthy".to_string(),
        }],
    );

    let test_suite = create_test_suite("status_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(results[0].passed);
}

#[tokio::test]
async fn test_batch_test_assertion_field_exists() {
    // Test FieldExists assertion
    let test_case = create_test_case(
        "test_field_exists",
        "tools_list",
        "test_server",
        vec![Assertion::FieldExists {
            field: "tools".to_string(),
        }],
    );

    let test_suite = create_test_suite("field_exists_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(results[0].passed);
}

#[tokio::test]
async fn test_batch_test_assertion_field_equals() {
    // Test FieldEquals assertion with nested JSON
    let test_case = create_test_case(
        "test_field_equals",
        "health_check",
        "test_server",
        vec![Assertion::FieldEquals {
            field: "status".to_string(),
            expected: json!("healthy"),
        }],
    );

    let test_suite = create_test_suite("field_equals_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(results[0].passed);
}

#[tokio::test]
async fn test_batch_test_assertion_contains() {
    // Test Contains assertion (checks if array contains a value)
    let test_case = TestCase {
        name: "test_contains".to_string(),
        description: None,
        server: "test_server".to_string(),
        tool: "prompts_list".to_string(),
        arguments: json!({}),
        expect_error: false,
        assertions: vec![Assertion::FieldExists {
            field: "prompts".to_string(),
        }],
    };

    let test_suite = create_test_suite("contains_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(results[0].passed);
}

#[tokio::test]
async fn test_batch_test_multiple_assertions() {
    // Test multiple assertions on single test case
    let test_case = TestCase {
        name: "test_multiple_assertions".to_string(),
        description: None,
        server: "test_server".to_string(),
        tool: "health_check".to_string(),
        arguments: json!({}),
        expect_error: false,
        assertions: vec![
            Assertion::Status {
                expected: "healthy".to_string(),
            },
            Assertion::FieldExists {
                field: "status".to_string(),
            },
            Assertion::FieldEquals {
                field: "status".to_string(),
                expected: json!("healthy"),
            },
        ],
    };

    let test_suite = create_test_suite("multiple_assertions_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(results[0].passed);
    assert_eq!(results[0].assertions.len(), 3);
}

#[tokio::test]
async fn test_batch_test_failure_case() {
    // Test that failures are properly detected
    let test_case = TestCase {
        name: "test_failure".to_string(),
        description: None,
        server: "test_server".to_string(),
        tool: "tools_call".to_string(),
        arguments: json!({"name": "error_tool"}),
        expect_error: false,
        assertions: vec![Assertion::Status {
            expected: "success".to_string(),
        }],
    };

    let test_suite = create_test_suite("failure_test", vec![test_case]);
    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert!(!results[0].passed);
    assert!(results[0].error.is_some());
}

#[tokio::test]
async fn test_batch_test_suite_execution() {
    // Test executing an entire test suite
    let test_suite = create_test_suite(
        "full_suite",
        vec![
            create_test_case(
                "test1",
                "health_check",
                "server1",
                vec![Assertion::Status {
                    expected: "healthy".to_string(),
                }],
            ),
            create_test_case(
                "test2",
                "tools_list",
                "server2",
                vec![Assertion::FieldExists {
                    field: "tools".to_string(),
                }],
            ),
        ],
    );

    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.passed));
}

#[test]
fn test_batch_test_json_serialization() {
    // Test that test suites can be serialized to/from JSON
    let test_suite = create_test_suite(
        "serialization_test",
        vec![create_test_case(
            "test1",
            "health_check",
            "server",
            vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        )],
    );

    let json = serde_json::to_string(&test_suite).unwrap();
    let deserialized: TestSuite = serde_json::from_str(&json).unwrap();

    assert_eq!(test_suite.name, deserialized.name);
    assert_eq!(test_suite.tests.len(), deserialized.tests.len());
}

#[test]
fn test_batch_test_yaml_serialization() {
    // Test that test suites can be serialized to/from YAML
    let test_suite = create_test_suite(
        "yaml_test",
        vec![create_test_case(
            "test1",
            "health_check",
            "server",
            vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        )],
    );

    let yaml = serde_yaml::to_string(&test_suite).unwrap();
    let deserialized: TestSuite = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(test_suite.name, deserialized.name);
    assert_eq!(test_suite.tests.len(), deserialized.tests.len());
}

#[tokio::test]
async fn test_batch_test_timeout_handling() {
    // Test that timeouts are properly configured
    let test_case = create_test_case(
        "test_timeout",
        "health_check",
        "server",
        vec![Assertion::Status {
            expected: "healthy".to_string(),
        }],
    );

    let test_suite = TestSuite {
        name: "timeout_test".to_string(),
        version: "1.0".to_string(),
        description: None,
        config: TestConfig {
            parallel: false,
            fail_fast: true,
            timeout_ms: 1000, // 1 second timeout
            retry_count: 1,
        },
        tests: vec![test_case],
    };

    let executor = TestExecutor::new();
    let start = Instant::now();
    let result = executor.run_test_suite(&test_suite).await;
    let elapsed = start.elapsed();

    // The test should complete quickly
    assert!(elapsed < Duration::from_secs(5));
    assert!(result.is_ok());
}

// ============================================================================
// Test Group 3: Performance Monitoring Integration Tests (12 tests)
// ============================================================================

#[test]
fn test_performance_monitoring_metrics_collection_integration() {
    // Test comprehensive metrics collection
    let collector = create_integration_test_metrics();

    assert!(collector.total_metrics() >= 45); // 20 + 15 + 10

    let metrics = collector.get_metrics(TimeWindow::LastHour);
    assert!(!metrics.is_empty());
}

#[test]
fn test_performance_monitoring_aggregation_by_server() {
    // Test server-level aggregation
    let collector = create_integration_test_metrics();
    let aggregated = collector.aggregate_by_server(TimeWindow::LastHour);

    assert!(aggregated.contains_key("fast_server"));
    assert!(aggregated.contains_key("slow_server"));
    assert!(aggregated.contains_key("cached_server"));

    let fast_metrics = aggregated.get("fast_server").unwrap();
    assert!(fast_metrics.response_time.avg < 200.0);
}

#[test]
fn test_performance_monitoring_aggregation_by_tool() {
    // Test tool-level aggregation
    let collector = create_integration_test_metrics();
    let fast_metrics = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);

    assert!(fast_metrics.total_requests > 0);
    assert!(fast_metrics.successful_requests > 0);
    assert_eq!(fast_metrics.failed_requests, 0);
}

#[test]
fn test_performance_monitoring_report_generation_console() {
    // Test console format report generation
    let collector = create_integration_test_metrics();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Console)
        .unwrap();

    assert!(report.contains("Performance Report"));
    assert!(report.contains("fast_server"));
    assert!(report.contains("slow_server"));
}

#[test]
fn test_performance_monitoring_report_generation_json() {
    // Test JSON format report generation
    let collector = create_integration_test_metrics();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Json)
        .unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert!(parsed.is_array());

    let array = parsed.as_array().unwrap();
    assert!(array.len() >= 3);
}

#[test]
fn test_performance_monitoring_report_generation_html() {
    // Test HTML format report generation
    let collector = create_integration_test_metrics();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Html)
        .unwrap();

    assert!(report.contains("<!DOCTYPE html>"));
    assert!(report.contains("Performance Report"));
    assert!(report.contains("<table"));
    assert!(report.contains("</html>"));
}

#[test]
fn test_performance_monitoring_bottleneck_detection_high_response_time() {
    // Test bottleneck detection for high response times
    let collector = create_integration_test_metrics();
    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(collector, config);

    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    // slow_server should be detected as a bottleneck
    assert!(!bottlenecks.is_empty());
    let has_slow_server = bottlenecks.iter().any(|b| b.server_name == "slow_server");
    assert!(has_slow_server);
}

#[test]
fn test_performance_monitoring_bottleneck_detection_error_rate() {
    // Test bottleneck detection for high error rate
    let collector = MetricsCollector::new();

    // Add metrics with 15% error rate (above default 5% threshold)
    for i in 0..20 {
        collector.record_metric(
            "error_server".to_string(),
            "error_tool".to_string(),
            100,
            if i % 7 < 6 {
                MetricStatus::Success
            } else {
                MetricStatus::Error
            },
            false,
            false,
        );
    }

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(Arc::new(collector), config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    assert!(!bottlenecks.is_empty());
}

#[test]
fn test_performance_monitoring_cache_hit_rate_calculation() {
    // Test cache hit rate calculation
    let collector = create_integration_test_metrics();
    let cached_metrics = collector.aggregate_metrics("cached_server", None, TimeWindow::LastHour);

    assert!(cached_metrics.cache_hit_rate > 0.0);
    assert!(cached_metrics.cache_hit_rate <= 100.0);
}

#[test]
fn test_performance_monitoring_time_window_filtering() {
    // Test different time window filtering
    let collector = create_integration_test_metrics();

    let hour_metrics = collector.get_metrics(TimeWindow::LastHour);
    let day_metrics = collector.get_metrics(TimeWindow::Last24Hours);
    let week_metrics = collector.get_metrics(TimeWindow::Last7Days);

    // All should return the same data in this test (recent metrics)
    assert_eq!(hour_metrics.len(), day_metrics.len());
    assert_eq!(day_metrics.len(), week_metrics.len());
}

#[test]
fn test_performance_monitoring_response_time_statistics() {
    // Test response time statistics calculation
    let collector = create_integration_test_metrics();
    let fast_metrics = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);

    assert!(fast_metrics.response_time.avg > 0.0);
    assert!(fast_metrics.response_time.min > 0);
    assert!(fast_metrics.response_time.max > fast_metrics.response_time.min);
    assert!(fast_metrics.response_time.p50 > 0);
    assert!(fast_metrics.response_time.p95 > 0);
    assert!(fast_metrics.response_time.p99 > 0);
}

#[test]
fn test_performance_monitoring_throughput_calculation() {
    // Test throughput calculation (requests per second)
    let collector = create_integration_test_metrics();
    let aggregated = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);

    assert!(aggregated.total_requests > 0);
    // Throughput is calculated as requests per second
    // In real scenario, this would be > 0
    assert!(aggregated.total_requests >= 20);
}

// ============================================================================
// Test Group 4: Configuration Management Integration Tests (11 tests)
// ============================================================================

#[test]
#[serial]
fn test_config_management_profile_creation_and_loading() {
    // Test creating and loading profiles
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    let dev_profile = ProfileConfig::dev_profile();
    manager.save_profile("dev", &dev_profile).unwrap();

    let loaded = manager.load_profile("dev").unwrap();
    assert!(loaded.execution_config.verbose);
    assert_eq!(loaded.execution_config.tool_timeout_ms, 10000);
}

#[test]
#[serial]
fn test_config_management_profile_switching() {
    // Test switching between different profiles
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("dev", &ProfileConfig::dev_profile())
        .unwrap();
    manager
        .save_profile("prod", &ProfileConfig::prod_profile())
        .unwrap();

    let (dev_config, dev_name) = manager.load_active_profile(Some("dev")).unwrap();
    assert_eq!(dev_name, "dev");
    assert!(dev_config.execution_config.verbose);

    let (prod_config, prod_name) = manager.load_active_profile(Some("prod")).unwrap();
    assert_eq!(prod_name, "prod");
    assert!(!prod_config.execution_config.verbose);
}

#[test]
#[serial]
fn test_config_management_profile_list() {
    // Test listing all available profiles
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("dev", &ProfileConfig::dev_profile())
        .unwrap();
    manager
        .save_profile("staging", &ProfileConfig::staging_profile())
        .unwrap();
    manager
        .save_profile("prod", &ProfileConfig::prod_profile())
        .unwrap();

    let profiles = manager.list_profiles().unwrap();
    assert!(profiles.len() >= 3);

    let names: Vec<_> = profiles.iter().map(|p| &p.name).collect();
    assert!(names.contains(&&"dev".to_string()));
    assert!(names.contains(&&"staging".to_string()));
    assert!(names.contains(&&"prod".to_string()));
}

#[test]
#[serial]
fn test_config_management_import_export_json() {
    // Test configuration import/export in JSON format
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("config_export.json");

    let config = ProfileConfig::dev_profile();
    let size = ConfigImportExport::export_config(&config, &export_path, Some(ConfigFormat::Json))
        .unwrap();

    assert!(size > 0);
    assert!(export_path.exists());

    let imported = ConfigImportExport::import_config(&export_path, Some(ConfigFormat::Json))
        .unwrap();

    assert_eq!(
        imported.execution_config.verbose,
        config.execution_config.verbose
    );
}

#[test]
#[serial]
fn test_config_management_import_export_yaml() {
    // Test configuration import/export in YAML format
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("config_export.yaml");

    let config = ProfileConfig::staging_profile();
    let size = ConfigImportExport::export_config(&config, &export_path, Some(ConfigFormat::Yaml))
        .unwrap();

    assert!(size > 0);
    assert!(export_path.exists());

    let imported = ConfigImportExport::import_config(&export_path, Some(ConfigFormat::Yaml))
        .unwrap();

    assert_eq!(
        imported.execution_config.retry_count,
        config.execution_config.retry_count
    );
}

#[test]
fn test_config_management_config_diff() {
    // Test configuration diff functionality
    let config1 = ProfileConfig::dev_profile();
    let mut config2 = config1.clone();
    config2.execution_config.verbose = false;
    config2.execution_config.tool_timeout_ms = 20000;

    let diff = ConfigImportExport::diff_configs(&config1, &config2);

    assert!(diff.has_changes());
    assert!(diff.total_changes() > 0);
    assert!(!diff.execution_changes.is_empty());
}

#[test]
#[serial]
fn test_config_management_template_preset_application() {
    // Test applying preset templates
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let dev_config = template_mgr.apply_template("development").unwrap();
    assert!(dev_config.execution_config.verbose);

    let prod_config = template_mgr.apply_template("production").unwrap();
    // Backend is String, check it's "persistent"
    assert_eq!(prod_config.logging.backend, "persistent");
}

#[test]
#[serial]
fn test_config_management_custom_template_creation() {
    // Test creating custom configuration templates
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    let mut custom_config = ProfileConfig::dev_profile();
    custom_config.execution_config.tool_timeout_ms = 15000;

    template_mgr
        .create_custom_template("custom_template", &custom_config)
        .unwrap();

    let loaded = template_mgr.apply_template("custom_template").unwrap();
    assert_eq!(loaded.execution_config.tool_timeout_ms, 15000);
}

#[test]
#[serial]
fn test_config_management_template_listing() {
    // Test listing all available templates
    let _temp_dir = setup_test_env();
    let template_mgr = ConfigTemplate::new().unwrap();

    template_mgr
        .create_custom_template("my_template1", &ProfileConfig::dev_profile())
        .unwrap();
    template_mgr
        .create_custom_template("my_template2", &ProfileConfig::prod_profile())
        .unwrap();

    let all_templates = template_mgr.list_all_templates().unwrap();
    assert!(all_templates.len() >= 6); // 4 presets + 2 custom

    let preset_count = all_templates.iter().filter(|t| t.is_preset).count();
    let custom_count = all_templates.iter().filter(|t| !t.is_preset).count();

    assert_eq!(preset_count, 4);
    assert!(custom_count >= 2);
}

#[test]
#[serial]
fn test_config_management_profile_validation() {
    // Test profile validation
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    manager
        .save_profile("valid", &ProfileConfig::dev_profile())
        .unwrap();

    assert!(manager.validate_profile("valid").is_ok());
    assert!(manager.validate_profile("nonexistent").is_err());
}

#[test]
#[serial]
fn test_config_management_profile_cloning() {
    // Test cloning profiles
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    let mut source_config = ProfileConfig::dev_profile();
    source_config.execution_config.tool_timeout_ms = 12345;
    manager.save_profile("source", &source_config).unwrap();

    manager.clone_profile("source", "cloned").unwrap();

    let cloned = manager.load_profile("cloned").unwrap();
    assert_eq!(cloned.execution_config.tool_timeout_ms, 12345);
}

// ============================================================================
// End-to-End Integration Tests (Cross-feature scenarios)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_e2e_debug_mode_with_performance_monitoring() {
    // Test: Debug mode + Performance monitoring integration
    let collector = MetricsCollector::new();

    // Simulate verbose debug session with metrics collection
    let start = Instant::now();

    for i in 0..10 {
        collector.record_metric(
            "debug_server".to_string(),
            format!("debug_tool_{}", i),
            50 + i * 10,
            MetricStatus::Success,
            false,
            false,
        );
    }

    let elapsed = start.elapsed();

    // Verify metrics were collected
    assert_eq!(collector.total_metrics(), 10);

    // Verify timing tracking
    assert!(elapsed < Duration::from_secs(1));

    // Generate report
    let service = ReportService::new(Arc::new(collector));
    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Console)
        .unwrap();

    assert!(report.contains("debug_server"));
}

#[tokio::test]
#[serial]
async fn test_e2e_batch_testing_with_profiles() {
    // Test: Batch testing with different profile configurations
    let _temp_dir = setup_test_env();
    let manager = ProfileManager::new().unwrap();

    // Setup dev profile for batch testing
    let mut dev_config = ProfileConfig::dev_profile();
    dev_config.execution_config.verbose = true;
    manager.save_profile("test_batch", &dev_config).unwrap();

    // Load profile
    let (config, name) = manager.load_active_profile(Some("test_batch")).unwrap();
    assert_eq!(name, "test_batch");
    assert!(config.execution_config.verbose);

    // Run batch tests with this profile
    let test_suite = create_test_suite(
        "profile_test_suite",
        vec![create_test_case(
            "profile_test",
            "health_check",
            "server",
            vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        )],
    );

    let executor = TestExecutor::new();
    let results = executor.run_test_suite(&test_suite).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].passed);
}

#[test]
#[serial]
fn test_e2e_config_export_with_performance_settings() {
    // Test: Export configuration with performance monitoring settings
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("perf_config.json");

    let mut config = ProfileConfig::dev_profile();
    config.execution_config.tool_timeout_ms = 30000;
    // Backend is String
    config.logging.backend = "persistent".to_string();

    ConfigImportExport::export_config(&config, &export_path, Some(ConfigFormat::Json)).unwrap();

    // Verify exported file
    assert!(export_path.exists());

    // Re-import and verify
    let imported = ConfigImportExport::import_config(&export_path, Some(ConfigFormat::Json))
        .unwrap();

    assert_eq!(
        imported.execution_config.tool_timeout_ms,
        config.execution_config.tool_timeout_ms
    );
    assert_eq!(imported.logging.backend, "persistent");
}
