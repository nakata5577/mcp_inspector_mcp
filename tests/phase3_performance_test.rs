//! Phase 3 Performance Tests
//!
//! This module contains comprehensive performance tests for Phase 3 features.
//! Tests are designed to run quickly in CI environments using mock implementations.
//!
//! Test Categories:
//! - Response Time Tests: Measure and validate response times for various operations
//! - Throughput Tests: Test system capacity with concurrent and batch operations
//! - Memory Usage Tests: Validate memory behavior during extended operations
//! - Stress Tests: Test system behavior under high load conditions
//!
//! Total Test Count: 18 performance tests

use mcp_inspector_mcp::models::{MetricStatus, TimeWindow};
use mcp_inspector_mcp::services::{
    BottleneckDetector, DetectionConfig, MetricsCollector, ReportFormat, ReportService,
    TestExecutor,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a MetricsCollector with realistic test data
fn create_performance_test_metrics(operations_count: usize) -> Arc<MetricsCollector> {
    let collector = Arc::new(MetricsCollector::new());

    // Simulate fast server with varying response times
    for i in 0..operations_count {
        let response_time = 50 + (i % 100) as u64; // 50-150ms range
        collector.record_metric(
            "fast_server".to_string(),
            format!("tool_{}", i % 10),
            response_time,
            MetricStatus::Success,
            i % 4 == 0, // 25% cache hit rate
            i % 3 == 0, // 33% connection reuse
        );
    }

    collector
}

/// Creates a MetricsCollector with slow server metrics
fn create_slow_server_metrics(operations_count: usize) -> Arc<MetricsCollector> {
    let collector = Arc::new(MetricsCollector::new());

    for i in 0..operations_count {
        let response_time = 1000 + (i % 500) as u64; // 1000-1500ms range
        collector.record_metric(
            "slow_server".to_string(),
            "slow_tool".to_string(),
            response_time,
            if i % 10 == 0 {
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
// Test Group 1: Response Time Tests (6 tests)
// ============================================================================

#[test]
fn test_perf_response_time_single_operation() {
    // Test: Measure response time for a single metrics recording operation
    let collector = MetricsCollector::new();
    let start = Instant::now();

    collector.record_metric(
        "test_server".to_string(),
        "test_tool".to_string(),
        100,
        MetricStatus::Success,
        false,
        false,
    );

    let duration = start.elapsed();

    // Recording should be extremely fast (< 10ms)
    assert!(
        duration < Duration::from_millis(10),
        "Single operation took {:?}, expected < 10ms",
        duration
    );
    assert_eq!(collector.total_metrics(), 1);
}

#[test]
fn test_perf_response_time_bulk_recording() {
    // Test: Measure response time for bulk metrics recording
    let collector = MetricsCollector::new();
    let operations_count = 1000;
    let start = Instant::now();

    for i in 0..operations_count {
        collector.record_metric(
            "server".to_string(),
            format!("tool_{}", i % 10),
            50 + i,
            MetricStatus::Success,
            false,
            false,
        );
    }

    let duration = start.elapsed();

    // 1000 operations should complete quickly (< 100ms)
    assert!(
        duration < Duration::from_millis(100),
        "1000 operations took {:?}, expected < 100ms",
        duration
    );
    assert_eq!(collector.total_metrics(), operations_count as usize);
}

#[test]
fn test_perf_response_time_aggregation() {
    // Test: Measure response time for metrics aggregation
    let collector = create_performance_test_metrics(500);
    let start = Instant::now();

    let _aggregated = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);

    let duration = start.elapsed();

    // Aggregation should be fast (< 50ms)
    assert!(
        duration < Duration::from_millis(50),
        "Aggregation took {:?}, expected < 50ms",
        duration
    );
}

#[test]
fn test_perf_response_time_server_aggregation() {
    // Test: Measure response time for server-level aggregation
    let collector = create_performance_test_metrics(300);
    let start = Instant::now();

    let _aggregated = collector.aggregate_by_server(TimeWindow::LastHour);

    let duration = start.elapsed();

    // Server aggregation should be fast (< 50ms)
    assert!(
        duration < Duration::from_millis(50),
        "Server aggregation took {:?}, expected < 50ms",
        duration
    );
}

#[test]
fn test_perf_response_time_report_generation_console() {
    // Test: Measure response time for console report generation
    let collector = create_performance_test_metrics(200);
    let service = ReportService::new(collector);
    let start = Instant::now();

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Console)
        .unwrap();

    let duration = start.elapsed();

    // Report generation should be reasonably fast (< 100ms)
    assert!(
        duration < Duration::from_millis(100),
        "Console report took {:?}, expected < 100ms",
        duration
    );
    assert!(!report.is_empty());
}

#[test]
fn test_perf_response_time_report_generation_json() {
    // Test: Measure response time for JSON report generation
    let collector = create_performance_test_metrics(200);
    let service = ReportService::new(collector);
    let start = Instant::now();

    let report = service
        .generate_report(TimeWindow::LastHour, ReportFormat::Json)
        .unwrap();

    let duration = start.elapsed();

    // JSON report should be fast (< 100ms)
    assert!(
        duration < Duration::from_millis(100),
        "JSON report took {:?}, expected < 100ms",
        duration
    );
    assert!(!report.is_empty());

    // Verify JSON is valid
    let _parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
}

// ============================================================================
// Test Group 2: Throughput Tests (5 tests)
// ============================================================================

#[test]
fn test_perf_throughput_sequential_operations() {
    // Test: Measure throughput for sequential operations
    let collector = MetricsCollector::new();
    let operations_count = 1000;
    let start = Instant::now();

    for _i in 0..operations_count {
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            50,
            MetricStatus::Success,
            false,
            false,
        );
    }

    let duration = start.elapsed();
    let ops_per_second = operations_count as f64 / duration.as_secs_f64();

    // Should achieve at least 10,000 ops/sec
    assert!(
        ops_per_second > 10_000.0,
        "Throughput: {:.0} ops/sec, expected > 10,000 ops/sec",
        ops_per_second
    );
}

#[tokio::test]
async fn test_perf_throughput_concurrent_test_execution() {
    // Test: Measure throughput for concurrent test execution
    use mcp_inspector_mcp::models::{Assertion, TestCase, TestConfig, TestSuite};

    let test_cases: Vec<TestCase> = (0..10)
        .map(|i| TestCase {
            name: format!("test_{}", i),
            description: None,
            server: "test_server".to_string(),
            tool: "health_check".to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![Assertion::Status {
                expected: "healthy".to_string(),
            }],
        })
        .collect();

    let test_suite = TestSuite {
        name: "throughput_test".to_string(),
        version: "1.0".to_string(),
        description: None,
        config: TestConfig {
            parallel: true,
            fail_fast: false,
            timeout_ms: 5000,
            retry_count: 0,
        },
        tests: test_cases,
    };

    let executor = TestExecutor::new();
    let start = Instant::now();

    let results = executor.run_test_suite(&test_suite).await.unwrap();

    let duration = start.elapsed();

    // All tests should pass
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.passed));

    // Parallel execution should be faster than sequential
    // With 50ms mock delay per test, parallel should be < 1 second
    assert!(
        duration < Duration::from_secs(1),
        "Parallel execution took {:?}, expected < 1s",
        duration
    );
}

#[test]
fn test_perf_throughput_metrics_aggregation_multiple_servers() {
    // Test: Measure throughput for aggregating metrics from multiple servers
    let collector = Arc::new(MetricsCollector::new());

    // Add metrics for 5 different servers
    for server_id in 0..5 {
        for i in 0..100 {
            collector.record_metric(
                format!("server_{}", server_id),
                format!("tool_{}", i % 10),
                50 + i,
                MetricStatus::Success,
                false,
                false,
            );
        }
    }

    let start = Instant::now();
    let aggregated = collector.aggregate_by_server(TimeWindow::LastHour);
    let duration = start.elapsed();

    // Should aggregate 500 metrics across 5 servers quickly (< 100ms)
    assert_eq!(aggregated.len(), 5);
    assert!(
        duration < Duration::from_millis(100),
        "Multi-server aggregation took {:?}, expected < 100ms",
        duration
    );
}

#[test]
fn test_perf_throughput_bottleneck_detection() {
    // Test: Measure throughput for bottleneck detection
    let collector = create_slow_server_metrics(100);
    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(collector, config);

    let start = Instant::now();
    let bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);
    let duration = start.elapsed();

    // Bottleneck detection should be fast (< 50ms)
    assert!(
        duration < Duration::from_millis(50),
        "Bottleneck detection took {:?}, expected < 50ms",
        duration
    );
    assert!(!bottlenecks.is_empty());
}

#[test]
fn test_perf_throughput_large_dataset_aggregation() {
    // Test: Measure throughput with large dataset
    let collector = create_performance_test_metrics(5000);
    let start = Instant::now();

    let _aggregated = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);

    let duration = start.elapsed();

    // Should handle 5000 metrics efficiently (< 200ms)
    assert!(
        duration < Duration::from_millis(200),
        "Large dataset aggregation took {:?}, expected < 200ms",
        duration
    );
}

// ============================================================================
// Test Group 3: Memory Usage Tests (3 tests)
// ============================================================================

#[test]
fn test_perf_memory_circular_buffer_limit() {
    // Test: Verify circular buffer properly limits memory usage
    let collector = MetricsCollector::new();
    let max_buffer_size = 10_000;

    // Add more metrics than the buffer can hold
    for i in 0..max_buffer_size + 1000 {
        collector.record_metric(
            "server".to_string(),
            "tool".to_string(),
            i as u64,
            MetricStatus::Success,
            false,
            false,
        );
    }

    // Buffer should be limited to MAX_METRICS_BUFFER_SIZE (10,000)
    assert_eq!(
        collector.total_metrics(),
        max_buffer_size,
        "Buffer size should be limited to {}",
        max_buffer_size
    );
}

#[test]
fn test_perf_memory_aggregation_no_leak() {
    // Test: Verify multiple aggregations don't cause memory accumulation
    let collector = create_performance_test_metrics(500);

    // Perform multiple aggregations
    for _ in 0..100 {
        let _aggregated = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);
    }

    // Metrics count should remain stable
    assert_eq!(
        collector.total_metrics(),
        500,
        "Metrics count should remain stable after aggregations"
    );
}

#[test]
fn test_perf_memory_report_generation_stability() {
    // Test: Verify report generation doesn't accumulate memory
    let collector = create_performance_test_metrics(300);
    let service = ReportService::new(collector.clone());

    // Generate multiple reports
    for i in 0..50 {
        let format = if i % 2 == 0 {
            ReportFormat::Console
        } else {
            ReportFormat::Json
        };

        let report = service.generate_report(TimeWindow::LastHour, format).unwrap();
        assert!(!report.is_empty());
    }

    // Metrics count should remain stable
    assert_eq!(
        collector.total_metrics(),
        300,
        "Metrics count should remain stable after report generation"
    );
}

// ============================================================================
// Test Group 4: Stress Tests (4 tests)
// ============================================================================

#[tokio::test]
async fn test_perf_stress_concurrent_metrics_recording() {
    // Test: Stress test with concurrent metrics recording
    use tokio::task;

    let collector = Arc::new(MetricsCollector::new());
    let mut handles = vec![];

    // Spawn 10 concurrent tasks, each recording 100 metrics
    for task_id in 0..10 {
        let collector_clone = collector.clone();
        let handle = task::spawn(async move {
            for i in 0..100 {
                collector_clone.record_metric(
                    format!("server_{}", task_id),
                    format!("tool_{}", i),
                    50 + i,
                    MetricStatus::Success,
                    false,
                    false,
                );
            }
        });
        handles.push(handle);
    }

    let start = Instant::now();

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();

    // Should handle concurrent recording (< 500ms)
    assert!(
        duration < Duration::from_millis(500),
        "Concurrent recording took {:?}, expected < 500ms",
        duration
    );
    assert_eq!(collector.total_metrics(), 1000);
}

#[tokio::test]
async fn test_perf_stress_rapid_test_execution() {
    // Test: Stress test with rapid sequential test execution
    use mcp_inspector_mcp::models::{Assertion, TestCase, TestConfig, TestSuite};

    let test_cases: Vec<TestCase> = (0..20)
        .map(|i| TestCase {
            name: format!("stress_test_{}", i),
            description: None,
            server: "test_server".to_string(),
            tool: if i % 2 == 0 {
                "health_check"
            } else {
                "tools_list"
            }
            .to_string(),
            arguments: serde_json::json!({}),
            expect_error: false,
            assertions: vec![
                if i % 2 == 0 {
                    Assertion::Status {
                        expected: "healthy".to_string(),
                    }
                } else {
                    Assertion::FieldExists {
                        field: "tools".to_string(),
                    }
                },
            ],
        })
        .collect();

    let test_suite = TestSuite {
        name: "stress_test_suite".to_string(),
        version: "1.0".to_string(),
        description: None,
        config: TestConfig {
            parallel: false,
            fail_fast: false,
            timeout_ms: 5000,
            retry_count: 0,
        },
        tests: test_cases,
    };

    let executor = TestExecutor::new();
    let start = Instant::now();

    let results = executor.run_test_suite(&test_suite).await.unwrap();

    let duration = start.elapsed();

    // All tests should pass
    assert_eq!(results.len(), 20);
    assert!(results.iter().all(|r| r.passed));

    // Sequential execution should complete in reasonable time (< 3 seconds)
    assert!(
        duration < Duration::from_secs(3),
        "Rapid test execution took {:?}, expected < 3s",
        duration
    );
}

#[test]
fn test_perf_stress_high_volume_aggregation() {
    // Test: Stress test with high volume of aggregations
    let collector = create_performance_test_metrics(1000);
    let start = Instant::now();

    // Perform many aggregations
    for _ in 0..100 {
        let _server_agg = collector.aggregate_by_server(TimeWindow::LastHour);
        let _tool_agg = collector.aggregate_by_tool("fast_server", TimeWindow::LastHour);
        let _metrics_agg = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);
    }

    let duration = start.elapsed();

    // Should handle 300 aggregations (< 2 seconds)
    assert!(
        duration < Duration::from_secs(2),
        "High volume aggregation took {:?}, expected < 2s",
        duration
    );
}

#[test]
fn test_perf_stress_mixed_operations() {
    // Test: Stress test with mixed operations (recording, aggregation, reporting)
    let collector = Arc::new(MetricsCollector::new());
    let service = ReportService::new(collector.clone());
    let start = Instant::now();

    // Perform mixed operations
    for i in 0..100 {
        // Record metrics
        collector.record_metric(
            format!("server_{}", i % 5),
            format!("tool_{}", i % 10),
            50 + i,
            MetricStatus::Success,
            i % 4 == 0,
            i % 3 == 0,
        );

        // Periodic aggregation
        if i % 10 == 0 {
            let _agg = collector.aggregate_metrics(
                &format!("server_{}", i % 5),
                None,
                TimeWindow::LastHour,
            );
        }

        // Periodic reporting
        if i % 20 == 0 {
            let _report = service.generate_report(TimeWindow::LastHour, ReportFormat::Console);
        }
    }

    let duration = start.elapsed();

    // Mixed operations should complete efficiently (< 500ms)
    assert!(
        duration < Duration::from_millis(500),
        "Mixed operations took {:?}, expected < 500ms",
        duration
    );
    assert_eq!(collector.total_metrics(), 100);
}

// ============================================================================
// Performance Benchmarking Tests (informational, not strict assertions)
// ============================================================================

#[test]
fn test_perf_benchmark_summary() {
    // Test: Provide a comprehensive performance benchmark summary
    println!("\n=== Performance Benchmark Summary ===\n");

    // 1. Single operation latency
    let collector = MetricsCollector::new();
    let start = Instant::now();
    collector.record_metric(
        "server".to_string(),
        "tool".to_string(),
        100,
        MetricStatus::Success,
        false,
        false,
    );
    let single_op_latency = start.elapsed();
    println!("Single operation latency: {:?}", single_op_latency);

    // 2. Bulk recording throughput
    let collector = MetricsCollector::new();
    let operations = 10000;
    let start = Instant::now();
    for i in 0..operations {
        collector.record_metric(
            "server".to_string(),
            format!("tool_{}", i % 10),
            50 + i,
            MetricStatus::Success,
            false,
            false,
        );
    }
    let bulk_duration = start.elapsed();
    let throughput = operations as f64 / bulk_duration.as_secs_f64();
    println!(
        "Bulk recording: {} ops in {:?} ({:.0} ops/sec)",
        operations, bulk_duration, throughput
    );

    // 3. Aggregation performance
    let collector = create_performance_test_metrics(1000);
    let start = Instant::now();
    let _agg = collector.aggregate_metrics("fast_server", None, TimeWindow::LastHour);
    let agg_duration = start.elapsed();
    println!("Aggregation (1000 metrics): {:?}", agg_duration);

    // 4. Report generation performance
    let collector = create_performance_test_metrics(500);
    let service = ReportService::new(collector);
    let start = Instant::now();
    let _report = service.generate_report(TimeWindow::LastHour, ReportFormat::Console);
    let report_duration = start.elapsed();
    println!("Console report (500 metrics): {:?}", report_duration);

    // 5. Bottleneck detection performance
    let collector = create_slow_server_metrics(200);
    let config = DetectionConfig::default();
    let detector = BottleneckDetector::new(collector, config);
    let start = Instant::now();
    let _bottlenecks = detector.detect_bottlenecks(TimeWindow::LastHour);
    let detection_duration = start.elapsed();
    println!("Bottleneck detection (200 metrics): {:?}", detection_duration);

    println!("\n=== End of Performance Benchmark ===\n");

    // This test always passes - it's for information only
}
