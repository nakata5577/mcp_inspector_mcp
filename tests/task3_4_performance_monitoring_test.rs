//! Integration tests for Task 3.4: Performance Monitoring CLI Integration
//!
//! This module tests the integration of performance monitoring features
//! including metrics collection, report generation, and bottleneck detection.

use mcp_inspector_mcp::models::{MetricStatus, TimeWindow};
use mcp_inspector_mcp::services::{
    BottleneckDetector, DetectionConfig, MetricsCollector, ReportFormat, ReportService,
};
use std::sync::Arc;

/// Helper function to create a metrics collector with test data
fn create_test_metrics_collector() -> Arc<MetricsCollector> {
    let collector = Arc::new(MetricsCollector::new());

    // Add test metrics for server1
    for i in 0..10 {
        collector.record_metric(
            "server1".to_string(),
            "tool1".to_string(),
            100 + i * 10,
            MetricStatus::Success,
            i % 3 == 0,
            i % 2 == 0,
        );
    }

    // Add test metrics for server2 with higher response times
    for i in 0..5 {
        collector.record_metric(
            "server2".to_string(),
            "tool2".to_string(),
            1500 + i * 100,
            if i == 4 {
                MetricStatus::Error
            } else {
                MetricStatus::Success
            },
            false,
            false,
        );
    }

    collector
}

// ============================================================================
// Test Group 1: Metrics Collection (3 tests)
// ============================================================================

#[test]
fn test_metrics_collection_tool_execution() {
    let collector = MetricsCollector::new();

    collector.record_metric(
        "test_server".to_string(),
        "test_tool".to_string(),
        250,
        MetricStatus::Success,
        false,
        false,
    );

    assert_eq!(collector.total_metrics(), 1);

    let metrics = collector.get_metrics(TimeWindow::LastHour);
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].server_name, "test_server");
    assert_eq!(metrics[0].tool_name, "test_tool");
    assert_eq!(metrics[0].response_time_ms, 250);
}

#[test]
fn test_metrics_collection_cache_hit_tracking() {
    let collector = MetricsCollector::new();

    // Record metrics with cache hits
    collector.record_metric(
        "server".to_string(),
        "tool".to_string(),
        100,
        MetricStatus::Success,
        true,
        false,
    );

    collector.record_metric(
        "server".to_string(),
        "tool".to_string(),
        150,
        MetricStatus::Success,
        false,
        false,
    );

    let aggregated = collector.aggregate_metrics("server", None, TimeWindow::LastHour);
    assert_eq!(aggregated.total_requests, 2);
    assert!((aggregated.cache_hit_rate - 50.0).abs() < 0.1);
}

#[test]
fn test_metrics_collection_error_tracking() {
    let collector = MetricsCollector::new();

    // Record successful and failed metrics
    for i in 0..10 {
        let status = if i < 8 {
            MetricStatus::Success
        } else {
            MetricStatus::Error
        };
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            100,
            status,
            false,
            false,
        );
    }

    let aggregated = collector.aggregate_metrics("server", None, TimeWindow::LastHour);
    assert_eq!(aggregated.total_requests, 10);
    assert_eq!(aggregated.successful_requests, 8);
    assert_eq!(aggregated.failed_requests, 2);
    assert!((aggregated.error_rate - 20.0).abs() < 0.1);
}

// ============================================================================
// Test Group 2: Report Generation (6 tests)
// ============================================================================

#[test]
fn test_report_generation_console_format() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Console)
        .expect("Failed to generate console report");

    assert!(report.contains("Performance Report"));
    assert!(report.contains("Server: server1"));
    assert!(report.contains("Server: server2"));
    assert!(report.contains("Response Time:"));
    assert!(report.contains("Throughput:"));
}

#[test]
fn test_report_generation_json_format() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Json)
        .expect("Failed to generate JSON report");

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&report).expect("Report is not valid JSON");

    assert!(parsed.is_array());
    let array = parsed.as_array().unwrap();
    assert!(!array.is_empty());
}

#[test]
fn test_report_generation_html_format() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Html)
        .expect("Failed to generate HTML report");

    assert!(report.contains("<!DOCTYPE html>"));
    assert!(report.contains("<html"));
    assert!(report.contains("Performance Report"));
    assert!(report.contains("</html>"));
    assert!(report.contains("<table"));
}

#[test]
fn test_report_generation_time_window_1h() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Console)
        .expect("Failed to generate report");

    assert!(report.contains("Last Hour"));
}

#[test]
fn test_report_generation_time_window_24h() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::Last24Hours, ReportFormat::Console)
        .expect("Failed to generate report");

    assert!(report.contains("Last 24 Hours"));
}

#[test]
fn test_report_generation_time_window_7d() {
    let collector = create_test_metrics_collector();
    let service = ReportService::new(collector);

    let report = service
        .generate_report(TimeWindow::Last7Days, ReportFormat::Console)
        .expect("Failed to generate report");

    assert!(report.contains("Last 7 Days"));
}

// ============================================================================
// Test Group 3: Bottleneck Detection (4 tests)
// ============================================================================

#[test]
fn test_bottleneck_detection_high_response_time() {
    let collector = MetricsCollector::new();

    // Add metrics with high response times (above 1000ms threshold)
    for i in 0..5 {
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            1200 + i * 100,
            MetricStatus::Success,
            false,
            false,
        );
    }

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(Arc::new(collector), config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    assert!(!bottlenecks.is_empty());
    assert!(bottlenecks
        .iter()
        .any(|b| matches!(b.bottleneck_type, mcp_inspector_mcp::services::BottleneckType::HighResponseTime)));
}

#[test]
fn test_bottleneck_detection_high_error_rate() {
    let collector = MetricsCollector::new();

    // Add metrics with high error rate (above 5% threshold)
    for i in 0..10 {
        let status = if i < 9 {
            MetricStatus::Success
        } else {
            MetricStatus::Error
        };
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            500,
            status,
            false,
            false,
        );
    }

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(Arc::new(collector), config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    assert!(!bottlenecks.is_empty());
    assert!(bottlenecks
        .iter()
        .any(|b| matches!(b.bottleneck_type, mcp_inspector_mcp::services::BottleneckType::HighErrorRate)));
}

#[test]
fn test_bottleneck_detection_low_throughput() {
    let collector = MetricsCollector::new();

    // Add only 1 metric (very low throughput)
    collector.record_metric(
        "server".to_string(),
        "tool".to_string(),
        500,
        MetricStatus::Success,
        false,
        false,
    );

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(Arc::new(collector), config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    assert!(!bottlenecks.is_empty());
    assert!(bottlenecks
        .iter()
        .any(|b| matches!(b.bottleneck_type, mcp_inspector_mcp::services::BottleneckType::LowThroughput)));
}

#[test]
fn test_bottleneck_detection_alert_format() {
    let collector = MetricsCollector::new();

    // Add metrics with high response times
    for _ in 0..5 {
        collector.record_metric(
            "test_server".to_string(),
            "test_tool".to_string(),
            1500,
            MetricStatus::Success,
            false,
            false,
        );
    }

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(Arc::new(collector), config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    assert!(!bottlenecks.is_empty());
    let alert = bottlenecks[0].format_alert();

    assert!(alert.contains("BOTTLENECK DETECTED"));
    assert!(alert.contains("Server:"));
    assert!(alert.contains("Type:"));
    assert!(alert.contains("Severity:"));
    assert!(alert.contains("Recommendation:"));
}

// ============================================================================
// Test Group 4: CLI Integration (2 tests)
// ============================================================================

#[test]
fn test_cli_metrics_option_integration() {
    // This test verifies that metrics aggregation works correctly
    // which is the backend for the --metrics CLI option
    let collector = create_test_metrics_collector();

    let server_metrics = collector.aggregate_by_server(TimeWindow::LastHour);

    assert!(!server_metrics.is_empty());
    assert!(server_metrics.contains_key("server1"));
    assert!(server_metrics.contains_key("server2"));

    let server1_metrics = server_metrics.get("server1").unwrap();
    assert_eq!(server1_metrics.server_name, "server1");
    assert!(server1_metrics.total_requests > 0);
}

#[test]
fn test_cli_detect_bottlenecks_option_integration() {
    // This test verifies that bottleneck detection works correctly
    // which is the backend for the --detect-bottlenecks CLI option
    let collector = create_test_metrics_collector();

    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(collector, config);
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);

    // server2 has high response times (1500ms+), should trigger bottleneck
    assert!(!bottlenecks.is_empty());

    let has_server2_bottleneck = bottlenecks
        .iter()
        .any(|b| b.server_name == "server2");
    assert!(has_server2_bottleneck);
}
